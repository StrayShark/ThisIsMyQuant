//! 品种基本面快照（持仓/基差/仓单占位），供 prompt v4 注入。

use serde_json::{json, Value};

use crate::adapters::AkshareClient;

pub async fn fetch_fundamentals(akshare: &AkshareClient, symbol: &str) -> Value {
    match akshare.fetch_market_fundamentals(symbol).await {
        Ok(v) => v,
        Err(e) => {
            log::debug!("fundamentals fetch failed for {symbol}: {e}");
            json!({
                "symbol": symbol,
                "source": "unavailable",
                "open_interest": null,
                "warehouse": null,
                "basis": null,
                "note": "基本面数据暂不可用"
            })
        }
    }
}
