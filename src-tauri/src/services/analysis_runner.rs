use std::sync::Arc;

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::engine::{
    build_context, collect_news_ids, facts_from_dimension_summary, parse_llm_report, render_prompt,
    summarize_context, PROMPT_VERSION, SYSTEM_PROMPT,
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
    anomaly_reason: Option<&str>,
) -> Result<AnalysisReport, String> {
    if state.llm_snapshot().available_providers().is_empty() {
        return Err("no LLM provider configured".into());
    }

    let jinshi_client = if state.config().jinshi_enabled {
        Some(state.jinshi.lock().await.clone())
    } else {
        None
    };
    let ctx = build_context(
        &state.akshare,
        jinshi_client.as_ref(),
        Some(state.db.as_ref()),
        symbol,
    )
    .await;
    let prompt = render_prompt(&ctx, trigger);

    let llm = state.llm_snapshot();
    let (raw_content, provider_name) = if stream {
        let app = app.ok_or("stream requires app handle")?;
        let mut chunks = String::new();
        let used = llm
            .stream(&prompt, SYSTEM_PROMPT, provider, |token| {
                chunks.push_str(&token);
                let _ = app.emit(
                    "analysis-delta",
                    crate::models::AnalysisDeltaEvent { text: token },
                );
            })
            .await
            .map_err(|e| e.to_string())?;
        (chunks, used)
    } else {
        llm.complete_with_provider(&prompt, SYSTEM_PROMPT, provider)
            .await
            .map_err(|e| e.to_string())?
    };

    let parsed = parse_llm_report(&raw_content);
    let news_ids = collect_news_ids(&ctx);
    let content = ensure_report_disclaimer(parsed.content);

    let report = AnalysisReport {
        id: Uuid::new_v4().to_string(),
        symbol: symbol.to_string(),
        trigger: trigger.to_string(),
        provider: provider_name,
        prompt_version: PROMPT_VERSION.into(),
        context_summary: summarize_context(&ctx),
        content,
        created_at: Utc::now().to_rfc3339(),
        tags: vec![],
        dimension_summary: parsed.dimension_summary,
        news_ids,
        anomaly_reason: anomaly_reason.map(String::from),
    };
    state.db.save_report(&report).map_err(|e| e.to_string())?;

    if let Some(ref summary) = report.dimension_summary {
        let facts =
            facts_from_dimension_summary(&report.symbol, &report.id, summary, &report.created_at);
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

fn ensure_report_disclaimer(content: String) -> String {
    if content.contains("不构成投资建议") {
        content
    } else {
        format!("{content}\n\n> 免责声明：本分析仅供参考，不构成投资建议。")
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_report_disclaimer;

    #[test]
    fn appends_disclaimer_when_llm_omits_it() {
        let content = ensure_report_disclaimer("短期关注区间震荡。".into());
        assert!(content.contains("不构成投资建议"));
    }

    #[test]
    fn keeps_existing_disclaimer() {
        let input = "本分析仅供参考，不构成投资建议。".to_string();
        assert_eq!(ensure_report_disclaimer(input.clone()), input);
    }
}
