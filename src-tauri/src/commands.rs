use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter, State};

use crate::engine::sectors::{self, SectorView};
use crate::engine::dimensions;
use crate::models::{
    AnalysisDoneEvent, AnalysisReport, ApiResponse, AppSettingsView, CalendarEvent, Contract,
    DimensionFact,
    DimensionView, FollowupMessage, HealthResponse,
    JinshiHealth, KLine, RealtimeHealth, TriggerAnalysisResult,
    AkshareHealth, PollStatus, NewsPollStatus,
};
use crate::services::{run_analysis, run_followup};
use crate::state::AppState;

#[tauri::command]
pub async fn get_health(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<HealthResponse>, String> {
    let llm = state.llm.health().await;
    let mut feeds = std::collections::HashMap::new();
    feeds.insert("akshare".into(), state.akshare_ready);
    let poll_status = if let Some(p) = &state.market_poll {
        let s = p.status().await;
        Some(PollStatus {
            running: s.running,
            interval: s.interval,
            symbols: s.symbols,
            symbol_count: s.symbol_count,
        })
    } else {
        None
    };
    let poll_running = poll_status.as_ref().map(|p| p.running).unwrap_or(false);
    Ok(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        feeds,
        llm,
        db: true,
        akshare: AkshareHealth {
            history: state.akshare_ready,
        },
        poll: poll_status,
        news_poll: state.news_poll.as_ref().map(|_| NewsPollStatus {
            running: true,
            interval: state.config.jinshi_poll_interval,
        }),
        realtime: RealtimeHealth {
            available: poll_running,
            source: if poll_running {
                Some("market_poll".into())
            } else {
                None
            },
        },
        jinshi: JinshiHealth {
            enabled: state.config.jinshi_enabled,
            connected: state.jinshi.is_connected(),
        },
    }))
}

