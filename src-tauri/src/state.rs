use std::sync::{Arc, RwLock};

use tokio::sync::Mutex;

use crate::adapters::{AkshareClient, AkshareStockProvider, JinshiClient, LlmRouter};
use crate::config::{Config, LlmProviderConfig, UserPreferences};
use crate::db::Database;
use crate::models::Tick;
use crate::services::{
    resolve_prev_close, AnomalyWatcher, BackfillStatusHandle, BatchAnalysisHandle,
    MarketPollHandle, NewsPollHandle, QuoteCache, ReplayRunner, ScheduleHandle,
    ScheduleStatusHandle, SimTradingService, StockDataSyncService, StockPaperTradingService,
};

pub struct AppState {
    pub config_store: Arc<RwLock<Config>>,
    pub user_preferences: Arc<RwLock<UserPreferences>>,
    pub db: Arc<Database>,
    pub akshare: AkshareClient,
    pub stock_provider: AkshareStockProvider,
    pub jinshi: Arc<Mutex<JinshiClient>>,
    pub llm: Arc<RwLock<LlmRouter>>,
    pub market_poll: Arc<Mutex<Option<Arc<MarketPollHandle>>>>,
    pub news_poll: Arc<Mutex<Option<NewsPollHandle>>>,
    pub schedule: std::sync::Mutex<Option<ScheduleHandle>>,
    pub schedule_status: ScheduleStatusHandle,
    pub akshare_ready: bool,
    pub anomaly: Arc<AnomalyWatcher>,
    pub backfill_status: BackfillStatusHandle,
    pub feed_source: String,
    pub batch_analysis: BatchAnalysisHandle,
    pub quote_cache: Arc<RwLock<QuoteCache>>,
    pub sim_trading: Arc<SimTradingService>,
    pub stock_sync: Arc<StockDataSyncService>,
    pub stock_paper: Arc<StockPaperTradingService>,
    pub replay_runner: Arc<RwLock<Option<ReplayRunner>>>,
}

impl AppState {
    pub fn config(&self) -> std::sync::RwLockReadGuard<'_, Config> {
        self.config_store.read().unwrap_or_else(|e| e.into_inner())
    }

    pub async fn poll_handle(&self) -> Option<Arc<MarketPollHandle>> {
        self.market_poll.lock().await.clone()
    }

    pub fn llm_read(&self) -> std::sync::RwLockReadGuard<'_, LlmRouter> {
        self.llm.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn llm_snapshot(&self) -> LlmRouter {
        self.llm_read().clone()
    }

    pub fn replace_llm(&self, providers: Vec<LlmProviderConfig>, default: String) {
        *self.llm.write().unwrap_or_else(|e| e.into_inner()) = LlmRouter::new(providers, default);
    }

    pub fn user_prefs(&self) -> UserPreferences {
        self.user_preferences
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set_user_prefs(&self, prefs: UserPreferences) {
        *self
            .user_preferences
            .write()
            .unwrap_or_else(|e| e.into_inner()) = prefs;
    }

    pub fn preferences_file_path(&self) -> std::path::PathBuf {
        crate::config::preferences_path(&self.config().database_path)
    }

    pub async fn prev_close_for(&self, symbol: &str) -> Option<f64> {
        if let Some(p) = self
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .prev_close(symbol)
        {
            return Some(p);
        }
        resolve_prev_close(&self.db, &self.akshare, symbol).await
    }

    pub async fn apply_tick_to_quotes(
        &self,
        tick: &Tick,
        forming_daily: Option<crate::models::KLine>,
    ) {
        let sym = tick.symbol.to_lowercase();
        let prev_close = match self.prev_close_for(&sym).await {
            Some(p) if p > 0.0 => p,
            _ => tick.last_price,
        };
        self.quote_cache
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .update_from_tick(tick, prev_close, forming_daily);
    }
}
