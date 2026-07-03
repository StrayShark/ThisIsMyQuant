//! 统一定时任务：每 N 小时拉取金融数据 + 全部 core 品种全面分析。

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::engine::sectors;
use crate::models::{dt_to_iso, ScheduleStatus};
use crate::services::analysis_runner::run_analysis;
use crate::services::data_fetch_cycle::run_data_fetch_cycle;
use crate::state::AppState;

pub type ScheduleStatusHandle = Arc<Mutex<ScheduleStatus>>;

pub fn new_schedule_status(interval_hours: u64, enabled: bool) -> ScheduleStatusHandle {
    Arc::new(Mutex::new(ScheduleStatus {
        enabled,
        interval_hours,
        ..Default::default()
    }))
}

pub struct ScheduleHandle {
    pub interval_hours: u64,
    pub enabled: bool,
    status: ScheduleStatusHandle,
    _task: Option<JoinHandle<()>>,
}

impl ScheduleHandle {
    pub fn start(
        app: AppHandle,
        state: Arc<AppState>,
        config: &Config,
        status: ScheduleStatusHandle,
    ) -> Self {
        let hours = config.schedule_interval_hours.max(1);
        let enabled = config.schedule_enabled;

        let app_c = app.clone();
        let state_c = state.clone();
        let status_c = status.clone();
        let task = Some(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(90)).await;
            loop {
                let (cycle_enabled, interval_hours) = {
                    let cfg = state_c.config();
                    (cfg.schedule_enabled, cfg.schedule_interval_hours.max(1))
                };
                {
                    let mut st = status_c.lock().await;
                    st.enabled = cycle_enabled;
                    st.interval_hours = interval_hours;
                }
                if cycle_enabled {
                    let trigger = {
                        let cfg = state_c.config();
                        cfg.schedule_analysis_trigger.clone()
                    };
                    if let Err(e) =
                        run_full_cycle(&state_c, Some(&app_c), &trigger, status_c.clone()).await
                    {
                        log::warn!("scheduled cycle failed: {e}");
                    }
                }
                let sleep_hours = {
                    let cfg = state_c.config();
                    cfg.schedule_interval_hours.max(1)
                };
                tokio::time::sleep(Duration::from_secs(sleep_hours * 3600)).await;
            }
        }));

        if enabled {
            log::info!(
                "ScheduleRunner enabled: every {hours}h (data fetch + comprehensive analysis)"
            );
        } else {
            log::info!("ScheduleRunner idle (disabled); config changes apply without restart");
        }

        Self {
            interval_hours: hours,
            enabled,
            status,
            _task: task,
        }
    }

    pub fn status_handle(&self) -> ScheduleStatusHandle {
        self.status.clone()
    }
}

pub async fn run_full_cycle(
    state: &Arc<AppState>,
    app: Option<&AppHandle>,
    analysis_trigger: &str,
    status: ScheduleStatusHandle,
) -> Result<(), String> {
    {
        let mut st = status.lock().await;
        if st.cycle_in_progress {
            return Err("schedule cycle already running".into());
        }
        st.cycle_in_progress = true;
        st.last_error = None;
    }

    let result: Result<(), String> = async {
        let fetch = run_data_fetch_cycle(state).await?;
        {
            let mut st = status.lock().await;
            st.last_data_fetch = Some(fetch);
        }

        let (completed, total, errors) =
            run_comprehensive_analysis(state, app, analysis_trigger).await;

        {
            let mut st = status.lock().await;
            st.last_analysis_completed = completed;
            st.last_analysis_total = total;
            st.last_cycle_at = Some(dt_to_iso(Utc::now()));
            if !errors.is_empty() {
                st.last_error = Some(errors.join("; "));
            }
        }
        Ok(())
    }
    .await;

    let mut st = status.lock().await;
    st.cycle_in_progress = false;
    if let Err(ref e) = result {
        st.last_error = Some(e.clone());
    }
    result
}

pub async fn run_comprehensive_analysis(
    state: &Arc<AppState>,
    app: Option<&AppHandle>,
    trigger: &str,
) -> (usize, usize, Vec<String>) {
    let symbols = sectors::core_product_symbols();
    let total = symbols.len();
    let mut completed = 0usize;
    let mut errors = Vec::new();

    if state.llm_snapshot().available_providers().is_empty() {
        errors.push("no LLM provider configured".into());
        return (0, total, errors);
    }

    for sym in symbols {
        match run_analysis(state, app, &sym, trigger, None, false, None).await {
            Ok(_) => completed += 1,
            Err(e) => errors.push(format!("{sym}: {e}")),
        }
    }

    log::info!("comprehensive analysis ({trigger}): {completed}/{total} ok");
    (completed, total, errors)
}
