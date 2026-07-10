//! 测试用 AppState 引导（无 Tauri 窗口与后台任务）。

use std::sync::{Arc, Mutex, RwLock};

use reqwest::Client;
use tokio::sync::Mutex as AsyncMutex;

use crate::adapters::{AkshareClient, AkshareStockProvider, JinshiClient, LlmRouter};
use crate::config::{load_user_preferences, Config};
use crate::db::Database;
use crate::engine::anomaly::{AnomalyConfig, AnomalyDetector};
use crate::services::{
    hydrate_config_llm, maybe_import_llm_from_env_dev, new_schedule_status, AnomalyWatcher,
    BatchAnalysisHandle, SimTradingService, StockDataSyncService, StockPaperTradingService,
};
use crate::state::AppState;

pub async fn bootstrap_test_state() -> Arc<AppState> {
    let secrets = Config::load();
    let db = Arc::new(Database::open(&secrets.database_path).expect("db open"));
    db.init_schema().expect("schema");

    let prefs = load_user_preferences(&db, &secrets.database_path)
        .unwrap_or_else(|_| crate::config::UserPreferences::default().normalize());
    let mut config = Config::load_with_preferences(prefs.clone());
    let _ = hydrate_config_llm(&db, &mut config);
    let _ = maybe_import_llm_from_env_dev(&db, &mut config);

    let config_store = Arc::new(RwLock::new(config.clone()));
    let http = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)")
        .build()
        .expect("http client");

    let akshare = AkshareClient::new(http.clone());
    let akshare_ready = config.akshare_enabled && akshare.is_ready();
    let stock_provider = AkshareStockProvider::new(http.clone());

    let jinshi = Arc::new(AsyncMutex::new(JinshiClient::new(http.clone(), &config)));
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

    let quote_cache = Arc::new(RwLock::new(crate::services::QuoteCache::new()));
    let sim_trading = Arc::new(SimTradingService::new(db.clone(), quote_cache.clone()));
    let _ = sim_trading.init_defaults();
    let stock_sync = Arc::new(StockDataSyncService::new(
        db.clone(),
        AkshareStockProvider::new(http.clone()),
    ));
    let stock_paper = Arc::new(StockPaperTradingService::new(db.clone()));

    Arc::new(AppState {
        config_store,
        user_preferences: Arc::new(RwLock::new(prefs)),
        db,
        akshare,
        stock_provider,
        jinshi,
        llm,
        market_poll: Arc::new(AsyncMutex::new(None)),
        news_poll: Arc::new(AsyncMutex::new(None)),
        schedule: Mutex::new(None),
        schedule_status: new_schedule_status(
            config.schedule_interval_hours.max(1),
            config.schedule_enabled,
        ),
        akshare_ready,
        anomaly: Arc::new(AnomalyWatcher::new(Arc::new(AnomalyDetector::new(
            anomaly_cfg,
        )))),
        backfill_status: crate::services::new_status_handle(),
        feed_source: config.market_feed.clone(),
        batch_analysis: BatchAnalysisHandle::new(),
        quote_cache,
        sim_trading,
        stock_sync,
        stock_paper,
        replay_runner: Arc::new(RwLock::new(None)),
    })
}
