//! 每日收盘简报：在配置的小时点对 watchlist 跑「明日展望」分析。

use std::sync::Arc;

use chrono::{Datelike, Local, Timelike};
use tauri::AppHandle;
use tokio::time::{sleep, Duration};

use crate::services::schedule_runner::{run_full_cycle, ScheduleStatusHandle};
use crate::state::AppState;

pub fn spawn_daily_briefing(app: AppHandle, state: Arc<AppState>, status: ScheduleStatusHandle) {
    tokio::spawn(async move {
        let mut last_run_ordinal: Option<u32> = None;
        loop {
            sleep(Duration::from_secs(60)).await;
            let (enabled, hour, watchlist_empty, has_llm) = {
                let cfg = state.config();
                (
                    cfg.daily_briefing_enabled,
                    cfg.daily_briefing_hour,
                    cfg.watchlist.is_empty(),
                    !state.llm_snapshot().available_providers().is_empty(),
                )
            };
            if !enabled || watchlist_empty || !has_llm {
                continue;
            }
            let now = Local::now();
            if now.hour() as u8 != hour {
                continue;
            }
            let ordinal = now.ordinal();
            if last_run_ordinal == Some(ordinal) {
                continue;
            }
            last_run_ordinal = Some(ordinal);
            log::info!("DailyBriefing: running tomorrow analysis at {}:00", hour);
            if let Err(e) = run_full_cycle(&state, Some(&app), "tomorrow", status.clone()).await {
                log::warn!("DailyBriefing failed: {e}");
            }
        }
    });
}
