use std::sync::Arc;

use chrono::Utc;
use tauri::State;

use crate::models::{
    dt_to_iso, ApiResponse, MarketEvent, SaveWatchlistGroupRequest, SaveWatchlistItemRequest,
    WatchlistGroup, WatchlistItem, WatchlistSummary,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_watchlist_groups(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<WatchlistGroup>>, String> {
    match state.db.list_watchlist_groups() {
        Ok(groups) => Ok(ApiResponse::ok(groups)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn save_watchlist_group(
    state: State<'_, Arc<AppState>>,
    req: SaveWatchlistGroupRequest,
) -> Result<ApiResponse<WatchlistGroup>, String> {
    let now = dt_to_iso(Utc::now());
    let id = req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let group = WatchlistGroup {
        id,
        name: req.name,
        sort_order: req.sort_order.unwrap_or(0),
        created_at: now.clone(),
        updated_at: now,
    };
    match state.db.save_watchlist_group(&group) {
        Ok(_) => Ok(ApiResponse::ok(group)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn delete_watchlist_group(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ApiResponse<()>, String> {
    match state.db.delete_watchlist_group(&id) {
        Ok(_) => Ok(ApiResponse::ok(())),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_watchlist_items(
    state: State<'_, Arc<AppState>>,
    group_id: Option<String>,
) -> Result<ApiResponse<Vec<WatchlistItem>>, String> {
    match state.db.list_watchlist_items(group_id.as_deref()) {
        Ok(items) => Ok(ApiResponse::ok(items)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn save_watchlist_item(
    state: State<'_, Arc<AppState>>,
    req: SaveWatchlistItemRequest,
) -> Result<ApiResponse<WatchlistItem>, String> {
    let now = dt_to_iso(Utc::now());
    let id = req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let item = WatchlistItem {
        id,
        group_id: req.group_id,
        asset_type: req.asset_type,
        symbol: req.symbol,
        name: req.name,
        notes: req.notes,
        alert_price: req.alert_price,
        alert_pct: req.alert_pct,
        sort_order: req.sort_order.unwrap_or(0),
        created_at: now.clone(),
        updated_at: now,
    };
    match state.db.save_watchlist_item(&item) {
        Ok(_) => Ok(ApiResponse::ok(item)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn delete_watchlist_item(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ApiResponse<()>, String> {
    match state.db.delete_watchlist_item(&id) {
        Ok(_) => Ok(ApiResponse::ok(())),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_watchlist_summary(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<WatchlistSummary>, String> {
    match state.db.get_watchlist_summary() {
        Ok(summary) => Ok(ApiResponse::ok(summary)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_watchlist_events(
    _state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<MarketEvent>>, String> {
    // P1 阶段从 calendar / news 聚合；P0 先返回空数组占位
    Ok(ApiResponse::ok(vec![]))
}
