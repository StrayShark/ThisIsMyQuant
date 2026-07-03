use std::sync::Arc;

use tauri::State;

use crate::models::{
    AkshareHealth, ApiResponse, HealthResponse, JinshiHealth, NewsPollStatus, PollStatus,
    RealtimeHealth,
};
use crate::state::AppState;

#[tauri::command]
pub async fn get_health(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<HealthResponse>, String> {
    let llm_health = state.llm_snapshot().health().await;
    let mut feeds = std::collections::HashMap::new();
    feeds.insert("akshare".into(), state.akshare_ready);
    let poll_status = to_poll_status(&state).await;
    let poll_running = poll_status.as_ref().map(|p| p.running).unwrap_or(false);
    let (cal_ready, cal_at, cal_count) = {
        let j = state.jinshi.lock().await;
        j.calendar_status()
    };
    let jinshi_connected = state.jinshi.lock().await.is_connected();
    let news_poll_running = state.news_poll.lock().await.is_some();
    Ok(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        feeds,
        llm: llm_health,
        db: true,
        akshare: AkshareHealth {
            history: state.akshare_ready,
        },
        poll: poll_status,
        news_poll: news_poll_running.then(|| NewsPollStatus {
            running: true,
            interval: state.config().jinshi_poll_interval,
        }),
        realtime: RealtimeHealth {
            available: poll_running,
            source: if poll_running {
                Some(state.feed_source.clone())
            } else {
                None
            },
        },
        jinshi: JinshiHealth {
            enabled: state.config().jinshi_enabled,
            connected: jinshi_connected,
            calendar_ready: cal_ready,
            calendar_fetched_at: cal_at,
            calendar_cached_events: cal_count,
        },
        realtime_enabled: state.config().akshare_enabled && state.config().akshare_realtime_enabled,
        llm_last_errors: state.llm_snapshot().last_errors(),
    }))
}

#[tauri::command]
pub async fn get_runtime_status(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::RuntimeStatusView>, String> {
    let poll = to_poll_status(&state).await;
    let backfill = state.backfill_status.lock().await.clone();
    let schedule = state.schedule_status.lock().await.clone();
    Ok(ApiResponse::ok(crate::models::RuntimeStatusView {
        poll,
        backfill,
        feed_source: if state.feed_source.is_empty() {
            None
        } else {
            Some(state.feed_source.clone())
        },
        schedule,
    }))
}

#[tauri::command]
pub async fn get_status_dashboard(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::StatusDashboardView>, String> {
    let llm_health = state.llm_snapshot().health().await;
    let questdb = crate::db::QuestDbAdapter::new(&state.config().questdb_url);
    let poll = to_poll_status(&state).await;
    let backfill = state.backfill_status.lock().await.clone();
    let schedule = state.schedule_status.lock().await.clone();
    let stale_after_secs = {
        let cfg = state.config();
        (cfg.realtime_poll_interval * 3.0).ceil() as i64
    };
    let quote_status = {
        let cache = state.quote_cache.read().unwrap_or_else(|e| e.into_inner());
        cache.status(stale_after_secs)
    };
    let llm_last_errors = state.llm_snapshot().last_errors();
    let batch_job = state.batch_analysis.get_status().await;
    let runtime = crate::models::RuntimeStatusView {
        poll,
        backfill,
        feed_source: if state.feed_source.is_empty() {
            None
        } else {
            Some(state.feed_source.clone())
        },
        schedule,
    };
    Ok(ApiResponse::ok(crate::models::StatusDashboardView {
        runtime,
        quote_status,
        llm_health,
        llm_last_errors,
        questdb_configured: questdb.configured(),
        questdb_online: if questdb.configured() {
            questdb.ping().await
        } else {
            false
        },
        overseas: crate::adapters::list_overseas_symbols(),
        batch_job,
        prompt_version: crate::engine::PROMPT_VERSION.to_string(),
    }))
}

#[tauri::command]
pub async fn get_schedule_status(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::ScheduleStatus>, String> {
    Ok(ApiResponse::ok(state.schedule_status.lock().await.clone()))
}

#[tauri::command]
pub async fn trigger_data_fetch(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::DataFetchSummary>, String> {
    let summary = crate::services::run_data_fetch_cycle(state.inner()).await?;
    {
        let mut st = state.schedule_status.lock().await;
        st.last_data_fetch = Some(summary.clone());
    }
    Ok(ApiResponse::ok(summary))
}

pub(crate) async fn to_poll_status(state: &State<'_, Arc<AppState>>) -> Option<PollStatus> {
    if let Some(p) = state.poll_handle().await {
        let s = p.status().await;
        Some(PollStatus {
            running: s.running,
            interval: s.interval,
            symbols: s.symbols,
            symbol_count: s.symbol_count,
            feed_source: s.feed_source,
        })
    } else {
        None
    }
}
