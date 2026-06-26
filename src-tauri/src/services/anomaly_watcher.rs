//! 异动触发：检测后自动发起 anomaly 分析并推送通知。

use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::engine::anomaly::AnomalyDetector;
use crate::models::NotificationEvent;
use crate::services::run_analysis;
use crate::state::AppState;

pub struct AnomalyWatcher {
    detector: Arc<AnomalyDetector>,
}

impl AnomalyWatcher {
    pub fn new(detector: Arc<AnomalyDetector>) -> Self {
        Self { detector }
    }

    pub fn detector(&self) -> &Arc<AnomalyDetector> {
        &self.detector
    }

    pub fn update_config(&self, cfg: crate::engine::anomaly::AnomalyConfig) {
        self.detector.update_config(cfg);
    }

    pub fn on_tick(&self, state: &Arc<AppState>, app: &AppHandle, tick: &crate::models::Tick) {
        let Some(reason) = self.detector.on_tick(tick) else {
            return;
        };
        if state.llm_snapshot().available_providers().is_empty() {
            log::warn!("anomaly detected for {} but no LLM configured", tick.symbol);
            return;
        }

        let sym = tick.symbol.clone();
        let reason_owned = reason.clone();
        let st = Arc::clone(state);
        let app = app.clone();

        log::info!("anomaly trigger {sym}: {reason_owned}");
        let _ = app.emit(
            "notification",
            NotificationEvent {
                msg_type: "notification".into(),
                level: "warning".into(),
                title: format!("{sym} 异动触发"),
                body: reason_owned.clone(),
                link: Some("/reports".into()),
            },
        );

        tauri::async_runtime::spawn(async move {
            match run_analysis(
                &st,
                Some(&app),
                &sym,
                "anomaly",
                None,
                false,
                Some(&reason_owned),
            )
            .await
            {
                Ok(report) => {
                    let _ = app.emit(
                        "notification",
                        NotificationEvent {
                            msg_type: "notification".into(),
                            level: "info".into(),
                            title: format!("{} 异动分析报告", report.symbol),
                            body: report.context_summary.clone(),
                            link: Some(format!("/reports/{}", report.id)),
                        },
                    );
                }
                Err(e) => log::warn!("anomaly analysis failed for {sym}: {e}"),
            }
        });
    }
}
