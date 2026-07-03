use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::State;

use crate::engine::sectors::{self, SectorView};
use crate::models::{ApiResponse, Contract, KLine};
use crate::state::AppState;

#[tauri::command]
pub async fn list_products(
    state: State<'_, Arc<AppState>>,
    tier: Option<String>,
) -> Result<ApiResponse<Vec<SectorView>>, String> {
    let filter = tier.unwrap_or_else(|| "core".into());
    let liquidity = state.db.get_latest_liquidity_map().unwrap_or_default();
    Ok(ApiResponse::ok(sectors::build_catalog(&filter, &liquidity)))
}

#[tauri::command]
pub async fn list_contracts(
    state: State<'_, Arc<AppState>>,
    exchange: Option<String>,
) -> Result<ApiResponse<Vec<Contract>>, String> {
    let result = match state.db.get_contracts(exchange.as_deref()) {
        Ok(list) if !list.is_empty() => Ok(list),
        _ => state.akshare.get_contracts().await.inspect(|contracts| {
            let _ = state.db.save_contracts(contracts);
        }),
    };
    match result {
        Ok(contracts) => Ok(ApiResponse::ok(contracts)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_klines(
    state: State<'_, Arc<AppState>>,
    symbol: String,
    interval: String,
    start: Option<String>,
    end: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<KLine>>, String> {
    let end_dt = end
        .and_then(|s| crate::models::parse_dt(&s))
        .unwrap_or_else(Utc::now);
    let start_dt = start
        .and_then(|s| crate::models::parse_dt(&s))
        .unwrap_or_else(|| end_dt - Duration::days(120));
    let limit = limit.unwrap_or(1000).min(10000);
    let sym = symbol.to_lowercase();

    let mut klines = state
        .db
        .get_klines(&sym, &interval, start_dt, end_dt, limit)
        .unwrap_or_default();

    let needs_fetch =
        klines.is_empty() || (interval == "1d" && crate::services::is_daily_klines_stale(&klines));

    if needs_fetch {
        match state
            .akshare
            .get_history(&sym, &interval, start_dt, end_dt)
            .await
        {
            Ok(mut fetched) if !fetched.is_empty() => {
                if fetched.len() as i64 > limit {
                    fetched = fetched.split_off(fetched.len() - limit as usize);
                }
                let _ = state.db.save_klines(&fetched);
                klines = fetched;
            }
            Ok(_) if klines.is_empty() => {
                return Ok(ApiResponse::err(format!("no kline data for {sym}")));
            }
            Err(e) if klines.is_empty() => return Ok(ApiResponse::err(e.to_string())),
            Err(e) => log::debug!("kline refresh {sym}: {e}"),
            _ => {}
        }
    }

    if interval == "1d" {
        if let Some(forming) = state
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .forming_daily(&sym)
        {
            crate::services::merge_forming_daily(&mut klines, &forming);
        }
    }

    Ok(ApiResponse::ok(klines))
}

#[tauri::command]
pub async fn market_subscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = state.poll_handle().await {
        poll.subscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(
            serde_json::json!({ "subscribed": symbols }),
        ))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}

#[tauri::command]
pub async fn market_unsubscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = state.poll_handle().await {
        poll.unsubscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(
            serde_json::json!({ "unsubscribed": symbols }),
        ))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}

#[tauri::command]
pub async fn get_realtime_quotes(
    state: State<'_, Arc<AppState>>,
    symbols: Option<Vec<String>>,
) -> Result<ApiResponse<Vec<crate::models::RealtimeQuote>>, String> {
    let list = state
        .quote_cache
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot(symbols.as_deref());
    Ok(ApiResponse::ok(list))
}

#[tauri::command]
pub async fn get_symbol_context(symbol: String) -> Result<ApiResponse<serde_json::Value>, String> {
    Ok(ApiResponse::ok(sectors::sector_context(&symbol)))
}
