use std::collections::HashSet;

use chrono::{Duration, Utc};

use crate::adapters::default_calendar_range_from_today;
use crate::engine::{calendar_filter, sectors};
use crate::error::{AppError, AppResult};
use crate::models::{
    dt_to_iso, parse_dt, AiReportSummary, AiSource, AiSummaryRequest, AnalysisReport,
    CalendarEvent, KLine, NewsItemView, RealtimeQuote, SimOrder, SimPosition,
};
use crate::services::SCHEDULED_CALENDAR_CACHE_KEY;
use crate::state::AppState;

/// 引用式 AI 的上下文数据包。
///
/// 所有字段均来自真实数据库/缓存/外部接口，不随 LLM 虚构。
#[derive(Debug, Clone)]
pub struct AiContextBundle {
    pub quotes: Vec<RealtimeQuote>,
    pub news: Vec<NewsItemView>,
    pub events: Vec<CalendarEvent>,
    pub klines: Vec<KLine>,
    pub positions: Vec<SimPosition>,
    pub orders: Vec<SimOrder>,
    pub reports: Vec<AnalysisReport>,
    pub target_symbol: Option<String>,
    pub target_assets: Vec<String>,
}

pub const DISCLAIMER: &str = "仅供研究与复盘，不构成投资建议。";

