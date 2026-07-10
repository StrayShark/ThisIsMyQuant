use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::engine::{dimensions, sectors};
use crate::models::{
    AnalysisDoneEvent, AnalysisReport, ApiResponse, DimensionFact, DimensionView,
    TriggerAnalysisResult,
};
use crate::services::{run_analysis, run_followup, SimTradingService};
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

#[tauri::command]
pub async fn generate_trade_review(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
    days: Option<i64>,
) -> Result<ApiResponse<AnalysisReport>, String> {
    let sim = SimTradingService::new(state.db.clone(), state.quote_cache.clone());
    let account_id = match account_id {
        Some(id) => id,
        None => match sim.default_account() {
            Ok(a) => a.id,
            Err(e) => return Ok(ApiResponse::err(e.to_string())),
        },
    };
    let snapshot = match sim.get_snapshot(Some(&account_id)) {
        Ok(s) => s,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };
    let trades = match sim.list_trades(Some(&account_id), None, 100) {
        Ok(t) => t,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };
    let journals = match sim.list_journals(Some(&account_id), None, 50) {
        Ok(j) => j,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };
    let curve = match sim.list_equity_curve(&account_id, days.unwrap_or(30)) {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let prompt = format!(
        "请基于以下模拟交易记录生成一份交易复盘报告。只评价执行纪律、风险暴露和可改进动作，不承诺收益。必须包含免责声明：仅供参考，不构成投资建议。\n\n账户：{}\n权益：{:.2}\n已实现盈亏：{:.2}\n未实现盈亏：{:.2}\n\n近期成交：{} 笔\n{}\n\n复盘日记：{} 条\n{}\n\n资金曲线（最近 7 个点）：{}\n",
        snapshot.account.name,
        snapshot.account.equity,
        snapshot.account.realized_pnl,
        snapshot.account.unrealized_pnl,
        trades.len(),
        serde_json::to_string(&trades).unwrap_or_default(),
        journals.len(),
        serde_json::to_string(&journals).unwrap_or_default(),
        serde_json::to_string(&curve.iter().rev().take(7).collect::<Vec<_>>()).unwrap_or_default(),
    );

    let llm = state.llm_snapshot();
    if llm.available_providers().is_empty() {
        return Ok(ApiResponse::err("no LLM provider configured"));
    }
    let provider = state.config().default_llm_provider.clone();
    let content = match llm
        .complete(&prompt, "你是一位期货交易复盘教练。", Some(&provider))
        .await
    {
        Ok(r) => r,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let report = AnalysisReport {
        id: uuid::Uuid::new_v4().to_string(),
        symbol: "portfolio".into(),
        trigger: "trade_review".into(),
        provider,
        prompt_version: crate::engine::PROMPT_VERSION.into(),
        context_summary: format!(
            "account={} equity={:.2}",
            snapshot.account.id, snapshot.account.equity
        ),
        content: ensure_disclaimer(content),
        created_at: chrono::Utc::now().to_rfc3339(),
        tags: vec!["trade_review".into()],
        dimension_summary: None,
        news_ids: vec![],
        anomaly_reason: None,
    };
    match state.db.save_report(&report) {
        Ok(_) => Ok(ApiResponse::ok(report)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

fn ensure_disclaimer(mut content: String) -> String {
    let disclaimer = "免责声明：以上内容仅供参考，不构成投资建议。";
    if !content.contains("免责声明") && !content.contains("不构成投资建议") {
        content.push_str("\n\n");
        content.push_str(disclaimer);
    }
    content
}
