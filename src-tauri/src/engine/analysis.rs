use chrono::{Duration, Utc};

use crate::adapters::{default_calendar_range_from_today, AkshareClient, JinshiClient};
use crate::db::Database;
use crate::engine::{calendar_filter, dimensions, indicator, sectors};

/// LLM 分析 prompt 版本，落库于 reports.prompt_version。
pub const PROMPT_VERSION: &str = "v4";

pub const SYSTEM_PROMPT: &str = "你是一名专业的期货市场分析师，擅长结合技术面与基本面进行走势研判。\
技术分析默认基于**日 K 线**（1d）周期：趋势、均线、MACD、支撑阻力均优先从日 K 视角解读；\
勿混用分钟级走势替代日 K 结论。请基于提供的数据给出客观、结构化的分析，包含：趋势研判、关键支撑阻力、\
资金面/指标信号、分维度产业资讯解读（含国内宏观与国外金融环境，如美国 CPI/PPI、美联储利率决策等对商品与汇率的传导）、\
潜在风险与关注点。语言简洁专业，避免泛泛而谈。\
务必声明：本分析仅供参考，不构成投资建议。";

pub async fn build_context(
    akshare: &AkshareClient,
    jinshi: Option<&JinshiClient>,
    db: Option<&Database>,
    symbol: &str,
) -> serde_json::Value {
    let end = Utc::now();
    let start = end - Duration::days(90);
    let mut klines = akshare
        .get_history(symbol, "1d", start, end)
        .await
        .unwrap_or_default();
    if klines.len() > 60 {
        klines = klines.split_off(klines.len() - 60);
    }
    let summary = indicator::summary(&klines);

    let fundamentals =
        crate::engine::fundamentals::fetch_fundamentals(akshare, symbol).await;

    let main_symbol = sectors::get_product_by_symbol(symbol)
        .map(|p| p.symbol.to_uppercase())
        .unwrap_or_else(|| symbol.to_uppercase());

    let sector_ctx = sectors::sector_context(symbol);
    let sector_code = sector_ctx["code"].as_str().unwrap_or("");
    let dimension_codes = dimensions::sector_dimension_codes(sector_code);

    let mut news_items: Vec<serde_json::Value> = Vec::new();
    let mut news_by_dimension = serde_json::Map::new();

    if let Some(database) = db {
        for dim_code in &dimension_codes {
            if let Ok(items) = database.get_news_for_symbol(&main_symbol, Some(dim_code), 3) {
                if !items.is_empty() {
                    let label = dimensions::dimension_label(dim_code);
                    let entries: Vec<serde_json::Value> = items
                        .iter()
                        .map(|n| news_entry_json(n, Some(dim_code), Some(label)))
                        .collect();
                    news_by_dimension.insert(dim_code.to_string(), serde_json::Value::Array(entries));
                }
            }
        }
        if let Ok(stored) = database.get_news_for_symbol(&main_symbol, None, 12) {
            news_items = stored
                .iter()
                .map(|n| news_entry_json(n, None, None))
                .collect();
        }
    }

    if news_items.is_empty() {
        if let Some(j) = jinshi {
            if let Ok(raw) = j.fetch_for_symbol(symbol, 6).await {
                news_items = raw
                    .into_iter()
                    .map(|n| {
                        serde_json::json!({
                            "title": n.title,
                            "summary": n.summary,
                            "time": n.display_time,
                            "source": n.source,
                        })
                    })
                    .collect();
            }
        }
    }

    let mut calendar_events: Vec<serde_json::Value> = Vec::new();
    if let Some(j) = jinshi {
        let (cal_start, cal_end) = default_calendar_range_from_today();
        if let Ok(events) = j
            .fetch_calendar_events(cal_start, cal_end, 3, None)
            .await
        {
            let filtered =
                calendar_filter::filter_for_analysis(events, sector_code, &dimension_codes);
            calendar_events = filtered
                .into_iter()
                .map(|e| {
                    serde_json::json!({
                        "pub_time": e.pub_time,
                        "country": e.country,
                        "name": e.name,
                        "star": e.star,
                        "previous": e.previous,
                        "consensus": e.consensus,
                        "actual": e.actual,
                        "unit": e.unit,
                        "status": e.status,
                    })
                })
                .collect();
        }
    }

    let mut dimension_facts: Vec<serde_json::Value> = Vec::new();
    if let Some(database) = db {
        if let Ok(facts) = database.get_dimension_facts(&main_symbol, 15) {
            dimension_facts = facts
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "dimension_code": f.dimension_code,
                        "dimension_label": dimensions::dimension_label(&f.dimension_code),
                        "fact": f.fact,
                        "created_at": f.created_at,
                        "valid_until": f.valid_until,
                    })
                })
                .collect();
        }
    }

    let data_range: Vec<String> = if klines.is_empty() {
        vec![]
    } else {
        vec![
            klines.first().unwrap().start_time.clone(),
            klines.last().unwrap().start_time.clone(),
        ]
    };
    let recent_closes: Vec<f64> = klines.iter().rev().take(10).map(|k| k.close).rev().collect();

    let dimension_list: Vec<serde_json::Value> = dimension_codes
        .iter()
        .map(|code| {
            serde_json::json!({
                "code": code,
                "label": dimensions::dimension_label(code),
            })
        })
        .collect();

    serde_json::json!({
        "symbol": symbol,
        "interval": "1d",
        "sector": sector_ctx,
        "dimensions": dimension_list,
        "data_range": data_range,
        "bars_count": klines.len(),
        "indicator": summary,
        "recent_closes": recent_closes,
        "news": news_items,
        "news_by_dimension": news_by_dimension,
        "calendar_events": calendar_events,
        "dimension_facts": dimension_facts,
        "fundamentals": fundamentals,
        "prompt_version": PROMPT_VERSION,
    })
}

