use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::engine::{dimensions, sectors};
use crate::models::{
    AnalysisDoneEvent, ApiResponse, DimensionFact, DimensionView, TriggerAnalysisResult,
};
use crate::services::{run_analysis, run_followup};
use crate::state::AppState;

#[tauri::command]
pub async fn list_dimensions(
    symbol: Option<String>,
) -> Result<ApiResponse<Vec<DimensionView>>, String> {
    let dims = if let Some(sym) = symbol {
        let sector = sectors::sector_context(&sym);
        let code = sector["code"].as_str().unwrap_or("");
        dimensions::sector_dimension_codes(code)
            .into_iter()
            .map(|c| DimensionView {
                code: c.to_string(),
                label: dimensions::dimension_label(c).to_string(),
            })
            .collect()
    } else {
        dimensions::all_dimensions()
            .iter()
            .map(|d| DimensionView {
                code: d.code.to_string(),
                label: d.label.to_string(),
            })
            .collect()
    };
    Ok(ApiResponse::ok(dims))
}

#[tauri::command]
pub async fn list_dimension_facts(
    state: State<'_, Arc<AppState>>,
    symbol: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<DimensionFact>>, String> {
    let sym = symbol.unwrap_or_else(|| "rb0".into());
    match state
        .db
        .get_dimension_facts(&sym, limit.unwrap_or(50).min(200))
    {
        Ok(facts) => Ok(ApiResponse::ok(facts)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn trigger_analysis(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    symbol: String,
    trigger: Option<String>,
    provider: Option<String>,
) -> Result<ApiResponse<TriggerAnalysisResult>, String> {
    let trigger = trigger.unwrap_or_else(|| "manual".into());
    match run_analysis(
        &state,
        Some(&app),
        &symbol,
        &trigger,
        provider.as_deref(),
        false,
        None,
    )
    .await
    {
        Ok(report) => Ok(ApiResponse::ok(TriggerAnalysisResult {
            report_id: report.id,
            symbol: report.symbol,
        })),
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

#[tauri::command]
pub async fn stream_analysis(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    symbol: String,
    trigger: Option<String>,
    provider: Option<String>,
) -> Result<(), String> {
    let trigger = trigger.unwrap_or_else(|| "manual".into());
    let st = Arc::clone(state.inner());
    let sym = symbol.clone();
    tauri::async_runtime::spawn(async move {
        match run_analysis(
            &st,
            Some(&app),
            &sym,
            &trigger,
            provider.as_deref(),
            true,
            None,
        )
        .await
        {
            Ok(report) => {
                let _ = app.emit(
                    "analysis-done",
                    AnalysisDoneEvent {
                        status: "ok".into(),
                        report_id: report.id.clone(),
                        symbol: report.symbol.clone(),
                        dimension_summary: report.dimension_summary.clone(),
                    },
                );
                let _ = app.emit(
                    "notification",
                    crate::models::NotificationEvent {
                        msg_type: "notification".into(),
                        level: "info".into(),
                        title: format!("{} 分析报告已生成", report.symbol),
                        body: report.context_summary,
                        link: Some(format!("/reports/{}", report.id)),
                    },
                );
            }
            Err(e) => {
                let _ = app.emit("analysis-error", e);
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn analysis_followup(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    report_id: String,
    question: String,
    provider: Option<String>,
) -> Result<(), String> {
    let st = Arc::clone(state.inner());
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_followup(&st, &app, &report_id, &question, provider.as_deref()).await {
            let _ = app.emit("followup-error", e);
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn trigger_batch_analysis(
    state: State<'_, Arc<AppState>>,
    symbols: Option<Vec<String>>,
    trigger: Option<String>,
    provider: Option<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let symbols: Vec<String> = match symbols {
        Some(list) if !list.is_empty() => list
            .into_iter()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => crate::engine::sectors::core_product_symbols(),
    };
    if symbols.is_empty() {
        return Ok(ApiResponse::err("no core products configured"));
    }
    state
        .batch_analysis
        .spawn(
            Arc::clone(state.inner()),
            symbols.clone(),
            trigger.unwrap_or_else(|| "manual".into()),
            provider,
        )
        .map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(serde_json::json!({
        "started": true,
        "total": symbols.len()
    })))
}

#[tauri::command]
pub async fn get_batch_status(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::BatchJobStatus>, String> {
    Ok(ApiResponse::ok(state.batch_analysis.get_status().await))
}

#[tauri::command]
pub async fn trigger_comprehensive_analysis(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let total = crate::engine::sectors::core_product_symbols().len();
    let st = Arc::clone(state.inner());
    let status = state.schedule_status.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::services::run_full_cycle(&st, Some(&app), "manual", status).await;
    });
    Ok(ApiResponse::ok(serde_json::json!({
        "started": true,
        "total": total,
        "includes_data_fetch": true
    })))
}
