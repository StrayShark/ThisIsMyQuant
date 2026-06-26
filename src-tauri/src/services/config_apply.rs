//! 将 UserPreferences / Config 变更应用到运行时（行情订阅、异动、轮询重启等）。

use std::sync::Arc;

use tauri::AppHandle;

use crate::config::{Config, UserPreferences};
use crate::engine::anomaly::AnomalyConfig;
use crate::state::AppState;

pub async fn apply_runtime_config(state: &Arc<AppState>, cfg: Config) {
    let old_watchlist = state.config().watchlist.clone();

    {
        let mut w = state.config_store.write().unwrap_or_else(|e| e.into_inner());
        *w = cfg.clone();
    }

    state.anomaly.update_config(AnomalyConfig {
        enabled: cfg.anomaly_enabled,
        price_pct_threshold: cfg.anomaly_price_pct,
        window_secs: cfg.anomaly_window_secs,
        cooldown_secs: cfg.anomaly_cooldown_secs,
    });

    if let Some(poll) = state.poll_handle().await {
        let new_set: std::collections::HashSet<_> =
            cfg.watchlist.iter().map(|s| s.to_lowercase()).collect();
        let old_set: std::collections::HashSet<_> =
            old_watchlist.iter().map(|s| s.to_lowercase()).collect();
        let to_add: Vec<_> = new_set.difference(&old_set).cloned().collect();
        let to_remove: Vec<_> = old_set.difference(&new_set).cloned().collect();
        if !to_add.is_empty() {
            poll.subscribe(to_add).await;
        }
        if !to_remove.is_empty() {
            poll.unsubscribe(to_remove).await;
        }
    }

    {
        let mut st = state.schedule_status.lock().await;
        st.enabled = cfg.schedule_enabled && !cfg.watchlist.is_empty();
        st.interval_hours = cfg.schedule_interval_hours.max(1);
    }
}

pub async fn apply_preferences(
    app: &AppHandle,
    state: &Arc<AppState>,
    prefs: UserPreferences,
) -> Config {
    let prefs = prefs.normalize();
    let mut cfg = state.config().clone();
    prefs.apply_to(&mut cfg);
    apply_runtime_config(state, cfg.clone()).await;
    crate::services::restart_runtime_polls(app, state).await;
    cfg
}
