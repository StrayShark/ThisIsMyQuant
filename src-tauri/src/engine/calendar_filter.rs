//! 按板块/维度过滤宏观日历事件，供 LLM 分析上下文使用。

use crate::models::CalendarEvent;

const MAX_EVENTS: usize = 20;

/// 按板块维度与重要性筛选日历事件。
pub fn filter_for_analysis(
    events: Vec<CalendarEvent>,
    sector_code: &str,
    dimension_codes: &[&str],
) -> Vec<CalendarEvent> {
    let countries = countries_for_dimensions(dimension_codes);
    let keywords = sector_calendar_keywords(sector_code);

    let mut picked: Vec<CalendarEvent> = events
        .into_iter()
        .filter(|e| event_relevant(e, &countries, &keywords))
        .collect();

    picked.sort_by(|a, b| {
        b.star
            .cmp(&a.star)
            .then_with(|| a.pub_time.cmp(&b.pub_time))
    });
    picked.truncate(MAX_EVENTS);
    picked
}

/// 按事件名称关键词过滤（如「非农」「CPI」）。
pub fn filter_by_keyword(events: Vec<CalendarEvent>, keyword: &str) -> Vec<CalendarEvent> {
    let kw = keyword.trim().to_lowercase();
    if kw.is_empty() {
        return events;
    }
    events
        .into_iter()
        .filter(|e| e.name.to_lowercase().contains(&kw))
        .collect()
}

fn event_relevant(event: &CalendarEvent, countries: &[&str], keywords: &[&str]) -> bool {
    if event.star >= 4 {
        return true;
    }
    if countries.iter().any(|c| country_matches(&event.country, c)) {
        return true;
    }
    let name = event.name.to_lowercase();
    keywords.iter().any(|kw| name.contains(&kw.to_lowercase()))
}

fn country_matches(event_country: &str, target: &str) -> bool {
    event_country.contains(target) || target.contains(event_country)
}

fn countries_for_dimensions(dimension_codes: &[&str]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for code in dimension_codes {
        match *code {
            "macro" => out.push("中国"),
            "overseas_finance" => {
                out.push("美国");
                out.push("欧元区");
                out.push("英国");
                out.push("日本");
            }
            "overseas_upstream" => {
                out.push("美国");
                out.push("中国");
            }
            _ => {}
        }
    }
    if out.is_empty() {
        out.extend(["中国", "美国", "欧元区"]);
    }
    out.sort_unstable();
    out.dedup();
    out
}

fn sector_calendar_keywords(sector_code: &str) -> Vec<&'static str> {
    match sector_code {
        "black" => vec![
            "PMI", "CPI", "PPI", "GDP", "非农", "库存", "产量", "铁", "钢", "地产",
        ],
        "metals" => vec![
            "CPI",
            "PPI",
            "非农",
            "美联储",
            "FOMC",
            "GDP",
            "PMI",
            "美元",
            "LME",
            "铜",
            "金",
        ],
        "agriculture" => vec![
            "CPI", "PPI", "PMI", "GDP", "天气", "USDA", "库存", "产量", "出口", "进口",
        ],
        "energy_chemical" => vec![
            "CPI",
            "PPI",
            "非农",
            "OPEC",
            "原油",
            "库存",
            "GDP",
            "PMI",
            "美元",
            "美联储",
        ],
        "shipping" => vec!["CPI", "PMI", "GDP", "出口", "进口", "贸易", "地缘", "油价"],
        _ => vec!["CPI", "PPI", "非农", "PMI", "GDP", "LPR", "美联储"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CalendarEvent;

    fn ev(country: &str, name: &str, star: u8) -> CalendarEvent {
        CalendarEvent {
            id: name.into(),
            pub_time: "2025-06-01 09:30".into(),
            country: country.into(),
            name: name.into(),
            star,
            previous: None,
            consensus: None,
            actual: None,
            unit: None,
            affect: None,
            status: "pending".into(),
            event_type: "data".into(),
        }
    }

    #[test]
    fn keeps_high_star_events() {
        let events = vec![ev("德国", "ZEW", 3), ev("美国", "CPI", 5)];
        let out = filter_for_analysis(events, "metals", &["overseas_finance", "macro"]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "CPI");
    }

    #[test]
    fn filters_by_sector_keywords() {
        let events = vec![ev("中国", "5月制造业PMI", 3), ev("日本", "机器订单", 3)];
        let out = filter_for_analysis(events, "black", &["demand", "macro"]);
        assert!(out.iter().any(|e| e.name.contains("PMI")));
        assert!(!out.iter().any(|e| e.name.contains("机器")));
    }
}
