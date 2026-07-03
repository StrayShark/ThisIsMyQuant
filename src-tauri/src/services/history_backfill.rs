//! 历史 K 线回填：启动时对全部 core 品种批量拉取并落库。

use std::sync::Arc;

use chrono::{Duration, Utc};
use tokio::sync::Mutex;

use crate::models::BackfillStatus;
use crate::state::AppState;

pub type BackfillStatusHandle = Arc<Mutex<BackfillStatus>>;

pub fn new_status_handle() -> BackfillStatusHandle {
    Arc::new(Mutex::new(BackfillStatus::default()))
}

pub fn spawn_history_backfill(state: Arc<AppState>, status: BackfillStatusHandle) {
    if !state.config().akshare_enabled {
        return;
    }
    let symbols = crate::engine::sectors::core_product_symbols();
    if symbols.is_empty() {
        return;
    }

    let days_daily = state.config().backfill_days_daily.max(30);
    let days_minute = state.config().backfill_days_minute.max(1);

    tokio::spawn(async move {
        {
            let mut st = status.lock().await;
            st.running = true;
            st.total = symbols.len();
            st.completed = 0;
        }

        let end = Utc::now();
        for sym in symbols {
            {
                let mut st = status.lock().await;
                st.current_symbol = Some(sym.clone());
            }

            let daily_start = end - Duration::days(days_daily);
            match state
                .akshare
                .get_history(&sym, "1d", daily_start, end)
                .await
            {
                Ok(klines) => {
                    let _ = state.db.save_klines(&klines);
                }
                Err(e) => {
                    let mut st = status.lock().await;
                    st.last_error = Some(format!("{sym} 1d: {e}"));
                }
            }

            let minute_start = end - Duration::days(days_minute);
            match state
                .akshare
                .get_history(&sym, "1m", minute_start, end)
                .await
            {
                Ok(klines) => {
                    let _ = state.db.save_klines(&klines);
                }
                Err(e) => {
                    let mut st = status.lock().await;
                    st.last_error = Some(format!("{sym} 1m: {e}"));
                }
            }

            let mut st = status.lock().await;
            st.completed += 1;
        }

        let mut st = status.lock().await;
        st.running = false;
        st.current_symbol = None;
        log::info!(
            "history backfill done: {}/{} symbols",
            st.completed,
            st.total
        );
    });
}
