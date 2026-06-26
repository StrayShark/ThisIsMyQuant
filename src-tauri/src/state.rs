use std::sync::{Arc, RwLock};

use tokio::sync::Mutex;

use crate::adapters::{AkshareClient, JinshiClient, LlmRouter};
use crate::config::{Config, LlmProviderConfig};
use crate::db::Database;
use crate::services::{
    AnomalyWatcher, BackfillStatusHandle, BatchAnalysisHandle, MarketPollHandle, NewsPollHandle,
    ScheduleHandle, ScheduleStatusHandle,
};

pub struct AppState {
    pub config_store: Arc<RwLock<Config>>,
    pub db: Arc<Database>,
    pub akshare: AkshareClient,
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
        *self.llm.write().unwrap_or_else(|e| e.into_inner()) =
            LlmRouter::new(providers, default);
    }
}