impl AiContextBundle {
    pub async fn build(state: &AppState, request: &AiSummaryRequest) -> AppResult<Self> {
        let task_type = request.task_type.as_str();
        let target_symbol = request.target_symbol.as_ref().map(|s| s.to_lowercase());
        let mut target_assets: Vec<String> = request
            .target_assets
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        if let Some(sym) = &target_symbol {
            if !target_assets.contains(sym) {
                target_assets.push(sym.clone());
            }
        }

        // 对于市场级任务，未指定标的时使用核心品种。
        if target_assets.is_empty()
            && matches!(
                task_type,
                "market_summary" | "leaderboard_explain" | "watchlist_summary" | "event_impact"
            )
        {
            target_assets = sectors::core_product_symbols()
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        let quotes = {
            let cache = state.quote_cache.read().unwrap_or_else(|e| e.into_inner());
            if target_assets.is_empty() {
                cache.snapshot(None)
            } else {
                cache.snapshot(Some(&target_assets))
            }
        };

        let news = Self::collect_news(state, &target_assets).await;
        let events = Self::collect_events(state, &target_assets).await?;
        let klines = if let Some(sym) = &target_symbol {
            Self::fetch_klines(state, sym).await
        } else {
            vec![]
        };

        let (positions, orders) =
            if task_type == "position_risk" || task_type == "custom" || !target_assets.is_empty() {
                Self::collect_positions_orders(state, &target_assets)?
            } else {
                (vec![], vec![])
            };

        let reports = state.db.get_reports(None, None, 15).unwrap_or_default();
        let reports: Vec<AnalysisReport> = reports
            .into_iter()
            .filter(|r| {
                target_assets.is_empty()
                    || target_assets
                        .iter()
                        .any(|a| r.symbol.to_lowercase().contains(a))
            })
            .collect();

        Ok(Self {
            quotes,
            news,
            events,
            klines,
            positions,
            orders,
            reports,
            target_symbol: request.target_symbol.clone(),
            target_assets: request.target_assets.clone().unwrap_or_default(),
        })
    }

    async fn collect_news(state: &AppState, target_assets: &[String]) -> Vec<NewsItemView> {
        let mut seen = HashSet::new();
        let mut items = Vec::new();
        if target_assets.is_empty() {
            if let Ok(latest) = state.db.get_latest_news(15) {
                for n in latest {
                    if seen.insert(n.id.clone()) {
                        items.push(n);
                    }
                }
            }
        } else {
            for sym in target_assets.iter().take(20) {
                if let Ok(mut list) = state.db.get_news_for_symbol(&sym.to_uppercase(), None, 5) {
                    for n in list.drain(..) {
                        if seen.insert(n.id.clone()) {
                            items.push(n);
                        }
                    }
                }
            }
            // 兜底：无分类资讯时补充最新市场资讯
            if items.is_empty() {
                if let Ok(latest) = state.db.get_latest_news(10) {
                    for n in latest {
                        if seen.insert(n.id.clone()) {
                            items.push(n);
                        }
                    }
                }
            }
        }
        items
    }

    async fn collect_events(
        state: &AppState,
        target_assets: &[String],
    ) -> AppResult<Vec<CalendarEvent>> {
        // 优先使用已缓存的财经日历（由 data_fetch_cycle 定时写入）。
        let cached = state.db.load_calendar_cache(SCHEDULED_CALENDAR_CACHE_KEY)?;
        let mut events = cached.unwrap_or_default();

        // 缓存为空且金十可用时，实时拉取。
        if events.is_empty() && state.config().jinshi_enabled {
            let (start, end) = default_calendar_range_from_today();
            let guard = state.jinshi.lock().await;
            if guard.is_connected() {
                match guard.fetch_calendar_events(start, end, 3, None).await {
                    Ok(fetched) => {
                        let _ = state.db.save_calendar_cache(
                            SCHEDULED_CALENDAR_CACHE_KEY,
                            &fetched,
                            None,
                        );
                        events = fetched;
                    }
                    Err(e) => log::warn!("ai_context calendar fetch: {e}"),
                }
            }
        }

        // 当存在单一品种时，按板块维度进一步过滤。
        if target_assets.len() == 1 {
            let sym = &target_assets[0];
            let sector_ctx = sectors::sector_context(sym);
            let sector_code = sector_ctx["code"].as_str().unwrap_or("");
            let dimension_codes = crate::engine::dimensions::sector_dimension_codes(sector_code);
            events = calendar_filter::filter_for_analysis(events, sector_code, &dimension_codes);
        }

        Ok(events.into_iter().take(20).collect())
    }

    async fn fetch_klines(state: &AppState, symbol: &str) -> Vec<KLine> {
        let sym = symbol.to_lowercase();
        let end = Utc::now();
        let start = end - Duration::days(120);
        let limit = 120;

        let mut klines = state
            .db
            .get_klines(&sym, "1d", start, end, limit)
            .unwrap_or_default();

        let needs_fetch = klines.is_empty() || crate::services::is_daily_klines_stale(&klines);
        if needs_fetch {
            match state.akshare.get_history(&sym, "1d", start, end).await {
                Ok(mut fetched) if !fetched.is_empty() => {
                    if fetched.len() > 60 {
                        fetched = fetched.split_off(fetched.len() - 60);
                    }
                    let _ = state.db.save_klines(&fetched);
                    klines = fetched;
                }
                _ => {}
            }
        }

        if let Some(forming) = state
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .forming_daily(&sym)
        {
            crate::services::merge_forming_daily(&mut klines, &forming);
        }
        klines
    }

    fn collect_positions_orders(
        state: &AppState,
        target_assets: &[String],
    ) -> AppResult<(Vec<SimPosition>, Vec<SimOrder>)> {
        let mut positions = state.sim_trading.list_positions(None)?;
        if !target_assets.is_empty() {
            positions.retain(|p| target_assets.contains(&p.symbol.to_lowercase()));
        }
        let mut orders = state.sim_trading.list_orders(None, None, 50)?;
        if !target_assets.is_empty() {
            orders.retain(|o| target_assets.contains(&o.symbol.to_lowercase()));
        }
        Ok((positions, orders))
    }

    /// 计算上下文中最新数据的时间点。
    pub fn latest_data_time(&self) -> Option<String> {
        let mut latest: Option<chrono::DateTime<Utc>> = None;
        for q in &self.quotes {
            if let Some(dt) = parse_dt(&q.timestamp) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        for n in &self.news {
            if let Some(dt) = parse_dt(&n.display_time) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        for e in &self.events {
            if let Some(dt) = parse_dt(&e.pub_time) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        for k in &self.klines {
            if let Some(dt) = parse_dt(&k.start_time) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        for p in &self.positions {
            if let Some(dt) = parse_dt(&p.updated_at) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        for o in &self.orders {
            if let Some(dt) = parse_dt(&o.updated_at) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        for r in &self.reports {
            if let Some(dt) = parse_dt(&r.created_at) {
                latest = Some(latest.map_or(dt, |l| l.max(dt)));
            }
        }
        latest.map(dt_to_iso)
    }

    /// 构建来源 ID 索引，用于校验 LLM 返回的引用不来自虚构数据。
    pub fn valid_source_ids(&self) -> HashSet<String> {
        let mut ids = HashSet::new();
        for q in &self.quotes {
            ids.insert(q.symbol.clone());
        }
        for n in &self.news {
            ids.insert(n.id.clone());
        }
        for e in &self.events {
            ids.insert(e.id.clone());
        }
        if let Some(sym) = self.target_symbol.as_ref() {
            ids.insert(format!("kline:{}", sym.to_lowercase()));
        }
        for p in &self.positions {
            ids.insert(position_source_id(p));
        }
        for o in &self.orders {
            ids.insert(o.id.clone());
        }
        for r in &self.reports {
            ids.insert(r.id.clone());
        }
        ids
    }
}

fn position_source_id(p: &SimPosition) -> String {
    format!("{}:{}:{}", p.account_id, p.symbol, p.position_side)
}

/// 生成带引用目录和输出格式约束的 Prompt。
pub fn render_ai_prompt(
    bundle: &AiContextBundle,
    task_type: &str,
    custom_prompt: Option<&str>,
) -> String {
    let task_label = task_label(task_type);
    let symbol_line = bundle
        .target_symbol
        .as_ref()
        .map(|s| format!("目标品种：{}", s))
        .unwrap_or_default();
    let assets_line = if bundle.target_assets.is_empty() {
        String::new()
    } else {
        format!("关注标的：{}", bundle.target_assets.join(", "))
    };

    let catalog = render_source_catalog(bundle);

    let custom_block = custom_prompt
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("\n## 用户额外要求\n{}\n", s))
        .unwrap_or_default();

    format!(
        "你是一位金融市场研究助手。请基于以下真实数据完成「{task_label}」，所有结论必须引用下方列出的来源。\n\
        {symbol_line}\n\
        {assets_line}\n\n\
        ## 实时行情\n\
        {}\n\n\
        ## 资讯\n\
        {}\n\n\
        ## 财经日历\n\
        {}\n\n\
        ## 日K线数据\n\
        {}\n\n\
        ## 模拟持仓\n\
        {}\n\n\
        ## 模拟订单\n\
        {}\n\n\
        ## 历史报告\n\
        {}\n\
        {custom_block}\n\
        ## 来源目录（引用时必须使用以下 id）\n\
        {catalog}\n\n\
        ## 输出格式要求\n\
        优先输出一个 JSON 对象，字段如下：\n\
        - content：Markdown 格式的分析正文，必须包含分点论述。\n\
        - sources：引用列表，每项包含 source_type（quote/news/calendar/kline/position/order/report）、id、title、display_time。id 必须来自上方来源目录。\n\
        - data_date：数据截止日期（ISO 字符串）。\n\
        - disclaimer：固定为「{DISCLAIMER}」。\n\
        如果无法输出 JSON，请保证正文中明显列出引用来源，并单独在最后输出免责声明：{DISCLAIMER}\n",
        render_quotes(bundle),
        render_news(bundle),
        render_events(bundle),
        render_klines(bundle),
        render_positions(bundle),
        render_orders(bundle),
        render_reports(bundle),
    )
}

fn task_label(task_type: &str) -> &str {
    match task_type {
        "market_summary" => "市场综述",
        "leaderboard_explain" => "榜单解读",
        "asset_brief" => "品种速览",
        "watchlist_summary" => "自选摘要",
        "position_risk" => "持仓风险分析",
        "event_impact" => "事件影响评估",
        "custom" => "自定义分析",
        _ => "AI 分析",
    }
}

fn render_quotes(bundle: &AiContextBundle) -> String {
    if bundle.quotes.is_empty() {
        return "（暂无实时行情）".into();
    }
    bundle
        .quotes
        .iter()
        .map(|q| {
            format!(
                "- [{}] {} 最新价={:.2} 涨跌幅={:.2}% 时间={}",
                q.symbol, q.symbol, q.last_price, q.change_pct, q.timestamp
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_news(bundle: &AiContextBundle) -> String {
    if bundle.news.is_empty() {
        return "（暂无资讯）".into();
    }
    bundle
        .news
        .iter()
        .take(20)
        .map(|n| {
            let summary = n.summary.chars().take(120).collect::<String>();
            format!(
                "- [{}] [{}] {} {} — {}",
                n.id, n.display_time, n.source, n.title, summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_events(bundle: &AiContextBundle) -> String {
    if bundle.events.is_empty() {
        return "（暂无财经日历事件）".into();
    }
    bundle
        .events
        .iter()
        .take(20)
        .map(|e| {
            format!(
                "- [{}] {} {} ★{} {} | 前值={} 预期={} 公布={}",
                e.id,
                e.pub_time,
                e.country,
                e.star,
                e.name,
                e.previous.as_deref().unwrap_or("-"),
                e.consensus.as_deref().unwrap_or("-"),
                e.actual.as_deref().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_klines(bundle: &AiContextBundle) -> String {
    if bundle.klines.is_empty() {
        return "（无日K线数据）".into();
    }
    let count = bundle.klines.len();
    let first = bundle
        .klines
        .first()
        .map(|k| k.start_time.as_str())
        .unwrap_or("-");
    let last = bundle
        .klines
        .last()
        .map(|k| k.start_time.as_str())
        .unwrap_or("-");
    let close = bundle.klines.last().map(|k| k.close).unwrap_or(0.0);
    format!(
        "- 共 {count} 根日K，区间 {first} ~ {last}，最新收盘 {close:.2}\n\
         - 最近 5 根收盘：{}",
        bundle
            .klines
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|k| format!("{:.2}", k.close))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_positions(bundle: &AiContextBundle) -> String {
    if bundle.positions.is_empty() {
        return "（无模拟持仓）".into();
    }
    bundle
        .positions
        .iter()
        .map(|p| {
            format!(
                "- [{}] {} 账户={} 方向={} 数量={} 均价={:.2} 浮盈={:.2}",
                position_source_id(p),
                p.symbol,
                p.account_id,
                p.position_side,
                p.total_qty,
                p.avg_price,
                p.unrealized_pnl
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_orders(bundle: &AiContextBundle) -> String {
    if bundle.orders.is_empty() {
        return "（无模拟订单）".into();
    }
    bundle
        .orders
        .iter()
        .map(|o| {
            format!(
                "- [{}] {} 方向={} 类型={} 状态={} 数量={}/{}",
                o.id, o.symbol, o.side, o.order_type, o.status, o.filled_quantity, o.quantity
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_reports(bundle: &AiContextBundle) -> String {
    if bundle.reports.is_empty() {
        return "（无历史报告）".into();
    }
    bundle
        .reports
        .iter()
        .take(10)
        .map(|r| {
            let title = if r.context_summary.is_empty() {
                r.content.chars().take(60).collect::<String>()
            } else {
                r.context_summary.clone()
            };
            format!(
                "- [{}] [{}] {} {} {}",
                r.id, r.trigger, r.symbol, r.created_at, title
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_source_catalog(bundle: &AiContextBundle) -> String {
    let mut lines = Vec::new();
    for q in &bundle.quotes {
        lines.push(format!(
            "- quote | id={} | title={} 实时行情 | time={}",
            q.symbol, q.symbol, q.timestamp
        ));
    }
    for n in &bundle.news {
        lines.push(format!(
            "- news | id={} | title={} | time={}",
            n.id,
            escape_catalog(&n.title),
            n.display_time
        ));
    }
    for e in &bundle.events {
        lines.push(format!(
            "- calendar | id={} | title={} | time={}",
            e.id,
            escape_catalog(&e.name),
            e.pub_time
        ));
    }
    if let Some(sym) = bundle.target_symbol.as_ref() {
        let time = bundle
            .klines
            .last()
            .map(|k| k.start_time.as_str())
            .unwrap_or("-");
        lines.push(format!(
            "- kline | id=kline:{} | title={} 日K线 | time={}",
            sym.to_lowercase(),
            sym,
            time
        ));
    }
    for p in &bundle.positions {
        lines.push(format!(
            "- position | id={} | title={} 持仓 | time={}",
            position_source_id(p),
            p.symbol,
            p.updated_at
        ));
    }
    for o in &bundle.orders {
        lines.push(format!(
            "- order | id={} | title={} 订单 | time={}",
            o.id, o.symbol, o.updated_at
        ));
    }
    for r in &bundle.reports {
        lines.push(format!(
            "- report | id={} | title={} | time={}",
            r.id,
            escape_catalog(&r.context_summary),
            r.created_at
        ));
    }
    if lines.is_empty() {
        "（当前无可用来源）".into()
    } else {
        lines.join("\n")
    }
}

fn escape_catalog(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// 从 LLM 输出中提取结构化内容。
///
/// - 优先解析 ```json 代码块或 JSON 对象。
/// - 解析失败时，将整个输出作为正文。
/// - 对来源 id 进行白名单校验，过滤掉不在上下文中的虚构引用。
/// - 自动追加免责声明。
pub fn parse_ai_report_output(
    text: &str,
    bundle: &AiContextBundle,
    provider: &str,
    task_type: &str,
    target_symbol: Option<String>,
) -> AiReportSummary {
    let valid_ids = bundle.valid_source_ids();
    let data_date = bundle.latest_data_time();

    let (mut content, mut sources, data_date) = if let Some(json_str) = extract_json_block(text) {
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(v) => {
                let content = v
                    .get("content")
                    .and_then(|c| c.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| text.to_string());
                let sources: Vec<AiSource> = v
                    .get("sources")
                    .and_then(|s| serde_json::from_value(s.clone()).ok())
                    .unwrap_or_default();
                let data_date = v
                    .get("data_date")
                    .and_then(|d| d.as_str())
                    .map(String::from)
                    .or(data_date);
                (content, sources, data_date)
            }
            Err(_) => (text.to_string(), Vec::new(), data_date),
        }
    } else {
        (text.to_string(), Vec::new(), data_date)
    };

    // 过滤虚构来源。
    sources.retain(|s| {
        s.id.as_ref()
            .map(|id| valid_ids.contains(id))
            .unwrap_or(false)
    });

    // 去重：同一个 id 只保留一次。
    let mut seen = HashSet::new();
    sources.retain(|s| {
        s.id.as_ref()
            .map(|id| seen.insert(id.clone()))
            .unwrap_or(false)
    });

    if sources.is_empty() {
        if let Some(source) = fallback_source(bundle) {
            sources.push(source);
        }
    }

    // 保证免责声明。
    if !content.contains("仅供研究与复盘") && !content.contains("不构成投资建议") {
        content.push_str("\n\n> ");
        content.push_str(DISCLAIMER);
    }

    AiReportSummary {
        id: uuid::Uuid::new_v4().to_string(),
        task_type: task_type.to_string(),
        target_symbol,
        content,
        sources,
        data_date,
        disclaimer: DISCLAIMER.to_string(),
        provider: provider.to_string(),
        created_at: Utc::now().to_rfc3339(),
    }
}

fn fallback_source(bundle: &AiContextBundle) -> Option<AiSource> {
    if let Some(q) = bundle.quotes.first() {
        return Some(AiSource {
            source_type: "quote".to_string(),
            id: Some(q.symbol.clone()),
            title: format!("{} 实时行情", q.symbol),
            display_time: Some(q.timestamp.clone()),
            url: None,
        });
    }
    if let Some(n) = bundle.news.first() {
        return Some(AiSource {
            source_type: "news".to_string(),
            id: Some(n.id.clone()),
            title: n.title.clone(),
            display_time: Some(n.display_time.clone()),
            url: None,
        });
    }
    if let Some(e) = bundle.events.first() {
        return Some(AiSource {
            source_type: "calendar".to_string(),
            id: Some(e.id.clone()),
            title: e.name.clone(),
            display_time: Some(e.pub_time.clone()),
            url: None,
        });
    }
    if let Some(sym) = bundle.target_symbol.as_ref() {
        if let Some(k) = bundle.klines.last() {
            return Some(AiSource {
                source_type: "kline".to_string(),
                id: Some(format!("kline:{}", sym.to_lowercase())),
                title: format!("{sym} 日K线"),
                display_time: Some(k.start_time.clone()),
                url: None,
            });
        }
    }
    if let Some(p) = bundle.positions.first() {
        return Some(AiSource {
            source_type: "position".to_string(),
            id: Some(position_source_id(p)),
            title: format!("{} 模拟持仓", p.symbol),
            display_time: Some(p.updated_at.clone()),
            url: None,
        });
    }
    if let Some(o) = bundle.orders.first() {
        return Some(AiSource {
            source_type: "order".to_string(),
            id: Some(o.id.clone()),
            title: format!("{} 模拟订单", o.symbol),
            display_time: Some(o.updated_at.clone()),
            url: None,
        });
    }
    if let Some(r) = bundle.reports.first() {
        return Some(AiSource {
            source_type: "report".to_string(),
            id: Some(r.id.clone()),
            title: r.context_summary.clone(),
            display_time: Some(r.created_at.clone()),
            url: None,
        });
    }
    None
}

fn extract_json_block(text: &str) -> Option<String> {
    if let Some(start) = text.find("```json") {
        let sub = &text[start + 7..];
        if let Some(end) = sub.find("```") {
            return Some(sub[..end].trim().to_string());
        }
    }
    if let Some(start) = text.find("```") {
        let sub = &text[start + 3..];
        if let Some(end) = sub.find("```") {
            let trimmed = sub[..end].trim();
            if trimmed.starts_with('{') {
                return Some(trimmed.to_string());
            }
        }
    }
    let trimmed = text.trim();
    if trimmed.starts_with('{') {
        Some(trimmed.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_bundle() -> AiContextBundle {
        AiContextBundle {
            quotes: vec![],
            news: vec![],
            events: vec![],
            klines: vec![],
            positions: vec![],
            orders: vec![],
            reports: vec![],
            target_symbol: Some("rb0".to_string()),
            target_assets: vec!["rb0".to_string()],
        }
    }

    #[test]
    fn parse_ai_report_output_adds_fallback_source() {
        let mut bundle = empty_bundle();
        bundle.klines.push(KLine {
            symbol: "rb0".to_string(),
            interval: "1d".to_string(),
            start_time: "2026-01-02T00:00:00Z".to_string(),
            open: 3500.0,
            high: 3550.0,
            low: 3490.0,
            close: 3520.0,
            volume: 1000,
            turnover: 3_520_000.0,
        });

        let summary = parse_ai_report_output(
            "{\"content\":\"螺纹钢小幅走强。\",\"sources\":[],\"data_date\":\"2026-01-02\",\"disclaimer\":\"仅供研究与复盘，不构成投资建议。\"}",
            &bundle,
            "test",
            "asset_brief",
            Some("rb0".to_string()),
        );
        assert_eq!(summary.sources.len(), 1);
        assert_eq!(summary.sources[0].source_type, "kline");
        assert!(summary.disclaimer.contains("不构成投资建议"));
    }
}

// 允许通过错误类型构造返回结果。
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
