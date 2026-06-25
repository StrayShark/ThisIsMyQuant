use std::sync::Arc;

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::engine::{
    build_context, collect_news_ids, facts_from_dimension_summary, parse_llm_report,
    render_prompt, summarize_context, SYSTEM_PROMPT,
};
use crate::models::{AnalysisDoneEvent, AnalysisReport, NotificationEvent};
use crate::state::AppState;

pub async fn run_analysis(
    state: &Arc<AppState>,
    app: Option<&AppHandle>,
    symbol: &str,
    trigger: &str,
    provider: Option<&str>,
    stream: bool,
) -> Result<AnalysisReport, String> {
    if state.llm.available_providers().is_empty() {
        return Err("no LLM provider configured".into());
    }

    let ctx = build_context(
        &state.akshare,
        if state.jinshi.is_connected() {
            Some(&state.jinshi)
        } else {
            None
        },
        Some(state.db.as_ref()),
        symbol,
    )
    .await;
    let prompt = render_prompt(&ctx, trigger);
    let provider_name = provider
        .map(String::from)
        .unwrap_or_else(|| state.llm.default_provider().to_string());

    let raw_content = if stream {
        let app = app.ok_or("stream requires app handle")?;
        let mut chunks = String::new();
        state
            .llm
            .stream(
                &prompt,
                SYSTEM_PROMPT,
                provider.as_deref(),
                |token| {
                    chunks.push_str(&token);
                    let _ = app.emit(
                        "analysis-delta",
                        crate::models::AnalysisDeltaEvent { text: token },
                    );
                },
            )
            .await
            .map_err(|e| e.to_string())?;
        chunks
    } else {
        state
            .llm
            .complete(&prompt, SYSTEM_PROMPT, provider.as_deref())
            .await
            .map_err(|e| e.to_string())?
    };

    let parsed = parse_llm_report(&raw_content);
    let news_ids = collect_news_ids(&ctx);

    let report = AnalysisReport {
        id: Uuid::new_v4().to_string(),
        symbol: symbol.to_string(),
        trigger: trigger.to_string(),
        provider: provider_name,
        prompt_version: "v2".into(),
        context_summary: summarize_context(&ctx),
        content: parsed.content,
        created_at: Utc::now().to_rfc3339(),
        tags: vec![],
        dimension_summary: parsed.dimension_summary,
        news_ids,
    };
    state.db.save_report(&report).map_err(|e| e.to_string())?;

    if let Some(ref summary) = report.dimension_summary {
        let facts = facts_from_dimension_summary(
            &report.symbol,
            &report.id,
            summary,
            &report.created_at,
        );
        let _ = state.db.replace_report_facts(&report.id, &facts);
    }

    if !stream {
        if let Some(app) = app {
            let sym = report.symbol.clone();
            let id = report.id.clone();
            let summary = report.context_summary.clone();
            let dimension_summary = report.dimension_summary.clone();
            let _ = app.emit(
                "analysis-done",
                AnalysisDoneEvent {
                    status: "ok".into(),
                    report_id: report.id.clone(),
                    symbol: report.symbol.clone(),
                    dimension_summary,
                },
            );
            let _ = app.emit(
                "notification",
                NotificationEvent {
                    msg_type: "notification".into(),
                    level: "info".into(),
                    title: format!("{sym} 分析报告已生成"),
                    body: summary,
                    link: Some(format!("/reports/{id}")),
                },
            );
        }
    }

    Ok(report)
}
