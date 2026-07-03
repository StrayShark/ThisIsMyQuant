//! 偏好变更后热重启行情 / 资讯轮询。

use std::sync::Arc;

use tauri::AppHandle;

use crate::adapters::feed_from_config;
use crate::services::{ingest_poll, IngestDeps, MarketPollHandle, NewsPollHandle};
use crate::state::AppState;

pub async fn restart_runtime_polls(app: &AppHandle, state: &Arc<AppState>) {
    restart_market_poll(app, state).await;
    restart_news_poll(state).await;
}

async fn restart_market_poll(app: &AppHandle, state: &Arc<AppState>) {
    if let Some(poll) = state.poll_handle().await {
        poll.abort();
    }
    *state.market_poll.lock().await = None;

    let cfg = state.config().clone();
    let symbols = crate::engine::sectors::core_product_symbols();
    if !cfg.akshare_enabled || !cfg.akshare_realtime_enabled || symbols.is_empty() {
        log::info!("MarketPoll stopped (disabled or no symbols)");
        return;
    }

    let feed = feed_from_config(&cfg.market_feed, state.akshare.clone());
    let poll = Arc::new(MarketPollHandle::start(
        app.clone(),
        feed,
        state.clone(),
        Some(state.anomaly.clone()),
        symbols,
        cfg.realtime_poll_interval,
    ));
    *state.market_poll.lock().await = Some(poll);
}

async fn restart_news_poll(state: &Arc<AppState>) {
    {
        let mut slot = state.news_poll.lock().await;
        if let Some(poll) = slot.take() {
            poll.abort();
        }
    }

    let cfg = state.config().clone();
    if !cfg.jinshi_enabled {
        log::info!("NewsPoll stopped (jinshi disabled)");
        return;
    }

    let mut jinshi = state.jinshi.lock().await;
    jinshi.sync_config(&cfg);
    if !jinshi.is_connected() {
        if let Err(e) = jinshi.connect().await {
            log::warn!("jinshi reconnect on prefs save: {e}");
        }
    }
    if !jinshi.is_connected() {
        log::info!("NewsPoll not started (jinshi offline)");
        return;
    }

    let llm_snap = state.llm_snapshot();
    let deps = IngestDeps {
        jinshi: &jinshi,
        db: &state.db,
        llm: Some(&llm_snap),
        classify_cfg: &cfg.news_classify,
        default_llm_provider: &cfg.default_llm_provider,
    };
    if let Err(e) = ingest_poll(&deps, 15).await {
        log::warn!("news ingest after poll restart: {e}");
    }
    drop(jinshi);

    let poll = NewsPollHandle::start(
        state.jinshi.lock().await.clone(),
        state.db.clone(),
        state.llm_snapshot(),
        &cfg,
        cfg.jinshi_poll_interval,
    );
    *state.news_poll.lock().await = Some(poll);
}