#[tauri::command]
pub async fn list_products(
    state: State<'_, Arc<AppState>>,
    tier: Option<String>,
) -> Result<ApiResponse<Vec<SectorView>>, String> {
    let filter = tier.unwrap_or_else(|| "core".into());
    let liquidity = state.db.get_latest_liquidity_map().unwrap_or_default();
    Ok(ApiResponse::ok(sectors::build_catalog(&filter, &liquidity)))
}

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
    match state.db.get_dimension_facts(&sym, limit.unwrap_or(50).min(200)) {
        Ok(facts) => Ok(ApiResponse::ok(facts)),
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
pub async fn list_contracts(
    state: State<'_, Arc<AppState>>,
    exchange: Option<String>,
) -> Result<ApiResponse<Vec<Contract>>, String> {
    let result = match state.db.get_contracts(exchange.as_deref()) {
        Ok(list) if !list.is_empty() => Ok(list),
        _ => state.akshare.get_contracts().await.map(|contracts| {
            let _ = state.db.save_contracts(&contracts);
            contracts
        }),
    };
    match result {
        Ok(contracts) => Ok(ApiResponse::ok(contracts)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_klines(
    state: State<'_, Arc<AppState>>,
    symbol: String,
    interval: String,
    start: Option<String>,
    end: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<KLine>>, String> {
    let end_dt = end
        .and_then(|s| crate::models::parse_dt(&s))
        .unwrap_or_else(Utc::now);
    let start_dt = start
        .and_then(|s| crate::models::parse_dt(&s))
        .unwrap_or_else(|| end_dt - Duration::days(120));
    let limit = limit.unwrap_or(1000).min(10000);
    let sym = symbol.to_lowercase();

    if let Ok(cached) = state.db.get_klines(&sym, &interval, start_dt, end_dt, limit) {
        if !cached.is_empty() {
            return Ok(ApiResponse::ok(cached));
        }
    }

    match state
        .akshare
        .get_history(&sym, &interval, start_dt, end_dt)
        .await
    {
        Ok(mut klines) => {
            if klines.len() as i64 > limit {
                klines = klines.split_off(klines.len() - limit as usize);
            }
            let _ = state.db.save_klines(&klines);
            Ok(ApiResponse::ok(klines))
        }
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

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
pub async fn trigger_analysis(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    symbol: String,
    trigger: Option<String>,
    provider: Option<String>,
) -> Result<ApiResponse<TriggerAnalysisResult>, String> {
    let trigger = trigger.unwrap_or_else(|| "manual".into());
    match run_analysis(&state, Some(&app), &symbol, &trigger, provider.as_deref(), false).await {
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
        match run_analysis(&st, Some(&app), &sym, &trigger, provider.as_deref(), true).await {
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
        if let Err(e) = run_followup(
            &st,
            &app,
            &report_id,
            &question,
            provider.as_deref(),
        )
        .await
        {
            let _ = app.emit("followup-error", e);
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn list_news(
    state: State<'_, Arc<AppState>>,
    symbol: Option<String>,
    dimension: Option<String>,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<crate::models::NewsItemView>>, String> {
    let limit = limit.unwrap_or(10).min(50) as i64;
    let symbol_ref = symbol.as_deref();

    if let Some(sym) = symbol_ref {
        let main = crate::engine::sectors::get_product_by_symbol(sym)
            .map(|p| p.symbol.to_uppercase())
            .unwrap_or_else(|| sym.to_uppercase());
        match state.db.get_news_for_symbol(
            &main,
            dimension.as_deref(),
            limit,
        ) {
            Ok(items) if !items.is_empty() => return Ok(ApiResponse::ok(items)),
            Ok(_) => {}
            Err(e) => log::warn!("list_news db: {e}"),
        }
    } else {
        match state.db.get_latest_news(limit) {
            Ok(items) if !items.is_empty() => return Ok(ApiResponse::ok(items)),
            Ok(_) => {}
            Err(e) => log::warn!("list_news db: {e}"),
        }
    }

    if !state.jinshi.is_connected() {
        return Ok(ApiResponse::ok(vec![]));
    }
    let result = if let Some(sym) = symbol_ref {
        state.jinshi.fetch_for_symbol(sym, limit as usize).await
    } else {
        state.jinshi.fetch_latest(limit as usize).await
    };
    match result {
        Ok(items) => {
            let views: Vec<crate::models::NewsItemView> = items
                .into_iter()
                .map(|n| crate::models::NewsItemView {
                    id: String::new(),
                    title: n.title,
                    summary: n.summary,
                    source: n.source,
                    category_id: n.category_id,
                    display_time: n.display_time,
                    url: n.url,
                    classifications: vec![],
                })
                .collect();
            Ok(ApiResponse::ok(views))
        }
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_calendar_events(
    state: State<'_, Arc<AppState>>,
    start: Option<String>,
    end: Option<String>,
    min_star: Option<u8>,
    country: Option<String>,
) -> Result<ApiResponse<Vec<CalendarEvent>>, String> {
    if !state.config.jinshi_enabled {
        return Ok(ApiResponse::ok(vec![]));
    }
    let (default_start, default_end) = crate::adapters::default_calendar_range_from_today();
    let start_date = start
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or(default_start);
    let end_date = end
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or(default_end);
    let min_star = min_star.unwrap_or(3).clamp(1, 5);
    match state
        .jinshi
        .fetch_calendar_events(
            start_date,
            end_date,
            min_star,
            country.as_deref(),
        )
        .await
    {
        Ok(events) => Ok(ApiResponse::ok(events)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<AppSettingsView>, String> {
    let sched = state
        .analysis_scheduler
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let (daily, realtime) = sched
        .as_ref()
        .map(|s| (s.daily_running, s.realtime_running))
        .unwrap_or((false, false));
    Ok(ApiResponse::ok(AppSettingsView {
        akshare_enabled: state.config.akshare_enabled,
        akshare_realtime_enabled: state.config.akshare_realtime_enabled,
        realtime_poll_interval: state.config.realtime_poll_interval,
        watchlist: state.config.watchlist.clone(),
        jinshi_enabled: state.config.jinshi_enabled,
        jinshi_poll_interval: state.config.jinshi_poll_interval,
        default_llm_provider: state.config.default_llm_provider.clone(),
        llm_providers: state.llm.available_providers(),
        daily_analysis_cron: state.config.daily_analysis_cron.clone(),
        realtime_analysis_interval: state.config.realtime_analysis_interval,
        scheduler_daily_running: daily,
        scheduler_realtime_running: realtime,
        database_path: state.config.database_path.display().to_string(),
        news_classify_enabled: state.config.news_classify.enabled,
        news_classify_batch: state.config.news_classify.batch_size,
    }))
}

#[tauri::command]
pub async fn market_subscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = &state.market_poll {
        poll.subscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(serde_json::json!({ "subscribed": symbols })))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}
