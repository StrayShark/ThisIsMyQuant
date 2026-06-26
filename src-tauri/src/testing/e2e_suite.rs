//! 客户端 E2E：各业务模块 + LLM 明日/短期分析。

use std::sync::Arc;
use std::time::Instant;

use chrono::{Duration, Utc};
use serde::Serialize;

use crate::engine::{build_context, dimensions, sectors};
use crate::services::run_analysis;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct E2eModuleResult {
    pub module: String,
    pub ok: bool,
    pub message: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct E2eAnalysisResult {
    pub trigger: String,
    pub report_id: String,
    pub symbol: String,
    pub content_len: usize,
    pub has_disclaimer: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct E2eSuiteReport {
    pub ok: bool,
    pub symbol: String,
    pub modules: Vec<E2eModuleResult>,
    pub analyses: Vec<E2eAnalysisResult>,
}

async fn run_module<F, Fut>(name: &str, f: F) -> E2eModuleResult
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    let start = Instant::now();
    match f().await {
        Ok(msg) => E2eModuleResult {
            module: name.into(),
            ok: true,
            message: msg,
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => E2eModuleResult {
            module: name.into(),
            ok: false,
            message: e,
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

fn report_has_horizon(content: &str, trigger: &str) -> bool {
    let lower = content.to_lowercase();
    match trigger {
        "tomorrow" => {
            lower.contains("明日") || lower.contains("下一交易日") || lower.contains("tomorrow")
        }
        "short_term" => {
            lower.contains("短期") || lower.contains("3-5") || lower.contains("3～5")
        }
        _ => true,
    }
}

pub async fn run_client_e2e_suite(state: &Arc<AppState>, symbol: &str) -> E2eSuiteReport {
    let sym = symbol.to_lowercase();
    let mut modules = Vec::new();

    modules.push(
        run_module("llm", || async {
            let llm = state.llm_snapshot();
            let providers = llm.available_providers();
            if providers.is_empty() {
                return Err("no LLM provider configured".into());
            }
            let health = llm.health().await;
            let online: Vec<_> = health.iter().filter(|(_, ok)| **ok).map(|(n, _)| n.clone()).collect();
            Ok(format!("providers={providers:?} online={online:?}"))
        })
        .await,
    );

    modules.push(
        run_module("akshare_klines", || {
            let ak = state.akshare.clone();
            let s = sym.clone();
            async move {
                let end = Utc::now();
                let start = end - Duration::days(30);
                let klines = ak.get_history(&s, "1d", start, end).await.map_err(|e| e.to_string())?;
                if klines.is_empty() {
                    return Err("empty klines".into());
                }
                Ok(format!("bars={}", klines.len()))
            }
        })
        .await,
    );

    modules.push(
        run_module("dimensions", || async {
            let dims = dimensions::all_dimensions();
            if dims.is_empty() {
                return Err("no dimensions".into());
            }
            Ok(format!("count={}", dims.len()))
        })
        .await,
    );

    modules.push(
        run_module("sectors", || {
            let s = sym.clone();
            async move {
                let ctx = sectors::sector_context(&s);
                let name = ctx["name"].as_str().unwrap_or("");
                if name.is_empty() {
                    return Err("empty sector context".into());
                }
                Ok(format!("sector={name}"))
            }
        })
        .await,
    );

    modules.push(
        run_module("analysis_context", || {
            let st = Arc::clone(state);
            let s = sym.clone();
            async move {
                let jinshi = if st.config().jinshi_enabled {
                    Some(st.jinshi.lock().await.clone())
                } else {
                    None
                };
                let ctx = build_context(
                    &st.akshare,
                    jinshi.as_ref(),
                    Some(st.db.as_ref()),
                    &s,
                )
                .await;
                let bars = ctx["bars_count"].as_u64().unwrap_or(0);
                if bars == 0 {
                    return Err("context has no klines".into());
                }
                Ok(format!("bars={bars} calendar={}", ctx["calendar_events"].as_array().map(|a| a.len()).unwrap_or(0)))
            }
        })
        .await,
    );

    modules.push(
        run_module("jinshi", || {
            let st = Arc::clone(state);
            async move {
                if !st.config().jinshi_enabled {
                    return Ok("disabled".into());
                }
                let j = st.jinshi.lock().await;
                if !j.is_connected() {
                    return Err("jinshi offline".into());
                }
                Ok("connected".into())
            }
        })
        .await,
    );

    modules.push(
        run_module("reports_db", || {
            let st = Arc::clone(state);
            async move {
                let reports = st.db.get_reports(None, None, 5).map_err(|e| e.to_string())?;
                Ok(format!("count={}", reports.len()))
            }
        })
        .await,
    );

    let mut analyses = Vec::new();
    for trigger in ["tomorrow", "short_term"] {
        let start = Instant::now();
        let label = format!("analysis_{trigger}");
        match run_analysis(state, None, &sym, trigger, None, false, None).await {
            Ok(report) => {
                let has_disclaimer = report.content.contains("不构成投资建议");
                let has_horizon = report_has_horizon(&report.content, trigger);
                let ok = report.content.len() >= 80 && has_disclaimer && has_horizon;
                let msg = format!(
                    "id={} len={} disclaimer={has_disclaimer} horizon={has_horizon}",
                    report.id,
                    report.content.len()
                );
                modules.push(E2eModuleResult {
                    module: label,
                    ok,
                    message: msg,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
                analyses.push(E2eAnalysisResult {
                    trigger: trigger.into(),
                    report_id: report.id,
                    symbol: report.symbol,
                    content_len: report.content.len(),
                    has_disclaimer,
                });
            }
            Err(e) => {
                modules.push(E2eModuleResult {
                    module: label,
                    ok: false,
                    message: e,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }
    }

    let ok = modules.iter().all(|m| m.ok) && analyses.len() == 2;
    E2eSuiteReport {
        ok,
        symbol: sym,
        modules,
        analyses,
    }
}
