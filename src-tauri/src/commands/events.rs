use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, Utc};
use tauri::State;

use crate::engine::sectors::{all_sectors, get_sector_by_symbol};
use crate::models::{
    parse_dt, ApiResponse, CalendarEvent, MarketEvent, MarketEventListResult, MarketEventQuery,
    NewsItemView,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_market_events(
    state: State<'_, Arc<AppState>>,
    query: MarketEventQuery,
) -> Result<ApiResponse<MarketEventListResult>, String> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200) as usize;
    let symbol_filter = query.symbol.as_ref().map(|s| s.to_uppercase());

    let mut events: Vec<MarketEvent> = Vec::new();

    // 1. 金十资讯
    let news_items = fetch_news_for_query(state.clone(), symbol_filter.as_deref(), limit).await;
    events.extend(news_to_events(news_items));

    // 2. 财经日历
    if query.source.as_deref().unwrap_or("all") != "jin10" {
        let calendar_items = fetch_calendar_for_query(state.clone(), &query).await;
        events.extend(calendar_to_events(calendar_items));
    }

    // 3. A 股公告：优先基于本地 A 股目录/行业生成研究提醒，避免事件流空白
    if should_include_source(&query.source, "announcement") {
        events.extend(stock_announcements(state.inner(), symbol_filter.as_deref()));
    }

    // 4. 财报日历：基于本地财务报告期生成可追踪事件
    if should_include_source(&query.source, "earnings") {
        events.extend(stock_earnings(state.inner(), symbol_filter.as_deref()));
    }

    // 5. 产业事件：从新闻中按板块关键词提取
    if should_include_source(&query.source, "industry") {
        let industry_news = if symbol_filter.is_some() {
            fetch_news_for_query(state.clone(), symbol_filter.as_deref(), limit).await
        } else {
            fetch_news_for_query(state.clone(), None, limit).await
        };
        events.extend(extract_industry_events(industry_news));
    }

    // 应用筛选
    let filtered: Vec<MarketEvent> = events
        .into_iter()
        .filter(|e| event_matches(e, &query, symbol_filter.as_deref()))
        .collect();

    // 按 display_time 降序
    let mut sorted = filtered;
    sorted.sort_by(|a, b| {
        let ta = parse_dt(&a.display_time);
        let tb = parse_dt(&b.display_time);
        tb.cmp(&ta)
    });

    // 分页/截断
    let total = sorted.len() as i64;
    sorted.truncate(limit);

    // 统计
    let mut by_source: HashMap<String, i64> = HashMap::new();
    for e in &sorted {
        *by_source.entry(e.source.clone()).or_insert(0) += 1;
    }

    Ok(ApiResponse::ok(MarketEventListResult {
        events: sorted,
        total,
        by_source,
    }))
}

async fn fetch_news_for_query(
    state: State<'_, Arc<AppState>>,
    symbol: Option<&str>,
    limit: usize,
) -> Vec<NewsItemView> {
    let db_limit = limit as i64;
    let symbol_ref = symbol.map(|s| s.to_uppercase());

    // 优先从数据库读取
    let db_items = if let Some(ref sym) = symbol_ref {
        state
            .db
            .get_news_for_symbol(sym, None, db_limit)
            .unwrap_or_default()
    } else {
        state.db.get_latest_news(db_limit).unwrap_or_default()
    };

    if !db_items.is_empty() {
        return db_items;
    }

    // 数据库为空且金十在线时，尝试实时拉取
    if !state.jinshi.lock().await.is_connected() {
        return vec![];
    }
    let jinshi = state.jinshi.lock().await;
    let result = if let Some(ref sym) = symbol_ref {
        jinshi.fetch_for_symbol(sym, limit).await
    } else {
        jinshi.fetch_latest(limit).await
    };
    drop(jinshi);

    match result {
        Ok(items) => items
            .into_iter()
            .map(|n| NewsItemView {
                id: String::new(),
                title: n.title,
                summary: n.summary,
                source: n.source,
                category_id: n.category_id,
                display_time: n.display_time,
                url: n.url,
                classifications: vec![],
            })
            .collect(),
        Err(e) => {
            log::warn!("list_market_events fetch news: {e}");
            vec![]
        }
    }
}

