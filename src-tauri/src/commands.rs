use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter, State};

use crate::config::LLM_CATALOG;
use crate::crypto::credentials::{encryption_ready, mask_secret};
use crate::engine::sectors::{self, SectorView};
use crate::engine::dimensions;
use crate::models::{
    AnalysisDoneEvent, AnalysisReport, ApiResponse, AppSettingsView, CalendarEvent, Contract,
    DimensionFact,
    DimensionView, FollowupMessage, HealthResponse,
    JinshiHealth, KLine, LlmProviderSetupView, LlmSetupStatus, NewsRecord, RealtimeHealth,
    SaveLlmSetupPayload, TriggerAnalysisResult,
    AkshareHealth, PollStatus, NewsPollStatus,
};
use crate::services::{
    hydrate_config_llm, load_llm_providers, restart_runtime_polls, run_analysis, run_followup,
    save_llm_provider, sync_llm_to_state,
};
use crate::state::AppState;

#[tauri::command]
pub async fn get_health(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<HealthResponse>, String> {
    let llm_health = state.llm_snapshot().health().await;
    let mut feeds = std::collections::HashMap::new();
    feeds.insert("akshare".into(), state.akshare_ready);
    let poll_status = if let Some(p) = state.poll_handle().await {
        let s = p.status().await;
        Some(PollStatus {
            running: s.running,
            interval: s.interval,
            symbols: s.symbols,
            symbol_count: s.symbol_count,
            feed_source: s.feed_source,
        })
    } else {
        None
    };
    let poll_running = poll_status.as_ref().map(|p| p.running).unwrap_or(false);
    let (cal_ready, cal_at, cal_count) = {
        let j = state.jinshi.lock().await;
        j.calendar_status()
    };
    let jinshi_connected = state.jinshi.lock().await.is_connected();
    let news_poll_running = state.news_poll.lock().await.is_some();
    Ok(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        feeds,
        llm: llm_health,
        db: true,
        akshare: AkshareHealth {
            history: state.akshare_ready,
        },
        poll: poll_status,
        news_poll: news_poll_running.then(|| NewsPollStatus {
            running: true,
            interval: state.config().jinshi_poll_interval,
        }),
        realtime: RealtimeHealth {
            available: poll_running,
            source: if poll_running {
                Some(state.feed_source.clone())
            } else {
                None
            },
        },
        jinshi: JinshiHealth {
            enabled: state.config().jinshi_enabled,
            connected: jinshi_connected,
            calendar_ready: cal_ready,
            calendar_fetched_at: cal_at,
            calendar_cached_events: cal_count,
        },
        llm_last_errors: state.llm_snapshot().last_errors(),
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
    match run_analysis(&state, Some(&app), &symbol, &trigger, provider.as_deref(), false, None).await {
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
        match run_analysis(&st, Some(&app), &sym, &trigger, provider.as_deref(), true, None).await {
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

    if !state.jinshi.lock().await.is_connected() {
        return Ok(ApiResponse::ok(vec![]));
    }
    let jinshi = state.jinshi.lock().await;
    let result = if let Some(sym) = symbol_ref {
        jinshi.fetch_for_symbol(sym, limit as usize).await
    } else {
        jinshi.fetch_latest(limit as usize).await
    };
    drop(jinshi);
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
pub async fn list_news_by_ids(
    state: State<'_, Arc<AppState>>,
    ids: Vec<String>,
) -> Result<ApiResponse<Vec<crate::models::NewsItemView>>, String> {
    match state.db.get_news_by_ids(&ids) {
        Ok(items) => Ok(ApiResponse::ok(items)),
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
    keyword: Option<String>,
) -> Result<ApiResponse<Vec<CalendarEvent>>, String> {
    if !state.config().jinshi_enabled {
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
    let cache_key = format!(
        "{start_date}|{end_date}|{min_star}|{}",
        country.as_deref().unwrap_or("*")
    );

    let apply_keyword = |mut events: Vec<CalendarEvent>| -> Vec<CalendarEvent> {
        if let Some(kw) = keyword.as_ref().filter(|k| !k.trim().is_empty()) {
            events = crate::engine::calendar_filter::filter_by_keyword(events, kw);
        }
        events
    };

    match state
        .jinshi
        .lock()
        .await
        .fetch_calendar_events(start_date, end_date, min_star, country.as_deref())
        .await
    {
        Ok(events) => {
            let filtered = apply_keyword(events);
            let _ = state
                .db
                .save_calendar_cache(&cache_key, &filtered, None);
            Ok(ApiResponse::ok(filtered))
        }
        Err(e) => {
            if let Ok(Some(cached)) = state.db.load_calendar_cache(&cache_key) {
                let filtered = apply_keyword(cached);
                let msg = format!(
                    "日历接口不可用，已使用本地缓存（{}）",
                    humanize_calendar_error(&e.to_string())
                );
                Ok(ApiResponse::ok_warn(filtered, msg))
            } else {
                Ok(ApiResponse::err(e.to_string()))
            }
        }
    }
}

fn humanize_calendar_error(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("401") || lower.contains("403") || lower.contains("token") {
        "MCP Token 可能已过期".into()
    } else if lower.contains("429") || lower.contains("rate") || lower.contains("限流") {
        "请求过于频繁，请稍后再试".into()
    } else {
        raw.chars().take(120).collect()
    }
}

#[tauri::command]
pub async fn list_unclassified_news(
    state: State<'_, Arc<AppState>>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<NewsRecord>>, String> {
    match state.db.get_unclassified_news(limit.unwrap_or(30).min(100)) {
        Ok(items) => Ok(ApiResponse::ok(items)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<AppSettingsView>, String> {
    let schedule_st = state.schedule_status.lock().await;
    let scheduler_running = schedule_st.enabled || schedule_st.cycle_in_progress;
    Ok(ApiResponse::ok(AppSettingsView {
        akshare_enabled: state.config().akshare_enabled,
        akshare_realtime_enabled: state.config().akshare_realtime_enabled,
        realtime_poll_interval: state.config().realtime_poll_interval,
        watchlist: state.config().watchlist.clone(),
        jinshi_enabled: state.config().jinshi_enabled,
        jinshi_poll_interval: state.config().jinshi_poll_interval,
        default_llm_provider: state.config().default_llm_provider.clone(),
        llm_providers: state.llm_snapshot().available_providers(),
        schedule_analysis_trigger: state.config().schedule_analysis_trigger.clone(),
        daily_briefing_enabled: state.config().daily_briefing_enabled,
        daily_briefing_hour: state.config().daily_briefing_hour,
        schedule_interval_hours: state.config().schedule_interval_hours,
        schedule_enabled: state.config().schedule_enabled,
        scheduler_running,
        database_path: state.config().database_path.display().to_string(),
        news_classify_enabled: state.config().news_classify.enabled,
        news_classify_batch: state.config().news_classify.batch_size,
        market_feed: state.config().market_feed.clone(),
        anomaly_enabled: state.config().anomaly_enabled,
        anomaly_price_pct: state.config().anomaly_price_pct,
        anomaly_window_secs: state.config().anomaly_window_secs,
        anomaly_cooldown_secs: state.config().anomaly_cooldown_secs,
        backfill_days_daily: state.config().backfill_days_daily,
        backfill_days_minute: state.config().backfill_days_minute,
        encryption_configured: crate::crypto::credentials::encryption_ready(
            &state.config().encryption_key,
        ),
        llm_keys_masked: state
            .config()
            .llm_providers
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    crate::crypto::credentials::mask_secret(&p.api_key),
                )
            })
            .collect(),
        ollama_configured: state
            .config()
            .llm_providers
            .iter()
            .any(|p| p.name == "ollama"),
        llm_last_errors: state.llm_snapshot().last_errors(),
        ticks_enabled: state.config().ticks_enabled,
        retention_days_klines: state.config().retention_days_klines,
        retention_days_ticks: state.config().retention_days_ticks,
        calendar_reminder_enabled: state.config().calendar_reminder_enabled,
        database_backend: state.config().database_backend.clone(),
        questdb_configured: !state.config().questdb_url.is_empty(),
    }))
}

#[tauri::command]
pub async fn market_subscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = state.poll_handle().await {
        poll.subscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(serde_json::json!({ "subscribed": symbols })))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}

#[tauri::command]
pub async fn market_unsubscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = state.poll_handle().await {
        poll.unsubscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(serde_json::json!({ "unsubscribed": symbols })))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}

#[tauri::command]
pub async fn get_runtime_status(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::RuntimeStatusView>, String> {
    let poll = if let Some(p) = state.poll_handle().await {
        let s = p.status().await;
        Some(PollStatus {
            running: s.running,
            interval: s.interval,
            symbols: s.symbols,
            symbol_count: s.symbol_count,
            feed_source: s.feed_source,
        })
    } else {
        None
    };
    let backfill = state.backfill_status.lock().await.clone();
    let schedule = state.schedule_status.lock().await.clone();
    Ok(ApiResponse::ok(crate::models::RuntimeStatusView {
        poll,
        backfill,
        feed_source: if state.feed_source.is_empty() {
            None
        } else {
            Some(state.feed_source.clone())
        },
        schedule,
    }))
}

#[tauri::command]
pub async fn get_symbol_context(
    symbol: String,
) -> Result<ApiResponse<serde_json::Value>, String> {
    Ok(ApiResponse::ok(sectors::sector_context(&symbol)))
}

#[tauri::command]
pub async fn get_llm_setup(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<LlmSetupStatus>, String> {
    let cfg = state.config();
    let configured: std::collections::HashMap<_, _> = cfg
        .llm_providers
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();
    let providers = LLM_CATALOG
        .iter()
        .map(|t| {
            let existing = configured.get(t.name);
            LlmProviderSetupView {
                name: t.name.to_string(),
                label: t.label.to_string(),
                default_base_url: t.default_base_url.to_string(),
                default_model: t.default_model.to_string(),
                key_required: t.key_required,
                configured: existing.is_some(),
                api_key_masked: existing
                    .map(|p| mask_secret(&p.api_key))
                    .unwrap_or_else(|| "（未配置）".into()),
                base_url: existing
                    .map(|p| p.base_url.clone())
                    .unwrap_or_else(|| t.default_base_url.to_string()),
                model: existing
                    .map(|p| p.model.clone())
                    .unwrap_or_else(|| t.default_model.to_string()),
            }
        })
        .collect();
    Ok(ApiResponse::ok(LlmSetupStatus {
        setup_required: cfg.llm_providers.is_empty(),
        default_provider: cfg.default_llm_provider.clone(),
        encryption_ready: encryption_ready(&cfg.encryption_key),
        providers,
    }))
}

#[tauri::command]
pub async fn save_llm_setup(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    payload: SaveLlmSetupPayload,
) -> Result<ApiResponse<LlmSetupStatus>, String> {
    let enc = state.config().encryption_key.clone();
    for cred in &payload.credentials {
        let key = cred.api_key.trim();
        let is_ollama = cred.provider == "ollama";
        if key.is_empty() && !is_ollama {
            continue;
        }
        save_llm_provider(
            &state.db,
            &enc,
            &cred.provider,
            key,
            cred.base_url.as_deref(),
            cred.model.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    }

    let mut cfg = state.config().clone();
    cfg.llm_providers = load_llm_providers(&state.db, &enc).map_err(|e| e.to_string())?;
    if cfg.llm_providers.is_empty() {
        return Ok(ApiResponse::err("请至少配置一个 LLM 提供商"));
    }
    if !payload.default_provider.is_empty() {
        cfg.default_llm_provider = payload.default_provider.clone();
    } else if !cfg.llm_providers.iter().any(|p| p.name == cfg.default_llm_provider) {
        cfg.default_llm_provider = cfg.llm_providers[0].name.clone();
    }

    {
        let mut w = state
            .config_store
            .write()
            .map_err(|e| e.to_string())?;
        *w = cfg.clone();
    }
    sync_llm_to_state(&state);
    restart_runtime_polls(&app, &state).await;

    if let Ok(Some(mut prefs)) = state.db.load_user_preferences() {
        prefs.default_llm_provider = cfg.default_llm_provider.clone();
        let _ = state.db.save_user_preferences(&prefs.normalize());
    }

    get_llm_setup(state).await
}

#[tauri::command]
pub async fn run_client_e2e(
    state: State<'_, Arc<AppState>>,
    symbol: Option<String>,
) -> Result<ApiResponse<crate::testing::E2eSuiteReport>, String> {
    #[cfg(not(debug_assertions))]
    {
        let _ = (state, symbol);
        return Ok(ApiResponse::err("client e2e 仅 debug 构建可用"));
    }
    #[cfg(debug_assertions)]
    {
        let sym = symbol.unwrap_or_else(|| "rb0".into());
        Ok(ApiResponse::ok(
            crate::testing::run_client_e2e_suite(state.inner(), &sym).await,
        ))
    }
}

#[tauri::command]
pub async fn get_user_preferences(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::config::UserPreferences>, String> {
    Ok(ApiResponse::ok(crate::config::UserPreferences::from_config(
        &state.config(),
    )))
}

#[tauri::command]
pub async fn save_user_preferences(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    prefs: crate::config::UserPreferences,
) -> Result<ApiResponse<crate::config::UserPreferences>, String> {
    let prefs = prefs.normalize();
    state
        .db
        .save_user_preferences(&prefs)
        .map_err(|e| e.to_string())?;
    let cfg = crate::services::apply_preferences(&app, &state, prefs).await;
    Ok(ApiResponse::ok(crate::config::UserPreferences::from_config(
        &cfg,
    )))
}

#[tauri::command]
pub async fn reload_config(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<AppSettingsView>, String> {
    let stored = state
        .db
        .load_user_preferences()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| crate::config::UserPreferences::from_config(&state.config()));
    let mut new_cfg = crate::config::Config::load_with_preferences(stored);
    hydrate_config_llm(&state.db, &mut new_cfg).map_err(|e| e.to_string())?;
    crate::services::apply_runtime_config(&state, new_cfg).await;
    sync_llm_to_state(&state);
    crate::services::restart_runtime_polls(&app, &state).await;
    get_settings(state).await
}

#[tauri::command]
pub async fn export_klines_csv(
    state: State<'_, Arc<AppState>>,
    symbol: String,
    interval: String,
    limit: Option<i64>,
) -> Result<ApiResponse<String>, String> {
    let end = Utc::now();
    let start = end - Duration::days(365);
    let klines = state
        .db
        .get_klines(&symbol.to_lowercase(), &interval, start, end, limit.unwrap_or(5000).min(10000))
        .map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(crate::services::klines_to_csv(&klines)))
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

#[tauri::command]
pub async fn import_klines_csv(
    state: State<'_, Arc<AppState>>,
    csv: String,
    symbol: String,
    interval: String,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let klines = crate::services::parse_klines_csv(&csv, &symbol, &interval)?;
    let n = state.db.save_klines(&klines).map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(serde_json::json!({ "imported": n })))
}

#[tauri::command]
pub async fn reclassify_news(
    state: State<'_, Arc<AppState>>,
    news_ids: Vec<String>,
    provider: Option<String>,
    use_llm: Option<bool>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let llm = state.llm_snapshot();
    let count = crate::services::reclassify_news(
        &state.db,
        &llm,
        &news_ids,
        provider.as_deref(),
        use_llm.unwrap_or(true),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(serde_json::json!({ "labels_saved": count })))
}

#[tauri::command]
pub async fn trigger_batch_analysis(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
    trigger: Option<String>,
    provider: Option<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
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
    Ok(ApiResponse::ok(
        state.batch_analysis.get_status().await,
    ))
}

#[tauri::command]
pub async fn probe_ollama(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<bool>, String> {
    let health = state.llm_snapshot().health().await;
    Ok(ApiResponse::ok(*health.get("ollama").unwrap_or(&false)))
}

#[tauri::command]
pub async fn get_status_dashboard(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::StatusDashboardView>, String> {
    let llm_health = state.llm_snapshot().health().await;
    let questdb = crate::db::QuestDbAdapter::new(&state.config().questdb_url);
    let poll = if let Some(p) = state.poll_handle().await {
        let s = p.status().await;
        Some(PollStatus {
            running: s.running,
            interval: s.interval,
            symbols: s.symbols,
            symbol_count: s.symbol_count,
            feed_source: s.feed_source,
        })
    } else {
        None
    };
    let backfill = state.backfill_status.lock().await.clone();
    let schedule = state.schedule_status.lock().await.clone();
    let runtime = crate::models::RuntimeStatusView {
        poll,
        backfill,
        feed_source: if state.feed_source.is_empty() {
            None
        } else {
            Some(state.feed_source.clone())
        },
        schedule,
    };
    Ok(ApiResponse::ok(crate::models::StatusDashboardView {
        runtime,
        llm_health,
        llm_last_errors: state.llm_snapshot().last_errors(),
        questdb_configured: questdb.configured(),
        questdb_online: if questdb.configured() {
            questdb.ping().await
        } else {
            false
        },
        overseas_stub: crate::adapters::list_overseas_symbols(),
        batch_job: state.batch_analysis.get_status().await,
        prompt_version: crate::engine::PROMPT_VERSION.to_string(),
    }))
}

#[tauri::command]
pub async fn get_schedule_status(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::ScheduleStatus>, String> {
    Ok(ApiResponse::ok(state.schedule_status.lock().await.clone()))
}

#[tauri::command]
pub async fn trigger_data_fetch(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::models::DataFetchSummary>, String> {
    let summary = crate::services::run_data_fetch_cycle(state.inner()).await?;
    {
        let mut st = state.schedule_status.lock().await;
        st.last_data_fetch = Some(summary.clone());
    }
    Ok(ApiResponse::ok(summary))
}

#[tauri::command]
pub async fn trigger_comprehensive_analysis(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let symbols: Vec<String> = state.config().watchlist.clone();
    if symbols.is_empty() {
        return Ok(ApiResponse::err("watchlist is empty"));
    }
    let st = Arc::clone(state.inner());
    let status = state.schedule_status.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::services::run_full_cycle(&st, Some(&app), "manual", status).await;
    });
    Ok(ApiResponse::ok(serde_json::json!({
        "started": true,
        "total": symbols.len(),
        "includes_data_fetch": true
    })))
}
