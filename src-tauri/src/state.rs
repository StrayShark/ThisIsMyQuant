use std::sync::Arc;

use crate::adapters::{AkshareClient, JinshiClient, LlmRouter};
use crate::config::Config;
use crate::db::Database;
use crate::services::{AnalysisSchedulerHandle, MarketPollHandle, NewsPollHandle};

pub struct AppState {
    pub config: Config,
    pub db: Arc<Database>,
    pub akshare: AkshareClient,
    pub jinshi: JinshiClient,
    pub llm: LlmRouter,
    pub market_poll: Option<Arc<MarketPollHandle>>,
    pub news_poll: Option<NewsPollHandle>,
    pub analysis_scheduler: std::sync::Mutex<Option<AnalysisSchedulerHandle>>,
    pub akshare_ready: bool,
}
