//! 定时数据拉取：财经日历、资讯、core 品种 K 线增量。

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::adapters::default_calendar_range_from_today;
use crate::models::DataFetchSummary;
use crate::services::news_ingest::{ingest_poll, IngestDeps};
use crate::state::AppState;

pub const SCHEDULED_CALENDAR_CACHE_KEY: &str = "scheduled|calendar|14d|star3";

pub async fn run_data_fetch_cycle(state: &Arc<AppState>) -> Result<DataFetchSummary, String> {
    let mut summary = DataFetchSummary::default();
    let (jinshi_enabled, news_classify, default_llm) = {
        let cfg = state.config();
        (
            cfg.jinshi_enabled,
            cfg.news_classify.clone(),
            cfg.default_llm_provider.clone(),
        )
    };

    if jinshi_enabled {
        let (start, end) = default_calendar_range_from_today();
        match state
            .jinshi
            .lock()
            .await
            .fetch_calendar_events(start, end, 3, None)
            .await
        {
            Ok(events) => {
                summary.calendar_events = events.len();
                state
                    .db
                    .save_calendar_cache(SCHEDULED_CALENDAR_CACHE_KEY, &events, None)
                    .map_err(|e| e.to_string())?;
                log::info!(
                    "scheduled data fetch: calendar {} events cached",
                    summary.calendar_events
                );
            }
            Err(e) => {
                log::warn!("scheduled calendar fetch failed: {e}");
            }
        }

        let jinshi = state.jinshi.lock().await;
        if jinshi.is_connected() {
            let llm_snap = state.llm_snapshot();
            let deps = IngestDeps {
                jinshi: &jinshi,
                db: &state.db,
                llm: Some(&llm_snap),
                classify_cfg: &news_classify,
                default_llm_provider: &default_llm,
            };
            match ingest_poll(&deps, 20).await {
                Ok((items, labels)) => {
                    summary.news_items = items;
                    summary.news_labels = labels;
                }
                Err(e) => log::warn!("scheduled news ingest failed: {e}"),
            }
        }
    }

    let symbols = crate::engine::sectors::core_product_symbols();
    if state.akshare_ready && !symbols.is_empty() {
        let end = Utc::now();
        let start = end - Duration::days(10);
        for sym in &symbols {
            match state.akshare.get_history(sym, "1d", start, end).await {
                Ok(klines) if !klines.is_empty() => {
                    let _ = state.db.save_klines(&klines);
                    summary.klines_symbols += 1;
                }
                Ok(_) => {}
                Err(e) => log::debug!("scheduled kline refresh {sym}: {e}"),
            }
        }
    }

    Ok(summary)
}
