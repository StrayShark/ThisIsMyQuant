use std::sync::Arc;

use tauri::State;

use crate::models::{ApiResponse, CalendarEvent, NewsRecord};
use crate::state::AppState;

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
        match state
            .db
            .get_news_for_symbol(&main, dimension.as_deref(), limit)
        {
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
            let _ = state.db.save_calendar_cache(&cache_key, &filtered, None);
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

pub(crate) fn humanize_calendar_error(raw: &str) -> String {
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
    Ok(ApiResponse::ok(
        serde_json::json!({ "labels_saved": count }),
    ))
}
