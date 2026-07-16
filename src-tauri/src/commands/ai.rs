use std::sync::Arc;

use tauri::State;

use crate::engine::ai_context::{
    parse_ai_report_output, render_ai_prompt, AiContextBundle, DISCLAIMER,
};
use crate::models::{
    AiReportSummary, AiSummaryRequest, AiTask, AiTaskListResult, AnalysisReport, ApiResponse,
};
use crate::state::AppState;

const AI_PROMPT_VERSION: &str = "ai-v1";

fn allowed_task_types() -> &'static [&'static str] {
    &[
        "market_summary",
        "leaderboard_explain",
        "asset_brief",
        "watchlist_summary",
        "position_risk",
        "event_impact",
        "custom",
    ]
}

#[tauri::command]
pub async fn generate_ai_summary(
    state: State<'_, Arc<AppState>>,
    request: AiSummaryRequest,
) -> Result<ApiResponse<AiReportSummary>, String> {
    if !allowed_task_types().contains(&request.task_type.as_str()) {
        return Ok(ApiResponse::err(format!(
            "unsupported task_type: {}",
            request.task_type
        )));
    }

    let provider = request
        .provider
        .clone()
        .unwrap_or_else(|| state.config().default_llm_provider.clone());

    let llm = state.llm_snapshot();
    if llm.available_providers().is_empty() {
        return Ok(ApiResponse::err("no LLM provider configured"));
    }

    let bundle = match AiContextBundle::build(&state, &request).await {
        Ok(b) => b,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let prompt = render_ai_prompt(&bundle, &request.task_type, request.prompt.as_deref());
    let system = format!(
        "你是一位金融市场研究助手。所有结论必须基于提供的数据并列出引用来源。\
         必须包含免责声明：{DISCLAIMER} \
         输出格式优先为 JSON：{{\"content\":\"...\",\"sources\":[...],\"data_date\":\"...\",\"disclaimer\":\"{DISCLAIMER}\"}}。"
    );

    let raw = match llm.complete_json(&prompt, &system, Some(&provider)).await {
        Ok(r) => r,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    if raw.trim().is_empty() {
        return Ok(ApiResponse::err("LLM returned empty output"));
    }

    let summary = parse_ai_report_output(
        &raw,
        &bundle,
        &provider,
        &request.task_type,
        request.target_symbol.clone(),
    );

    if summary.content.trim().is_empty() {
        return Ok(ApiResponse::err("LLM output has no content"));
    }

    let report = AnalysisReport {
        id: summary.id.clone(),
        symbol: summary
            .target_symbol
            .clone()
            .unwrap_or_else(|| "ai_summary".into()),
        trigger: summary.task_type.clone(),
        provider: summary.provider.clone(),
        prompt_version: AI_PROMPT_VERSION.into(),
        context_summary: format!(
            "task={} assets={} sources={}",
            summary.task_type,
            bundle.target_assets.len(),
            summary.sources.len()
        ),
        content: summary.content.clone(),
        created_at: summary.created_at.clone(),
        tags: vec!["ai".into(), summary.task_type.clone()],
        dimension_summary: None,
        news_ids: summary
            .sources
            .iter()
            .filter(|s| s.source_type == "news")
            .filter_map(|s| s.id.clone())
            .collect(),
        anomaly_reason: None,
    };

    match state.db.save_report(&report) {
        Ok(_) => Ok(ApiResponse::ok(summary)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_ai_tasks(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<AiTaskListResult>, String> {
    let reports = match state.db.get_reports(None, None, 100) {
        Ok(r) => r,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let tasks: Vec<AiTask> = reports
        .into_iter()
        .filter(|r| r.tags.contains(&"ai".to_string()))
        .map(|r| AiTask {
            id: r.id.clone(),
            task_type: r.trigger.clone(),
            status: "completed".into(),
            target_symbol: if r.symbol == "ai_summary" {
                None
            } else {
                Some(r.symbol.clone())
            },
            provider: r.provider,
            error: None,
            created_at: r.created_at.clone(),
            updated_at: r.created_at,
        })
        .collect();

    let running = tasks.iter().filter(|t| t.status == "running").count() as i64;
    Ok(ApiResponse::ok(AiTaskListResult { tasks, running }))
}