async fn fetch_calendar_for_query(
    state: State<'_, Arc<AppState>>,
    query: &MarketEventQuery,
) -> Vec<CalendarEvent> {
    if !state.config().jinshi_enabled {
        return vec![];
    }

    let (default_start, default_end) = crate::adapters::default_calendar_range_from_today();
    let start_date = query
        .start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(default_start - Duration::days(3));
    let end_date = query
        .end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(default_end);

    let min_star: u8 = 1;

    match state
        .jinshi
        .lock()
        .await
        .fetch_calendar_events(start_date, end_date, min_star, None)
        .await
    {
        Ok(events) => events,
        Err(e) => {
            log::warn!("list_market_events fetch calendar: {e}");
            vec![]
        }
    }
}

pub fn news_to_events(news: Vec<NewsItemView>) -> Vec<MarketEvent> {
    news.into_iter()
        .map(|n| {
            let symbols: Vec<String> = n
                .classifications
                .iter()
                .map(|c| c.symbol.to_uppercase())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let sectors: Vec<String> = symbols
                .iter()
                .filter_map(|s| get_sector_by_symbol(s).map(|sec| sec.name.clone()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let importance = infer_news_importance(&n.title, &n.summary);
            let event_type = infer_news_event_type(&n.title, &n.summary, &n.classifications);
            let direction = infer_direction(&n.title);
            let created_at = Utc::now().to_rfc3339();

            MarketEvent {
                id: if n.id.is_empty() {
                    uuid::Uuid::new_v4().to_string()
                } else {
                    n.id
                },
                title: n.title,
                source: "jin10".into(),
                source_id: None,
                source_url: Some(n.url),
                display_time: n.display_time,
                importance,
                event_type,
                affected_symbols: symbols,
                affected_sectors: sectors,
                direction,
                summary: Some(n.summary),
                created_at,
            }
        })
        .collect()
}

pub fn calendar_to_events(events: Vec<CalendarEvent>) -> Vec<MarketEvent> {
    events
        .into_iter()
        .map(|e| {
            let symbols: Vec<String> = e
                .affect
                .as_ref()
                .map(|a| {
                    a.split(&[',', '，', ';', '；'][..])
                        .map(|s| s.trim().to_uppercase())
                        .filter(|s| !s.is_empty())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default();
            let sectors: Vec<String> = symbols
                .iter()
                .filter_map(|s| get_sector_by_symbol(s).map(|sec| sec.name.clone()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let importance = calendar_star_to_importance(e.star);
            let event_type = infer_calendar_event_type(&e.name, &e.country);
            let direction = infer_direction(&e.name);
            let created_at = Utc::now().to_rfc3339();
            let summary = build_calendar_summary(&e);

            MarketEvent {
                id: if e.id.is_empty() {
                    uuid::Uuid::new_v4().to_string()
                } else {
                    e.id.clone()
                },
                title: format!("{} {}", e.country, e.name),
                source: "calendar".into(),
                source_id: Some(e.id),
                source_url: None,
                display_time: e.pub_time,
                importance,
                event_type,
                affected_symbols: symbols,
                affected_sectors: sectors,
                direction,
                summary,
                created_at,
            }
        })
        .collect()
}

fn build_calendar_summary(e: &CalendarEvent) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(v) = &e.previous {
        parts.push(format!("前值: {v}"));
    }
    if let Some(v) = &e.consensus {
        parts.push(format!("预期: {v}"));
    }
    if let Some(v) = &e.actual {
        parts.push(format!("公布: {v}"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" | "))
    }
}

pub fn infer_direction(title: &str) -> Option<String> {
    let t = title.to_lowercase();
    let bullish = [
        "上涨",
        "涨",
        "利好",
        "强劲",
        "超预期",
        "宽松",
        "降息",
        "扩张",
        "支持",
        "反弹",
        "走高",
        "偏多",
        "bullish",
        "surge",
        "rally",
        "gain",
    ];
    let bearish = [
        "下跌",
        "跌",
        "利空",
        "疲软",
        "不及预期",
        "收紧",
        "加息",
        "衰退",
        "压制",
        "走低",
        "偏空",
        "bearish",
        "drop",
        "fall",
        "plunge",
        "decline",
    ];
    let has_bull = bullish.iter().any(|k| t.contains(&k.to_lowercase()));
    let has_bear = bearish.iter().any(|k| t.contains(&k.to_lowercase()));
    match (has_bull, has_bear) {
        (true, false) => Some("bullish".into()),
        (false, true) => Some("bearish".into()),
        _ => Some("neutral".into()),
    }
}

fn calendar_star_to_importance(star: u8) -> String {
    match star {
        4..=5 => "high",
        3 => "medium",
        _ => "low",
    }
    .into()
}

fn infer_news_importance(title: &str, summary: &str) -> String {
    let text = format!("{} {}", title, summary).to_lowercase();
    let high_keywords = [
        "重要", "重磅", "突发", "重大", "紧急", "震惊", "剧变", "break",
    ];
    if high_keywords
        .iter()
        .any(|k| text.contains(&k.to_lowercase()))
    {
        "high".into()
    } else {
        "medium".into()
    }
}

fn infer_news_event_type(
    title: &str,
    summary: &str,
    classifications: &[crate::models::NewsClassificationView],
) -> String {
    let text = format!("{} {}", title, summary).to_lowercase();

    // 根据维度代码判断
    for c in classifications {
        match c.dimension_code.as_str() {
            "demand" | "inventory" | "domestic_supply" | "overseas_upstream" => {
                return "industry".into();
            }
            "macro" => return "macro".into(),
            "overseas_finance" => return "macro".into(),
            _ => {}
        }
    }

    if text.contains("cpi")
        || text.contains("ppi")
        || text.contains("pmi")
        || text.contains("gdp")
        || text.contains("非农")
        || text.contains("利率决议")
        || text.contains("央行")
    {
        "macro".into()
    } else if text.contains("财报") || text.contains("业绩") || text.contains("净利润") {
        "earnings".into()
    } else if text.contains("公告") || text.contains("披露") {
        "announcement".into()
    } else {
        "industry".into()
    }
}

fn infer_calendar_event_type(name: &str, country: &str) -> String {
    let text = format!("{} {}", country, name).to_lowercase();
    if text.contains("cpi")
        || text.contains("ppi")
        || text.contains("pmi")
        || text.contains("gdp")
        || text.contains("非农")
        || text.contains("就业")
        || text.contains("利率决议")
        || text.contains("央行")
        || text.contains("fed")
    {
        "macro".into()
    } else {
        "data".into()
    }
}

fn extract_industry_events(news: Vec<NewsItemView>) -> Vec<MarketEvent> {
    let sectors = all_sectors();
    let mut out = Vec::new();
    for n in news {
        let text = format!("{} {}", n.title, n.summary).to_lowercase();
        for sector in sectors {
            if sector
                .news_keywords
                .iter()
                .any(|kw| text.contains(&kw.to_lowercase()))
            {
                let symbols: Vec<String> = sector
                    .products
                    .iter()
                    .map(|p| p.symbol.to_uppercase())
                    .collect();
                out.push(MarketEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: format!("{}板块动态: {}", sector.name, n.title),
                    source: "industry".into(),
                    source_id: if n.id.is_empty() {
                        None
                    } else {
                        Some(n.id.clone())
                    },
                    source_url: Some(n.url.clone()),
                    display_time: n.display_time.clone(),
                    importance: infer_news_importance(&n.title, &n.summary),
                    event_type: "industry".into(),
                    affected_symbols: symbols.clone(),
                    affected_sectors: vec![sector.name.clone()],
                    direction: infer_direction(&n.title),
                    summary: Some(n.summary.clone()),
                    created_at: Utc::now().to_rfc3339(),
                });
            }
        }
    }
    out
}

fn stock_announcements(state: &Arc<AppState>, symbol_filter: Option<&str>) -> Vec<MarketEvent> {
    let now = Utc::now();
    let symbols = stock_event_symbols(state, symbol_filter);
    symbols
        .into_iter()
        .take(20)
        .map(|s| MarketEvent {
            id: format!("announcement-{}-{}", s.ts_code, now.format("%Y%m%d")),
            title: format!("{} 公告跟踪：关注公司公告、交易所问询与经营事项", s.name),
            source: "announcement".into(),
            source_id: Some(s.ts_code.clone()),
            source_url: None,
            display_time: (now - Duration::hours(2)).to_rfc3339(),
            importance: "medium".into(),
            event_type: "announcement".into(),
            affected_symbols: vec![s.ts_code.clone()],
            affected_sectors: s.industry.clone().into_iter().collect(),
            direction: Some("neutral".into()),
            summary: Some(format!(
                "本地 A 股公告提醒：{}({}) 所属行业 {}，建议在个股详情页结合行情、财务与最新公告复核。",
                s.name,
                s.ts_code,
                s.industry.clone().unwrap_or_else(|| "未分类".into())
            )),
            created_at: now.to_rfc3339(),
        })
        .collect()
}

fn stock_earnings(state: &Arc<AppState>, symbol_filter: Option<&str>) -> Vec<MarketEvent> {
    let now = Utc::now();
    let symbols = stock_event_symbols(state, symbol_filter);
    symbols
        .into_iter()
        .take(20)
        .map(|s| {
            let fin = state
                .db
                .get_stock_financial_metrics(&s.ts_code, 1)
                .ok()
                .and_then(|items| items.into_iter().next());
            let period = fin
                .as_ref()
                .map(|f| f.report_period.clone())
                .unwrap_or_else(|| infer_latest_report_period(now));
            let profit = fin
                .as_ref()
                .and_then(|f| f.net_profit_yoy)
                .map(|v| format!("净利同比 {v:+.1}%"))
                .unwrap_or_else(|| "等待财务数据同步".into());
            MarketEvent {
                id: format!("earnings-{}-{period}", s.ts_code),
                title: format!(
                    "{} 财报跟踪：{} 报告期",
                    s.name,
                    format_report_period(&period)
                ),
                source: "earnings".into(),
                source_id: Some(s.ts_code.clone()),
                source_url: None,
                display_time: (now - Duration::hours(6)).to_rfc3339(),
                importance: "medium".into(),
                event_type: "earnings".into(),
                affected_symbols: vec![s.ts_code.clone()],
                affected_sectors: s.industry.clone().into_iter().collect(),
                direction: fin
                    .as_ref()
                    .and_then(|f| f.net_profit_yoy)
                    .map(|v| if v >= 0.0 { "bullish" } else { "bearish" }.to_string())
                    .or_else(|| Some("neutral".into())),
                summary: Some(format!(
                    "{}({}) 最新报告期 {}，{}。来源：本地财务指标缓存/降级摘要。",
                    s.name,
                    s.ts_code,
                    format_report_period(&period),
                    profit
                )),
                created_at: now.to_rfc3339(),
            }
        })
        .collect()
}

fn stock_event_symbols(
    state: &Arc<AppState>,
    symbol_filter: Option<&str>,
) -> Vec<crate::models::StockSymbol> {
    if let Some(sym) = symbol_filter {
        if let Ok(Some(symbol)) = state.db.get_stock_symbol(sym) {
            return vec![symbol];
        }
    }
    let symbols = state
        .db
        .list_stock_symbols(None, None, 50)
        .unwrap_or_default();
    if symbols.is_empty() {
        fallback_stock_event_symbols()
    } else {
        symbols
    }
}

fn fallback_stock_event_symbols() -> Vec<crate::models::StockSymbol> {
    let now = Utc::now().to_rfc3339();
    [
        ("600000.SH", "600000", "浦发银行", "SH", "银行"),
        ("600519.SH", "600519", "贵州茅台", "SH", "白酒"),
        ("000001.SZ", "000001", "平安银行", "SZ", "银行"),
        ("300750.SZ", "300750", "宁德时代", "SZ", "电池"),
    ]
    .into_iter()
    .map(
        |(ts_code, symbol, name, exchange, industry)| crate::models::StockSymbol {
            ts_code: ts_code.into(),
            symbol: symbol.into(),
            name: name.into(),
            exchange: exchange.into(),
            market: Some("A股".into()),
            industry: Some(industry.into()),
            list_date: None,
            status: "active".into(),
            source: "fallback".into(),
            updated_at: now.clone(),
        },
    )
    .collect()
}

fn infer_latest_report_period(now: chrono::DateTime<Utc>) -> String {
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2026);
    match now.format("%m").to_string().parse::<u32>().unwrap_or(12) {
        1..=3 => format!("{}1231", year - 1),
        4..=6 => format!("{year}0331"),
        7..=9 => format!("{year}0630"),
        _ => format!("{year}0930"),
    }
}

fn format_report_period(period: &str) -> String {
    if period.len() == 8 {
        format!("{}-{}-{}", &period[0..4], &period[4..6], &period[6..8])
    } else {
        period.to_string()
    }
}

fn should_include_source(source_filter: &Option<String>, target: &str) -> bool {
    match source_filter.as_deref() {
        None | Some("all") | Some("") => true,
        Some(s) => s.eq_ignore_ascii_case(target),
    }
}

fn event_matches(e: &MarketEvent, query: &MarketEventQuery, symbol_filter: Option<&str>) -> bool {
    // source
    if let Some(s) = query.source.as_deref() {
        if !s.eq_ignore_ascii_case("all") && !e.source.eq_ignore_ascii_case(s) {
            return false;
        }
    }

    // symbol
    if let Some(sym) = symbol_filter {
        if !e
            .affected_symbols
            .iter()
            .any(|s| s.eq_ignore_ascii_case(sym))
            && !e.title.to_uppercase().contains(&sym.to_uppercase())
        {
            return false;
        }
    }

    // sector
    if let Some(sec) = query.sector.as_deref() {
        let sec_lower = sec.to_lowercase();
        let in_sectors = e
            .affected_sectors
            .iter()
            .any(|s| s.to_lowercase().contains(&sec_lower));
        let in_title = e.title.to_lowercase().contains(&sec_lower);
        if !in_sectors && !in_title {
            return false;
        }
    }

    // importance
    if let Some(imp) = query.importance.as_deref() {
        if !imp.eq_ignore_ascii_case("all") && !e.importance.eq_ignore_ascii_case(imp) {
            return false;
        }
    }

    // event_type
    if let Some(et) = query.event_type.as_deref() {
        let et_lower = et.to_lowercase();
        if et_lower != "all" && !e.event_type.to_lowercase().contains(&et_lower) {
            return false;
        }
    }

    // start / end
    if let Some(event_dt) = parse_dt(&e.display_time) {
        if let Some(start_str) = query.start.as_deref() {
            if let Some(start_dt) = parse_dt(start_str) {
                if event_dt < start_dt {
                    return false;
                }
            }
        }
        if let Some(end_str) = query.end.as_deref() {
            if let Some(end_dt) = parse_dt(end_str) {
                if event_dt > end_dt {
                    return false;
                }
            }
        }
    }

    true
}
