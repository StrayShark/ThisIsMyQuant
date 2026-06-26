//! 数据 retention 定时清理。

use std::sync::Arc;
use std::time::Duration;

use crate::state::AppState;

pub fn spawn_data_maintenance(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let (k_days, t_days) = {
                let cfg = state.config();
                (
                    cfg.retention_days_klines.max(30),
                    cfg.retention_days_ticks.max(7),
                )
            };
            if let Ok(k) = state.db.purge_old_klines(k_days) {
                if k > 0 {
                    log::info!("retention: purged {k} klines older than {k_days}d");
                }
            }
            if let Ok(t) = state.db.purge_old_ticks(t_days) {
                if t > 0 {
                    log::info!("retention: purged {t} ticks older than {t_days}d");
                }
            }
        }
    });
}
