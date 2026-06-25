use std::sync::Arc;
use std::time::Duration;

use chrono::{FixedOffset, TimeZone, Utc};
use tauri::AppHandle;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::services::analysis_runner::run_analysis;
use crate::state::AppState;

pub struct AnalysisSchedulerHandle {
    pub daily_running: bool,
    pub realtime_running: bool,
    _daily: Option<JoinHandle<()>>,
    _realtime: Option<JoinHandle<()>>,
}

impl AnalysisSchedulerHandle {
    pub fn start(app: AppHandle, state: Arc<AppState>, config: &Config) -> Self {
        let watchlist = config.watchlist.clone();
        let daily_cron = config.daily_analysis_cron.clone();
        let realtime_secs = config.realtime_analysis_interval;

        let daily_running = !watchlist.is_empty() && !daily_cron.is_empty();
        let realtime_running = !watchlist.is_empty() && realtime_secs > 0;

        let daily = if daily_running {
            let (minute, hour) = parse_cron(&daily_cron);
            let app_d = app.clone();
            let state_d = state.clone();
            let wl = watchlist.clone();
            Some(tokio::spawn(async move {
                loop {
                    let wait = secs_until_shanghai(hour, minute);
                    tokio::time::sleep(wait).await;
                    for sym in &wl {
                        if let Err(e) = run_analysis(
                            &state_d,
                            Some(&app_d),
                            sym,
                            "daily",
                            None,
                            false,
                        )
                        .await
                        {
                            log::warn!("scheduled daily analysis failed {sym}: {e}");
                        }
                    }
                }
            }))
        } else {
            None
        };

        let realtime = if realtime_running {
            let app_r = app.clone();
            let state_r = state.clone();
            let wl = watchlist.clone();
            let interval = Duration::from_secs(realtime_secs.max(60));
            Some(tokio::spawn(async move {
                loop {
                    tokio::time::sleep(interval).await;
                    for sym in &wl {
                        if let Err(e) = run_analysis(
                            &state_r,
                            Some(&app_r),
                            sym,
                            "realtime",
                            None,
                            false,
                        )
                        .await
                        {
                            log::warn!("scheduled realtime analysis failed {sym}: {e}");
                        }
                    }
                }
            }))
        } else {
            None
        };

        if daily_running {
            log::info!("AnalysisScheduler daily cron={daily_cron}");
        }
        if realtime_running {
            log::info!("AnalysisScheduler realtime every {realtime_secs}s");
        }

        Self {
            daily_running,
            realtime_running,
            _daily: daily,
            _realtime: realtime,
        }
    }
}

fn parse_cron(raw: &str) -> (u32, u32) {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() >= 2 {
        let minute = parts[0].parse().unwrap_or(0);
        let hour = parts[1].parse().unwrap_or(17);
        (minute, hour)
    } else {
        (0, 17)
    }
}

fn secs_until_shanghai(hour: u32, minute: u32) -> Duration {
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let now = Utc::now().with_timezone(&tz);
    let today = now.date_naive();
    let mut target = tz
        .from_local_datetime(
            &today
                .and_hms_opt(hour, minute, 0)
                .unwrap_or_else(|| today.and_hms_opt(17, 0, 0).unwrap()),
        )
        .unwrap();
    if target <= now {
        target += chrono::Duration::days(1);
    }
    target
        .signed_duration_since(now)
        .to_std()
        .unwrap_or(Duration::from_secs(3600))
}