fn news_entry_json(
    n: &crate::models::NewsItemView,
    dim_code: Option<&str>,
    dim_label: Option<&str>,
) -> serde_json::Value {
    let primary = n.classifications.first();
    serde_json::json!({
        "id": n.id,
        "title": n.title,
        "summary": n.summary,
        "time": n.display_time,
        "source": n.source,
        "dimension": dim_code
            .map(String::from)
            .or_else(|| primary.map(|c| c.dimension_code.clone())),
        "dimension_label": dim_label
            .map(String::from)
            .or_else(|| primary.map(|c| c.dimension_label.clone())),
        "classifications": n.classifications,
    })
}

pub fn render_prompt(ctx: &serde_json::Value, trigger: &str) -> String {
    let symbol = ctx["symbol"].as_str().unwrap_or("");
    let ind = &ctx["indicator"];
    let sector = &ctx["sector"];
    let sector_name = sector["name"].as_str().unwrap_or("未分类");
    let sector_desc = sector["description"].as_str().unwrap_or("");
    let product_name = sector["product_name"].as_str().unwrap_or(symbol);
    let main_symbol = sector["main_symbol"].as_str().unwrap_or(symbol);
    let related = sector["related_products"].clone();
    let drivers = sector["drivers"].clone();
    let dimension_list = ctx["dimensions"].as_array().cloned().unwrap_or_default();
    let news_by_dim = ctx["news_by_dimension"].as_object().cloned();
    let news = ctx["news"].as_array().cloned().unwrap_or_default();
    let calendar = ctx["calendar_events"].as_array().cloned().unwrap_or_default();
    let dimension_facts = ctx["dimension_facts"].as_array().cloned().unwrap_or_default();
    let fundamentals = &ctx["fundamentals"];

    let trigger_label = match trigger {
        "daily" => "每日收盘分析",
        "scheduled" => "定时全面分析",
        "realtime" => "盘中实时分析",
        "anomaly" => "异动触发分析",
        "tomorrow" => "明日走势展望（下一交易日）",
        "short_term" => "短期走势研判（未来3-5个交易日）",
        _ => "用户手动请求",
    };
    let change_pct = ind["change_pct"]
        .as_f64()
        .map(|v| format!("{v:.2}"))
        .unwrap_or_else(|| "N/A".into());

    let dimension_news_block = render_dimension_news_block(&dimension_list, news_by_dim.as_ref(), &news);
    let calendar_block = render_calendar_block(&calendar);
    let facts_block = render_dimension_facts_block(&dimension_facts);
    let fundamentals_block = render_fundamentals_block(fundamentals);
    let dimension_output_list = dimension_list
        .iter()
        .map(|d| {
            format!(
                "- [{}] {}：结合该维度资讯与板块逻辑给出要点",
                d["code"].as_str().unwrap_or(""),
                d["label"].as_str().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let horizon_block = match trigger {
        "tomorrow" => "\n\n## 重点输出（明日展望）\n\
            请在 Markdown 正文中**单独增加**「明日走势展望」小节，基于日 K 技术面、\
            财经日历中与下一交易日相关的 ★3+ 事件、以及分维度资讯，给出下一交易日的：\
            方向倾向（偏多/偏空/震荡）、关键价位、主要风险与催化。\n",
        "short_term" => "\n\n## 重点输出（短期研判）\n\
            请在 Markdown 正文中**单独增加**「短期走势研判（3-5个交易日）」小节，\
            结合日 K 均线/MACD 趋势、量能、宏观日程与产业维度，给出未来 3-5 个交易日的\
            路径推演、区间目标与止损/止盈参考（非具体操作建议）。\n",
        _ => "",
    };

    format!(
        "请对国内期货品种【{product_name}主力（{main_symbol}）】进行{trigger_label}。\n\
        **分析周期：日 K 线（1d）**——以下技术面数据均为日 K，请基于日 K 级别研判趋势与关键价位。\n\n\
        ## 品种与板块\n\
        - 所属板块：{sector_name}（{sector_desc}）\n\
        - 相关品种：{related}\n\
        - 板块驱动：{drivers}\n\n\
        ## 技术面（最近 {bars} 根日 K，来源 AKShare）\n\
        - 最新价：{last}\n\
        - 区间涨跌幅：{change_pct}%（如可用）\n\
        - MA5={ma5}  MA20={ma20}  MA60={ma60}\n\
        - MACD: DIF={dif}  DEA={dea}  HIST={hist}\n\
        - 近20日均量：{avg_vol}\n\
        - 区间最高/最低：{max_h} / {min_l}\n\
        - 近10日收盘：{closes}\n\n\
        ## 分维度资讯（已分类入库，供基本面参考）\n\
        {dimension_news_block}\n\n\
        ## 历史分析事实（dimension_facts，供延续性参考）\n\
        {facts_block}\n\n\
        ## 基本面快照（持仓/仓单/基差）\n\
        {fundamentals_block}\n\n\
        ## 宏观数据发布日程（金十财经日历，未来两周 · 已按板块筛选）\n\
        {calendar_block}\n\n\
        ## 输出要求\n\
        1. **首先**输出一个 JSON 代码块（```json），格式如下（键为维度 code，值为要点字符串数组）：\n\
        ```json\n\
        {{\"dimension_summary\":{{\"demand\":[\"要点1\"],\"inventory\":[\"要点1\"]}}}}\n\
        ```\n\
        2. **然后**输出 Markdown 正文，按以下分析维度逐一给出要点（无资讯的维度可简要说明依据）：\n\
        {dimension_output_list}\n\n\
        最后单独输出：\n\
        1. 趋势研判（多头/空头/震荡，**依据日 K 均线与 MACD**）\n\
        2. 关键支撑位与阻力位\n\
        3. 资金面/量能信号\n\
        4. 综合风险与短期关注点\n\
        5. 免责声明：本分析仅供参考，不构成投资建议。{horizon_block}",
        bars = ctx["bars_count"].as_u64().unwrap_or(0),
        last = ind["last"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        ma5 = ind["ma5"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        ma20 = ind["ma20"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        ma60 = ind["ma60"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        dif = ind["macd_dif"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        dea = ind["macd_dea"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        hist = ind["macd_hist"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        avg_vol = ind["avg_volume"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        max_h = ind["max_high"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        min_l = ind["min_low"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "N/A".into()),
        closes = ctx["recent_closes"].clone(),
    )
}

fn render_fundamentals_block(fundamentals: &serde_json::Value) -> String {
    if fundamentals.is_null() {
        return "（暂无基本面数据）".into();
    }
    let oi = fundamentals["open_interest"]
        .as_i64()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "—".into());
    let note = fundamentals["note"].as_str().unwrap_or("");
    format!(
        "- 持仓量（OI）：{oi}\n- 仓单：{}\n- 基差：{}\n- 来源：{}\n- 说明：{note}",
        fundamentals["warehouse"].as_str().unwrap_or("—"),
        fundamentals["basis"].as_str().unwrap_or("—"),
        fundamentals["source"].as_str().unwrap_or("—"),
    )
}

fn render_dimension_facts_block(facts: &[serde_json::Value]) -> String {
    if facts.is_empty() {
        return "（暂无历史分析事实）".into();
    }
    facts
        .iter()
        .enumerate()
        .map(|(i, f)| {
            format!(
                "  {}. [{}] {} — {}",
                i + 1,
                f["dimension_label"].as_str().unwrap_or(""),
                f["created_at"].as_str().unwrap_or(""),
                f["fact"].as_str().unwrap_or(""),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_calendar_block(events: &[serde_json::Value]) -> String {
    if events.is_empty() {
        return "（暂无日程数据，或未配置金十日历接口）".into();
    }
    events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let stars = e["star"].as_u64().unwrap_or(1);
            format!(
                "  {}. [{}] {} {} ★{} | 前值={} 预期={} 公布={}{}",
                i + 1,
                e["pub_time"].as_str().unwrap_or(""),
                e["country"].as_str().unwrap_or(""),
                e["name"].as_str().unwrap_or(""),
                stars,
                e["previous"].as_str().unwrap_or("-"),
                e["consensus"].as_str().unwrap_or("-"),
                e["actual"].as_str().unwrap_or("-"),
                e["unit"]
                    .as_str()
                    .map(|u| format!(" ({u})"))
                    .unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_dimension_news_block(
    dimensions: &[serde_json::Value],
    by_dim: Option<&serde_json::Map<String, serde_json::Value>>,
    fallback_news: &[serde_json::Value],
) -> String {
    let mut sections = Vec::new();

    if let Some(map) = by_dim {
        for dim in dimensions {
            let code = dim["code"].as_str().unwrap_or("");
            let label = dim["label"].as_str().unwrap_or(code);
            if let Some(items) = map.get(code).and_then(|v| v.as_array()) {
                if items.is_empty() {
                    continue;
                }
                let lines: Vec<String> = items
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        format!(
                            "  {}. [{}] {} — {}",
                            i + 1,
                            n["time"].as_str().unwrap_or(""),
                            n["title"].as_str().unwrap_or(""),
                            n["summary"].as_str().unwrap_or("").chars().take(160).collect::<String>()
                        )
                    })
                    .collect();
                sections.push(format!("### {label} ({code})\n{}", lines.join("\n")));
            }
        }
    }

    if sections.is_empty() && !fallback_news.is_empty() {
        let lines: Vec<String> = fallback_news
            .iter()
            .take(8)
            .enumerate()
            .map(|(i, n)| {
                let dim = n["dimension_label"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(|l| format!("[{l}] "))
                    .unwrap_or_default();
                format!(
                    "{}. {}{} {} — {}",
                    i + 1,
                    dim,
                    n["time"].as_str().unwrap_or(""),
                    n["title"].as_str().unwrap_or(""),
                    n["summary"].as_str().unwrap_or("").chars().take(160).collect::<String>()
                )
            })
            .collect();
        return lines.join("\n");
    }

    if sections.is_empty() {
        "（暂无相关资讯）".into()
    } else {
        sections.join("\n\n")
    }
}

pub fn summarize_context(ctx: &serde_json::Value) -> String {
    let ind = &ctx["indicator"];
    if ind.is_null() || ind.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return "no data".into();
    }
    format!(
        "日K last={} change%={} MA5={} MA20={} MACD_hist={}",
        ind["last"].as_f64().unwrap_or(0.0),
        ind["change_pct"].as_f64().unwrap_or(0.0),
        ind["ma5"].as_f64().unwrap_or(0.0),
        ind["ma20"].as_f64().unwrap_or(0.0),
        ind["macd_hist"].as_f64().unwrap_or(0.0),
    )
}
