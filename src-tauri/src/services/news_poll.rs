use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::adapters::{JinshiClient, LlmRouter};
use crate::config::Config;
use crate::db::Database;
use crate::services::news_ingest::{ingest_poll, IngestDeps};

pub struct NewsPollHandle {
    _task: JoinHandle<()>,
}

impl NewsPollHandle {
    pub fn start(
        jinshi: JinshiClient,
        db: Arc<Database>,
        llm: LlmRouter,
        config: &Config,
        interval_secs: f64,
    ) -> Self {
        let classify_cfg = config.news_classify.clone();
        let default_provider = config.default_llm_provider.clone();
        let interval = interval_secs.max(30.0);
        let task = tokio::spawn(async move {
            let dur = tokio::time::Duration::from_secs_f64(interval);
            loop {
                let deps = IngestDeps {
                    jinshi: &jinshi,
                    db: &db,
                    llm: Some(&llm),
                    classify_cfg: &classify_cfg,
                    default_llm_provider: &default_provider,
                };
                if let Err(e) = ingest_poll(&deps, 15).await {
                    log::warn!("news ingest failed: {e}");
                }
                tokio::time::sleep(dur).await;
            }
        });
        log::info!("NewsPoll started (ingest+rule+llm classify), interval={interval}s");
        Self { _task: task }
    }
}
