#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod adapters;
pub mod commands;
pub mod config;
pub mod db;
pub mod engine;
pub mod error;
pub mod models;
pub mod services;
pub mod state;

use std::sync::Arc;

use reqwest::Client;
use tauri::{Emitter, Manager};

use adapters::{AkshareClient, JinshiClient, LlmRouter};
use config::Config;
use db::Database;
use services::{AnalysisSchedulerHandle, LiquidityJobHandle, MarketPollHandle, NewsPollHandle, ingest_poll, IngestDeps};
use state::AppState;

async fn bootstrap(app: tauri::AppHandle) {
    log::info!("=== ThisIsMyQuant starting (Rust core) ===");
    let config = Config::load();
    let http = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)")
        .build()
        .expect("http client");

    let db = Arc::new(
        Database::open(&config.database_path).expect("db open"),
    );
    if let Err(e) = db.init_schema() {
        log::error!("schema init failed: {e}");
    }

    let akshare = AkshareClient::new(http.clone());
    let akshare_ready = config.akshare_enabled && akshare.is_ready();

    let mut jinshi = JinshiClient::new(http.clone(), &config);
    if config.jinshi_enabled {
        if let Err(e) = jinshi.connect().await {
            log::error!("jinshi connect: {e}");
        }
    }

    let llm = LlmRouter::new(
        config.llm_providers.clone(),
        config.default_llm_provider.clone(),
    );
    log::info!("LLM providers: {:?}", llm.available_providers());

    let market_poll = if config.akshare_realtime_enabled && !config.watchlist.is_empty() {
        Some(Arc::new(MarketPollHandle::start(
            app.clone(),
            akshare.clone(),
            config.watchlist.clone(),
            config.realtime_poll_interval,
        )))
    } else {
        None
    };

    let news_poll = if config.jinshi_enabled {
        if jinshi.is_connected() {
            let deps = IngestDeps {
                jinshi: &jinshi,
                db: &db,
                llm: Some(&llm),
                classify_cfg: &config.news_classify,
                default_llm_provider: &config.default_llm_provider,
            };
            if let Err(e) = ingest_poll(&deps, 15).await {
                log::warn!("initial news ingest: {e}");
            }
        }
        Some(NewsPollHandle::start(
            jinshi.clone(),
            db.clone(),
            llm.clone(),
            &config,
            config.jinshi_poll_interval,
        ))
    } else {
        None
    };

    if let Ok(contracts) = akshare.get_contracts().await {
        let _ = db.save_contracts(&contracts);
    }

    let state = Arc::new(AppState {
        config: config.clone(),
        db,
        akshare,
        jinshi,
        llm,
        market_poll,
        news_poll,
        analysis_scheduler: std::sync::Mutex::new(None),
        akshare_ready,
    });

    let scheduler = AnalysisSchedulerHandle::start(app.clone(), state.clone(), &config);
    *state
        .analysis_scheduler
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(scheduler);

    LiquidityJobHandle::spawn(state.clone());

    app.manage(state);

    log::info!("=== ThisIsMyQuant ready ===");
    let _ = app.emit("app-ready", ());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_health,
            commands::list_products,
            commands::list_dimensions,
            commands::list_dimension_facts,
            commands::list_followups,
            commands::list_contracts,
            commands::get_klines,
            commands::list_reports,
            commands::get_report,
            commands::trigger_analysis,
            commands::stream_analysis,
            commands::analysis_followup,
            commands::list_news,
            commands::list_calendar_events,
            commands::get_settings,
            commands::market_subscribe,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                bootstrap(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
