#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod adapters;
pub mod commands;
pub mod config;
pub mod crypto;
pub mod db;
pub mod engine;
pub mod error;
pub mod logging;
pub mod models;
pub mod services;
pub mod state;
pub mod testing;

use std::sync::{Arc, RwLock};

use reqwest::Client;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use adapters::{feed_from_config, AkshareClient, JinshiClient, LlmRouter};
use config::{load_user_preferences, Config};
use db::Database;
use engine::anomaly::{AnomalyConfig, AnomalyDetector};
use services::{
    ingest_poll, new_schedule_status, new_status_handle, spawn_calendar_reminder,
    spawn_daily_briefing, spawn_data_maintenance, spawn_history_backfill, AnomalyWatcher,
    BatchAnalysisHandle, IngestDeps, LiquidityJobHandle, MarketPollHandle, NewsPollHandle,
    ScheduleHandle,
};
use state::AppState;

#[cfg(debug_assertions)]
fn write_e2e_ready(_config: &Config, llm_providers: &[String]) {
    use std::fs;
    let root = config::project_root();
    let dir = root.join("data");
    let _ = fs::create_dir_all(&dir);
    let payload = serde_json::json!({
        "ready": true,
        "at": chrono::Utc::now().to_rfc3339(),
        "llm_providers": llm_providers,
        "e2e_http_port": testing::E2E_HTTP_PORT,
    });
    let path = dir.join("e2e-ready.json");
    if let Ok(text) = serde_json::to_string_pretty(&payload) {
        let _ = fs::write(path, text);
    }
}

