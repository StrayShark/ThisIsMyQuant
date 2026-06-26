//! 海外期货行情占位（H3）：返回空列表，预留扩展。

use serde_json::{json, Value};

pub fn list_overseas_symbols() -> Value {
    json!({
        "status": "stub",
        "message": "海外期货接口尚未接入，可后续对接 CME/NYMEX 等数据源",
        "symbols": []
    })
}

pub async fn fetch_overseas_quote(_symbol: &str) -> Value {
    json!({
        "status": "stub",
        "quote": null
    })
}
