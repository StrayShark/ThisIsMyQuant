use std::sync::Arc;

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::engine::{render_followup_prompt, FOLLOWUP_SYSTEM_PROMPT};
use crate::models::{FollowupDeltaEvent, FollowupDoneEvent, FollowupMessage};
use crate::state::AppState;

pub async fn run_followup(
    state: &Arc<AppState>,
    app: &AppHandle,
    report_id: &str,
    question: &str,
    provider: Option<&str>,
) -> Result<FollowupMessage, String> {
    if state.llm_snapshot().available_providers().is_empty() {
        return Err("no LLM provider configured".into());
    }
    let question = question.trim();
    if question.is_empty() {
        return Err("question is empty".into());
    }

    let report = state
        .db
        .get_report(report_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "report not found".to_string())?;

    let facts = state
        .db
        .get_dimension_facts(&report.symbol, 30)
        .unwrap_or_default();

    let news = if !report.news_ids.is_empty() {
        state
            .db
            .get_news_by_ids(&report.news_ids)
            .unwrap_or_default()
    } else {
        state
            .db
            .get_news_for_symbol(&report.symbol.to_uppercase(), None, 10)
            .unwrap_or_default()
    };

    let prompt = render_followup_prompt(&report, &facts, &news, question);
    let mut answer = String::new();

    let llm = state.llm_snapshot();
    llm.stream(&prompt, FOLLOWUP_SYSTEM_PROMPT, provider, |token| {
        answer.push_str(&token);
        let _ = app.emit("followup-delta", FollowupDeltaEvent { text: token });
    })
    .await
    .map_err(|e| e.to_string())?;

    let provider_name = provider
        .map(String::from)
        .unwrap_or_else(|| state.config().default_llm_provider.clone());

    let msg = FollowupMessage {
        id: Uuid::new_v4().to_string(),
        report_id: report_id.to_string(),
        symbol: report.symbol.to_uppercase(),
        question: question.to_string(),
        answer: answer.clone(),
        provider: provider_name,
        created_at: Utc::now().to_rfc3339(),
    };
    state.db.save_followup(&msg).map_err(|e| e.to_string())?;

    let _ = app.emit(
        "followup-done",
        FollowupDoneEvent {
            status: "ok".into(),
            report_id: report_id.to_string(),
            followup_id: Some(msg.id.clone()),
        },
    );

    Ok(msg)
}
