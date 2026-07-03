use std::sync::Arc;

use tauri::State;

use crate::models::{AnalysisReport, ApiResponse, FollowupMessage};
use crate::state::AppState;

#[tauri::command]
pub async fn list_reports(
    state: State<'_, Arc<AppState>>,
    symbol: Option<String>,
    trigger: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<AnalysisReport>>, String> {
    match state.db.get_reports(
        symbol.as_deref(),
        trigger.as_deref(),
        limit.unwrap_or(50).min(200),
    ) {
        Ok(reports) => Ok(ApiResponse::ok(reports)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_report(
    state: State<'_, Arc<AppState>>,
    report_id: String,
) -> Result<ApiResponse<AnalysisReport>, String> {
    match state.db.get_report(&report_id) {
        Ok(Some(r)) => Ok(ApiResponse::ok(r)),
        Ok(None) => Ok(ApiResponse::err("report not found")),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_followups(
    state: State<'_, Arc<AppState>>,
    report_id: Option<String>,
    symbol: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<FollowupMessage>>, String> {
    match state.db.get_followups(
        report_id.as_deref(),
        symbol.as_deref(),
        limit.unwrap_or(50).min(200),
    ) {
        Ok(items) => Ok(ApiResponse::ok(items)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn export_reports_csv(
    state: State<'_, Arc<AppState>>,
    symbol: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<String>, String> {
    let reports = state
        .db
        .get_reports(symbol.as_deref(), None, limit.unwrap_or(200).min(500))
        .map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(crate::services::reports_to_csv(&reports)))
}
