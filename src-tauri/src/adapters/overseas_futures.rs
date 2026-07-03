//! 海外期货参考行情：Yahoo Finance 免费连续合约（日内延迟，仅作联动分析参考）。

use chrono::Utc;
use serde_json::{json, Value};

#[derive(Clone, Copy)]
struct OverseasSymbol {
    symbol: &'static str,
    name: &'static str,
    exchange: &'static str,
    sector: &'static str,
}

const SYMBOLS: &[OverseasSymbol] = &[
    OverseasSymbol {
        symbol: "CL=F",
        name: "WTI 原油",
        exchange: "NYMEX",
        sector: "energy_chemical",
    },
    OverseasSymbol {
        symbol: "BZ=F",
        name: "Brent 原油",
        exchange: "ICE",
        sector: "energy_chemical",
    },
    OverseasSymbol {
        symbol: "NG=F",
        name: "Henry Hub 天然气",
        exchange: "NYMEX",
        sector: "energy_chemical",
    },
    OverseasSymbol {
        symbol: "GC=F",
        name: "COMEX 黄金",
        exchange: "COMEX",
        sector: "metals",
    },
    OverseasSymbol {
        symbol: "SI=F",
        name: "COMEX 白银",
        exchange: "COMEX",
        sector: "metals",
    },
    OverseasSymbol {
        symbol: "HG=F",
        name: "COMEX 铜",
        exchange: "COMEX",
        sector: "metals",
    },
    OverseasSymbol {
        symbol: "ZC=F",
        name: "CBOT 玉米",
        exchange: "CBOT",
        sector: "agriculture",
    },
    OverseasSymbol {
        symbol: "ZS=F",
        name: "CBOT 大豆",
        exchange: "CBOT",
        sector: "agriculture",
    },
    OverseasSymbol {
        symbol: "ZM=F",
        name: "CBOT 豆粕",
        exchange: "CBOT",
        sector: "agriculture",
    },
];

pub fn list_overseas_symbols() -> Value {
    json!({
        "status": "ok",
        "source": "yahoo_finance_chart",
        "message": "Yahoo Finance 海外期货参考源（日内延迟，仅用于内外盘联动）",
        "symbols": SYMBOLS.iter().map(|s| json!({
            "symbol": s.symbol,
            "name": s.name,
            "exchange": s.exchange,
            "sector": s.sector,
        })).collect::<Vec<_>>()
    })
}

pub async fn fetch_overseas_quote(symbol: &str) -> Value {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return json!({ "status": "error", "message": "empty symbol", "quote": null });
    }

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=5d&interval=1d",
        urlencoding::encode(symbol)
    );
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)")
        .build();
    let Ok(client) = client else {
        return json!({ "status": "error", "message": "http client init failed", "quote": null });
    };

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(raw) => parse_yahoo_quote(symbol, &raw),
            Err(e) => json!({ "status": "error", "message": e.to_string(), "quote": null }),
        },
        Err(e) => json!({ "status": "error", "message": e.to_string(), "quote": null }),
    }
}

fn parse_yahoo_quote(symbol: &str, raw: &Value) -> Value {
    let Some(result) = raw["chart"]["result"].as_array().and_then(|a| a.first()) else {
        return json!({ "status": "error", "message": "missing chart result", "quote": null });
    };
    let quote = &result["indicators"]["quote"][0];
    let closes = quote["close"].as_array().cloned().unwrap_or_default();
    let volumes = quote["volume"].as_array().cloned().unwrap_or_default();
    let timestamps = result["timestamp"].as_array().cloned().unwrap_or_default();

    let mut latest_idx = None;
    for (idx, close) in closes.iter().enumerate().rev() {
        if close.as_f64().is_some_and(|v| v.is_finite() && v > 0.0) {
            latest_idx = Some(idx);
            break;
        }
    }
    let Some(i) = latest_idx else {
        return json!({ "status": "error", "message": "missing latest close", "quote": null });
    };
    let last = closes[i].as_f64().unwrap_or_default();
    let prev = closes[..i]
        .iter()
        .rev()
        .find_map(|v| v.as_f64())
        .unwrap_or(last);
    let change_pct = if prev > 0.0 {
        (last - prev) / prev * 100.0
    } else {
        0.0
    };
    let ts = timestamps
        .get(i)
        .and_then(|v| v.as_i64())
        .and_then(|s| chrono::DateTime::from_timestamp(s, 0))
        .unwrap_or_else(Utc::now);
    let volume = volumes.get(i).and_then(|v| v.as_i64()).unwrap_or(0);

    json!({
        "status": "ok",
        "source": "yahoo_finance_chart",
        "quote": {
            "symbol": symbol,
            "last_price": last,
            "prev_close": prev,
            "change_pct": change_pct,
            "volume": volume,
            "timestamp": ts.to_rfc3339(),
        }
    })
}
