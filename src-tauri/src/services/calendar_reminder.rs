//! 日历事件临近提醒（★3+）。

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter};

use crate::models::NotificationEvent;
use crate::state::AppState;

pub fn spawn_calendar_reminder(app: AppHandle, state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut seen: HashSet<String> = HashSet::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let (reminder_enabled, ahead) = {
                let cfg = state.config();
                (cfg.calendar_reminder_enabled, cfg.calendar_reminder_mins.max(5))
            };
            if !reminder_enabled {
                continue;
            }

            let jinshi = state.jinshi.lock().await;
            if !jinshi.is_connected() {
                continue;
            }
            let now = Utc::now();
            let end = now + Duration::minutes(ahead as i64);
            let (start_date, _) = crate::adapters::default_calendar_range_from_today();
            let Ok(events) = jinshi
                .fetch_calendar_events(start_date, end.date_naive(), 3, None)
                .await
            else {
                continue;
            };
            drop(jinshi);

            for ev in events {
                let Some(pub_dt) = crate::models::parse_dt(&ev.pub_time) else {
                    continue;
                };
                let delta = pub_dt - now;
                if delta.num_minutes() < 0 || delta.num_minutes() > ahead as i64 {
                    continue;
                }
                if !seen.insert(ev.id.clone()) {
                    continue;
                }
                let _ = app.emit(
                    "notification",
                    NotificationEvent {
                        msg_type: "notification".into(),
                        level: "info".into(),
                        title: format!("日历提醒 · {}", ev.name),
                        body: format!(
                            "{} {} · ★{} · {} 分钟后",
                            ev.country,
                            ev.pub_time,
                            ev.star,
                            delta.num_minutes().max(0)
                        ),
                        link: Some("/".into()),
                    },
                );
            }
        }
    });
}
