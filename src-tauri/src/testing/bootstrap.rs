//! 测试用 AppState 引导（无 Tauri 窗口与后台任务）。

use std::sync::{Arc, Mutex, RwLock};

use reqwest::Client;
use tokio::sync::Mutex as AsyncMutex;

use crate::adapters::{AkshareClient, JinshiClient, LlmRouter};
use crate::config::Config;
use crate::db::Database;
use crate::engine::anomaly::{AnomalyConfig, AnomalyDetector};
use crate::services::{
    maybe_import_llm_from_env_dev, hydrate_config_llm, new_schedule_status,
    AnomalyWatcher, BatchAnalysisHandle,
};
use crate::state::AppState;

pub async fn bootstrap_test_state() -> Arc<AppState> {
    let secrets = Config::load();
    let db = Arc::new(Database::open(&secrets.database_path).expect("db open"));
    db.init_schema().expect("schema");

    let prefs = db
        .load_user_preferences()
        .ok()
        .flatten()
        .unwrap_or_default()
        .normalize();
    let mut config = Config::load_with_preferences(prefs);
    let _ = hydrate_config_llm(&db, &mut config);
    let _ = maybe_import_llm_from_env_dev(&db, &mut config);

    let config_store = Arc::new(RwLock::new(config.clone()));
    let http = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)")
        .build()
        .expect("http client");

    let akshare = AkshareClient::new(http.clone());
    let akshare_ready = config.akshare_enabled && akshare.is_ready();

    let jinshi = Arc::new(AsyncMutex::new(JinshiClient::new(http, &config)));
    if config.jinshi_enabled {
        let _ = jinshi.lock().await.connect().await;
    }

    let llm = Arc::new(RwLock::new(LlmRouter::new(
        config.llm_providers.clone(),
        config.default_llm_provider.clone(),
    )));

    let anomaly_cfg = AnomalyConfig {
        enabled: config.anomaly_enabled,
        price_pct_threshold: config.anomaly_price_pct,
        window_secs: config.anomaly_window_secs,
        cooldown_secs: config.anomaly_cooldown_secs,
    };

    Arc::new(AppState {
        config_store,
        db,
        akshare,
        jinshi,
        llm,
        market_poll: Arc::new(AsyncMutex::new(None)),
        news_poll: Arc::new(AsyncMutex::new(None)),
        schedule: Mutex::new(None),
        schedule_status: new_schedule_status(
            config.schedule_interval_hours.max(1),
            config.schedule_enabled && !config.watchlist.is_empty(),
        ),
        akshare_ready,
        anomaly: Arc::new(AnomalyWatcher::new(Arc::new(AnomalyDetector::new(anomaly_cfg)))),
        backfill_status: crate::services::new_status_handle(),
        feed_source: config.market_feed.clone(),
        batch_analysis: BatchAnalysisHandle::new(),
    })
}
