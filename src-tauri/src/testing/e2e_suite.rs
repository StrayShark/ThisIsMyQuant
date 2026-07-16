//! 客户端 E2E：各业务模块 + LLM 明日/短期分析。

use std::sync::Arc;
use std::time::Instant;

use chrono::{Duration, Utc};
use serde::Serialize;

use crate::adapters::{StockBarsRequest, StockDataProvider};
use crate::engine::ai_context::{AiContextBundle, DISCLAIMER};
use crate::engine::{build_context, dimensions, sectors};
use crate::models::{AiSummaryRequest, PlaceSimOrderRequest};
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
    pub source_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct E2eSymbolCheck {
    pub symbol: String,
    pub sector: String,
    pub bars: usize,
    pub context_bars: u64,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct E2eSuiteReport {
    pub ok: bool,
    pub symbol: String,
    pub symbol_checks: Vec<E2eSymbolCheck>,
    pub stock_checks: Vec<E2eModuleResult>,
    pub detail_checks: Vec<E2eModuleResult>,
    pub sim_checks: Vec<E2eModuleResult>,
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
        "short_term" => lower.contains("短期") || lower.contains("3-5") || lower.contains("3～5"),
        _ => true,
    }
}

pub async fn run_client_e2e_suite(
    state: &Arc<AppState>,
    symbol: &str,
    symbols: &[String],
) -> E2eSuiteReport {
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
            let online: Vec<_> = health
                .iter()
                .filter(|(_, ok)| **ok)
                .map(|(n, _)| n.clone())
                .collect();
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
                let klines = ak
                    .get_history(&s, "1d", start, end)
                    .await
                    .map_err(|e| e.to_string())?;
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
                let ctx =
                    build_context(&st.akshare, jinshi.as_ref(), Some(st.db.as_ref()), &s).await;
                let bars = ctx["bars_count"].as_u64().unwrap_or(0);
                if bars == 0 {
                    return Err("context has no klines".into());
                }
                Ok(format!(
                    "bars={bars} calendar={}",
                    ctx["calendar_events"]
                        .as_array()
                        .map(|a| a.len())
                        .unwrap_or(0)
                ))
            }
        })
        .await,
    );

    modules.push(
        run_module("fundamentals", || {
            let ak = state.akshare.clone();
            let s = sym.clone();
            async move {
                let v = crate::engine::fundamentals::fetch_fundamentals(&ak, &s).await;
                let source = v["source"].as_str().unwrap_or("unavailable");
                if source == "unavailable" {
                    return Err(v["note"]
                        .as_str()
                        .unwrap_or("fundamentals unavailable")
                        .into());
                }
                Ok(format!(
                    "source={source} oi={} prev_close={}",
                    v["open_interest"].as_i64().unwrap_or(0),
                    v["prev_close"].as_f64().unwrap_or(0.0)
                ))
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
        run_module("overseas_symbols", || async {
            let v = crate::adapters::list_overseas_symbols();
            let status = v["status"].as_str().unwrap_or("");
            let count = v["symbols"].as_array().map(|a| a.len()).unwrap_or(0);
            if status != "ok" || count == 0 {
                return Err(format!(
                    "overseas source unavailable status={status} count={count}"
                ));
            }
            Ok(format!(
                "source={} symbols={count}",
                v["source"].as_str().unwrap_or("unknown")
            ))
        })
        .await,
    );

    modules.push(
        run_module("professional_dashboard", || {
            let st = Arc::clone(state);
            async move {
                let v = crate::commands::data::professional_dashboard_view(&st).await;
                if v.factors.len() < 5 {
                    return Err(format!(
                        "expected 5 factor snapshots, got {}",
                        v.factors.len()
                    ));
                }
                if v.report_workflow.len() < 4 {
                    return Err(format!(
                        "expected report workflow, got {}",
                        v.report_workflow.len()
                    ));
                }
                if v.overseas_links.len() < 5 {
                    return Err(format!(
                        "expected overseas links, got {}",
                        v.overseas_links.len()
                    ));
                }
                Ok(format!(
                    "news={} factors={} alerts={} workflow={} overseas={}",
                    v.decision_flow.len(),
                    v.factors.len(),
                    v.alerts.len(),
                    v.report_workflow.len(),
                    v.overseas_links.len()
                ))
            }
        })
        .await,
    );

    modules.push(
        run_module("reports_db", || {
            let st = Arc::clone(state);
            async move {
                let reports = st
                    .db
                    .get_reports(None, None, 5)
                    .map_err(|e| e.to_string())?;
                Ok(format!("count={}", reports.len()))
            }
        })
        .await,
    );

    let stock_checks = run_stock_live_checks(state).await;
    let detail_checks = run_detail_live_checks(state, &sym).await;
    let sim_checks = run_sim_live_checks(state).await;

    let mut symbol_checks = Vec::new();
    let check_symbols = if symbols.is_empty() {
        vec![sym.clone()]
    } else {
        symbols.iter().map(|s| s.to_lowercase()).collect()
    };
    for check_sym in check_symbols {
        let end = Utc::now();
        let start = end - Duration::days(30);
        let sector_ctx = sectors::sector_context(&check_sym);
        let sector = sector_ctx["name"].as_str().unwrap_or("未分类").to_string();
        let klines = state
            .akshare
            .get_history(&check_sym, "1d", start, end)
            .await
            .unwrap_or_default();
        let jinshi = if state.config().jinshi_enabled {
            Some(state.jinshi.lock().await.clone())
        } else {
            None
        };
        let ctx = build_context(
            &state.akshare,
            jinshi.as_ref(),
            Some(state.db.as_ref()),
            &check_sym,
        )
        .await;
        let context_bars = ctx["bars_count"].as_u64().unwrap_or(0);
        let ok = !klines.is_empty() && context_bars > 0 && sector != "未分类";
        symbol_checks.push(E2eSymbolCheck {
            symbol: check_sym,
            sector,
            bars: klines.len(),
            context_bars,
            ok,
            message: if ok {
                "ok".into()
            } else {
                "missing klines/context/sector".into()
            },
        });
    }

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
                    source_count: 0,
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

    let ai_sym = sym.clone();
    modules.push(
        run_module("ai_sources", || {
            let st = Arc::clone(state);
            async move {
                let request = AiSummaryRequest {
                    task_type: "asset_brief".to_string(),
                    target_symbol: Some(ai_sym.clone()),
                    target_assets: Some(vec![ai_sym.clone()]),
                    prompt: Some("用两句话生成带来源的标的速览。".to_string()),
                    provider: None,
                };
                let bundle = AiContextBundle::build(&st, &request)
                    .await
                    .map_err(|e| e.to_string())?;
                let context_source_count = bundle.quotes.len()
                    + bundle.news.len()
                    + bundle.events.len()
                    + usize::from(!bundle.klines.is_empty())
                    + bundle.positions.len()
                    + bundle.orders.len()
                    + bundle.reports.len();
                if context_source_count == 0 {
                    return Err("AI context has no sources".into());
                }
                let llm = st.llm_snapshot();
                let provider = st.config().default_llm_provider.clone();
                let prompt = crate::engine::ai_context::render_ai_prompt(
                    &bundle,
                    &request.task_type,
                    request.prompt.as_deref(),
                );
                let system = format!(
                    "你是一位金融市场研究助手。输出 JSON，必须包含 content、sources、data_date、disclaimer，免责声明必须是：{DISCLAIMER}"
                );
                let raw = llm
                    .complete_json(&prompt, &system, Some(&provider))
                    .await
                    .map_err(|e| e.to_string())?;
                let summary = crate::engine::ai_context::parse_ai_report_output(
                    &raw,
                    &bundle,
                    &provider,
                    &request.task_type,
                    request.target_symbol,
                );
                if summary.sources.is_empty() {
                    return Err("AI summary has no sources".into());
                }
                if !summary.disclaimer.contains("不构成投资建议") {
                    return Err("AI summary missing disclaimer".into());
                }
                Ok(format!(
                    "sources={} data_date={}",
                    summary.sources.len(),
                    summary.data_date.as_deref().unwrap_or("-")
                ))
            }
        })
        .await,
    );

    let ok = modules.iter().all(|m| m.ok)
        && symbol_checks.iter().all(|s| s.ok)
        && stock_checks.iter().all(|m| m.ok)
        && detail_checks.iter().all(|m| m.ok)
        && sim_checks.iter().all(|m| m.ok)
        && analyses.len() == 2;
    E2eSuiteReport {
        ok,
        symbol: sym,
        symbol_checks,
        stock_checks,
        detail_checks,
        sim_checks,
        modules,
        analyses,
    }
}

async fn run_stock_live_checks(state: &Arc<AppState>) -> Vec<E2eModuleResult> {
    let mut checks = Vec::new();
    checks.push(
        run_module("a_stock_symbols", || {
            let st = Arc::clone(state);
            async move {
                if st.db.count_stock_symbols().unwrap_or(0) == 0 {
                    let symbols = st
                        .stock_provider
                        .list_symbols()
                        .await
                        .map_err(|e| e.to_string())?;
                    if symbols.is_empty() {
                        return Err("provider returned empty symbols".into());
                    }
                    st.db
                        .save_stock_symbols(&symbols)
                        .map_err(|e| e.to_string())?;
                }
                let symbols = st
                    .db
                    .list_stock_symbols(None, None, 3)
                    .map_err(|e| e.to_string())?;
                if symbols.len() < 3 {
                    return Err(format!("expected >=3 stock symbols, got {}", symbols.len()));
                }
                Ok(format!("symbols={}", symbols.len()))
            }
        })
        .await,
    );

    checks.push(
        run_module("a_stock_indices", || {
            let st = Arc::clone(state);
            async move {
                let codes = ["000001.SH", "399001.SZ", "399006.SZ"];
                let mut ok_count = 0usize;
                for code in codes {
                    let req = StockBarsRequest {
                        code: code.to_string(),
                        adjustment: "none".to_string(),
                        start_date: None,
                        end_date: None,
                        limit: 1,
                    };
                    let bars = st
                        .stock_provider
                        .list_index_bars(req)
                        .await
                        .map_err(|e| e.to_string())?;
                    if bars.last().and_then(|b| b.close).unwrap_or(0.0) > 0.0 {
                        ok_count += 1;
                    }
                    st.db
                        .save_stock_index_daily_bars(&bars)
                        .map_err(|e| e.to_string())?;
                }
                if ok_count < 3 {
                    return Err(format!("expected 3 live indices, got {ok_count}"));
                }
                Ok(format!("indices={ok_count}"))
            }
        })
        .await,
    );

    checks.push(
        run_module("a_stock_bars", || {
            let st = Arc::clone(state);
            async move {
                let req = StockBarsRequest {
                    code: "600000.SH".to_string(),
                    adjustment: "none".to_string(),
                    start_date: None,
                    end_date: None,
                    limit: 30,
                };
                let bars = st
                    .stock_provider
                    .list_stock_bars(req)
                    .await
                    .map_err(|e| e.to_string())?;
                if bars.is_empty() {
                    return Err("600000.SH empty bars".into());
                }
                st.db
                    .save_stock_daily_bars(&bars)
                    .map_err(|e| e.to_string())?;
                Ok(format!("600000.SH bars={}", bars.len()))
            }
        })
        .await,
    );

    checks
}

async fn run_detail_live_checks(state: &Arc<AppState>, symbol: &str) -> Vec<E2eModuleResult> {
    let mut checks = Vec::new();
    checks.push(
        run_module("detail_futures", || {
            let st = Arc::clone(state);
            let sym = symbol.to_string();
            async move {
                let end = Utc::now();
                let start = end - Duration::days(30);
                let klines = st
                    .akshare
                    .get_history(&sym, "1d", start, end)
                    .await
                    .map_err(|e| e.to_string())?;
                let sector = sectors::sector_context(&sym);
                let sector_name = sector["name"].as_str().unwrap_or("");
                if !klines.is_empty() && !sector_name.is_empty() {
                    Ok(format!(
                        "{} bars={} sector={sector_name}",
                        sym,
                        klines.len()
                    ))
                } else {
                    Err(format!("missing futures detail data {sym}"))
                }
            }
        })
        .await,
    );

    checks.push(
        run_module("detail_stock", || {
            let st = Arc::clone(state);
            async move {
                if st
                    .db
                    .get_stock_symbol("600000.SH")
                    .map_err(|e| e.to_string())?
                    .is_none()
                {
                    let symbols = st
                        .stock_provider
                        .list_symbols()
                        .await
                        .map_err(|e| e.to_string())?;
                    st.db
                        .save_stock_symbols(&symbols)
                        .map_err(|e| e.to_string())?;
                }
                let req = StockBarsRequest {
                    code: "600000.SH".to_string(),
                    adjustment: "none".to_string(),
                    start_date: None,
                    end_date: None,
                    limit: 30,
                };
                let bars = st
                    .stock_provider
                    .list_stock_bars(req)
                    .await
                    .map_err(|e| e.to_string())?;
                st.db
                    .save_stock_daily_bars(&bars)
                    .map_err(|e| e.to_string())?;
                let detail = st
                    .db
                    .get_stock_symbol("600000.SH")
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "600000.SH symbol not found".to_string())?;
                let latest = st
                    .db
                    .get_stock_daily_bars("600000.SH", "none", 1)
                    .map_err(|e| e.to_string())?
                    .last()
                    .cloned();
                if latest.as_ref().and_then(|b| b.close).unwrap_or(0.0) <= 0.0 {
                    return Err("600000.SH latest close missing".into());
                }
                Ok(format!("{} latest_bar={}", detail.name, latest.is_some()))
            }
        })
        .await,
    );
    checks
}

async fn run_sim_live_checks(state: &Arc<AppState>) -> Vec<E2eModuleResult> {
    vec![
        run_module("simulation_limit_submit_cancel", || {
            let st = Arc::clone(state);
            async move {
                let account = st
                    .sim_trading
                    .create_account("E2E 基础模拟盘".to_string(), 1_000_000.0)
                    .map_err(|e| e.to_string())?;
                st.sim_trading.seed_price("RB0", 3500.0);
                let order = st
                    .sim_trading
                    .place_order(PlaceSimOrderRequest {
                        account_id: account.id.clone(),
                        symbol: "RB0".to_string(),
                        side: "buy".to_string(),
                        offset: "open".to_string(),
                        order_type: "limit".to_string(),
                        price: Some(1000.0),
                        trigger_price: None,
                        stop_loss_price: None,
                        take_profit_price: None,
                        oco_group_id: None,
                        parent_order_id: None,
                        tif: Some("GFD".to_string()),
                        condition_operator: None,
                        trailing_distance_ticks: None,
                        quantity: 1,
                    })
                    .map_err(|e| e.to_string())?;
                if order.status != "open" {
                    return Err(format!("expected open order, got {}", order.status));
                }
                let cancelled = st
                    .sim_trading
                    .cancel_order(&order.id)
                    .map_err(|e| e.to_string())?;
                if cancelled.status != "cancelled" {
                    return Err(format!(
                        "expected cancelled order, got {}",
                        cancelled.status
                    ));
                }
                Ok(format!(
                    "order={} status={}",
                    cancelled.id, cancelled.status
                ))
            }
        })
        .await,
    ]
}
