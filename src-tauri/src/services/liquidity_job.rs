//! 启动后异步刷新各品种流动性快照。

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::engine::{liquidity, sectors};
use crate::error::AppResult;
use crate::state::AppState;

pub struct LiquidityJobHandle;

impl LiquidityJobHandle {
    pub fn spawn(state: Arc<AppState>) -> Self {
        tauri::async_runtime::spawn(async move {
            if let Err(e) = refresh_all(&state).await {
                log::warn!("liquidity refresh failed: {e}");
            }
        });
        Self
    }
}

async fn refresh_all(state: &AppState) -> AppResult<()> {
    if !state.akshare_ready {
        log::info!("liquidity refresh skipped: akshare unavailable");
        return Ok(());
    }

    let cfg = state.config().liquidity.clone();
    let end = Utc::now();
    let start = end - Duration::days(35);
    let mut updated = 0usize;

    for product in sectors::all_products() {
        let sym = product.symbol.to_lowercase();
        let mut klines = state
            .db
            .get_klines(&sym, "1d", start, end, 30)
            .unwrap_or_default();

        if klines.is_empty() {
            if let Ok(fetched) = state
                .akshare
                .get_history(&sym, "1d", start, end)
                .await
            {
                klines = fetched;
                if !klines.is_empty() {
                    let _ = state.db.save_klines(&klines);
                }
            }
        }

        let snap = liquidity::build_snapshot(
            &product.symbol,
            &klines,
            product.default_tier,
            &cfg,
        );
        state.db.save_liquidity_snapshot(&snap)?;
        updated += 1;
    }

    log::info!("liquidity refresh done: {updated} symbols");
    Ok(())
}