async fn bootstrap(app: tauri::AppHandle) {
    log::info!("=== ThisIsMyQuant starting (Rust core) ===");
    let secrets = Config::load();
    let db = Arc::new(Database::open(&secrets.database_path).expect("db open"));
    if let Err(e) = db.init_schema() {
        log::error!("schema init failed: {e}");
    }

    let prefs = load_user_preferences(&db, &secrets.database_path).unwrap_or_else(|e| {
        log::warn!("load preferences: {e}, using defaults");
        crate::config::UserPreferences::default().normalize()
    });
    let mut config = Config::load_with_preferences(prefs.clone());
    if let Err(e) = crate::services::hydrate_config_llm(&db, &mut config) {
        log::warn!("hydrate llm credentials: {e}");
    }
    if let Err(e) = crate::services::maybe_import_llm_from_env_dev(&db, &mut config) {
        log::warn!("dev llm env import: {e}");
    }
    let config_store = Arc::new(RwLock::new(config.clone()));
    let http = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)")
        .build()
        .expect("http client");

    let akshare = AkshareClient::new(http.clone());
    let akshare_ready = config.akshare_enabled && akshare.is_ready();

    let jinshi = Arc::new(Mutex::new(JinshiClient::new(http.clone(), &config)));
    if config.jinshi_enabled {
        if let Err(e) = jinshi.lock().await.connect().await {
            log::error!("jinshi connect: {e}");
        }
    }

    let llm = Arc::new(RwLock::new(LlmRouter::new(
        config.llm_providers.clone(),
        config.default_llm_provider.clone(),
    )));
    log::info!(
        "LLM providers: {:?}",
        llm.read()
            .unwrap_or_else(|e| e.into_inner())
            .available_providers()
    );

    let feed = feed_from_config(&config.market_feed, akshare.clone());
    let feed_source = feed.source_name().to_string();

    let anomaly_cfg = AnomalyConfig {
        enabled: config.anomaly_enabled,
        price_pct_threshold: config.anomaly_price_pct,
        window_secs: config.anomaly_window_secs,
        cooldown_secs: config.anomaly_cooldown_secs,
    };
    let anomaly = Arc::new(AnomalyWatcher::new(Arc::new(AnomalyDetector::new(
        anomaly_cfg,
    ))));
    let backfill_status = new_status_handle();
    let market_poll_slot: Arc<Mutex<Option<Arc<MarketPollHandle>>>> = Arc::new(Mutex::new(None));

    let news_poll_slot: Arc<Mutex<Option<NewsPollHandle>>> = Arc::new(Mutex::new(None));

    if config.jinshi_enabled {
        let jinshi_guard = jinshi.lock().await;
        if jinshi_guard.is_connected() {
            let llm_snap = llm.read().unwrap_or_else(|e| e.into_inner()).clone();
            let deps = IngestDeps {
                jinshi: &jinshi_guard,
                db: &db,
                llm: Some(&llm_snap),
                classify_cfg: &config.news_classify,
                default_llm_provider: &config.default_llm_provider,
            };
            if let Err(e) = ingest_poll(&deps, 15).await {
                log::warn!("initial news ingest: {e}");
            }
        }
        let poll = NewsPollHandle::start(
            jinshi_guard.clone(),
            db.clone(),
            llm.read().unwrap_or_else(|e| e.into_inner()).clone(),
            &config,
            config.jinshi_poll_interval,
        );
        *news_poll_slot.lock().await = Some(poll);
    }

    if let Ok(contracts) = akshare.get_contracts().await {
        let _ = db.save_contracts(&contracts);
    }

    let schedule_status = new_schedule_status(
        config.schedule_interval_hours.max(1),
        config.schedule_enabled,
    );

    let user_preferences = Arc::new(RwLock::new(prefs));

    #[cfg(debug_assertions)]
    let llm_provider_names: Vec<String> = llm
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .available_providers();

    let state = Arc::new(AppState {
        config_store: config_store.clone(),
        user_preferences,
        db,
        akshare,
        jinshi,
        llm,
        market_poll: market_poll_slot.clone(),
        news_poll: news_poll_slot,
        schedule: std::sync::Mutex::new(None),
        schedule_status: schedule_status.clone(),
        akshare_ready,
        anomaly: anomaly.clone(),
        backfill_status,
        feed_source: feed_source.clone(),
        batch_analysis: BatchAnalysisHandle::new(),
        quote_cache: Arc::new(std::sync::RwLock::new(crate::services::QuoteCache::new())),
    });

    let poll_symbols = crate::engine::sectors::core_product_symbols();
    if config.akshare_realtime_enabled && !poll_symbols.is_empty() {
        let poll = Arc::new(MarketPollHandle::start(
            app.clone(),
            feed,
            state.clone(),
            Some(anomaly),
            poll_symbols,
            config.realtime_poll_interval,
        ));
        *market_poll_slot.lock().await = Some(poll);
    }

    spawn_history_backfill(state.clone(), state.backfill_status.clone());

    let scheduler = ScheduleHandle::start(app.clone(), state.clone(), &config, schedule_status);
    *state.schedule.lock().unwrap_or_else(|e| e.into_inner()) = Some(scheduler);

    LiquidityJobHandle::spawn(state.clone());

    spawn_data_maintenance(state.clone());
    spawn_calendar_reminder(app.clone(), state.clone());
    spawn_daily_briefing(app.clone(), state.clone(), state.schedule_status.clone());

    app.manage(state.clone());

    #[cfg(debug_assertions)]
    {
        write_e2e_ready(&config, &llm_provider_names);
        testing::spawn_e2e_http_server(state);
    }

    log::info!("=== ThisIsMyQuant ready (feed={feed_source}) ===");
    let _ = app.emit("app-ready", ());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .level_for("app_lib", log::LevelFilter::Debug)
                .level_for("reqwest", log::LevelFilter::Warn)
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("h2", log::LevelFilter::Warn)
                .level_for("rustls", log::LevelFilter::Warn)
                .build(),
        )
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
            commands::list_news_by_ids,
            commands::list_calendar_events,
            commands::list_unclassified_news,
            commands::get_settings,
            commands::get_runtime_status,
            commands::get_symbol_context,
            commands::market_subscribe,
            commands::market_unsubscribe,
            commands::get_realtime_quotes,
            commands::reload_config,
            commands::get_llm_setup,
            commands::save_llm_setup,
            commands::get_user_preferences,
            commands::save_user_preferences,
            commands::export_klines_csv,
            commands::export_reports_csv,
            commands::import_klines_csv,
            commands::get_professional_dashboard,
            commands::reclassify_news,
            commands::trigger_batch_analysis,
            commands::get_batch_status,
            commands::get_status_dashboard,
            commands::probe_ollama,
            commands::trigger_data_fetch,
            commands::trigger_comprehensive_analysis,
            commands::get_schedule_status,
            commands::run_client_e2e,
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
