use std::collections::HashSet;

use chrono::{Duration, NaiveDate, Utc};
use reqwest::Client;
use serde_json::Value;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{CalendarEvent, news_content_hash};

const RILI_HEADERS: [(&str, &str); 3] = [
    ("x-app-id", "sKKYe29sFuJaeOCJ"),
    ("x-version", "0.28"),
    ("User-Agent", "Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)"),
];

pub struct CalendarFetchOptions<'a> {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub min_star: u8,
    pub country: Option<&'a str>,
}

pub async fn fetch_calendar_range(
    http: &Client,
    config: &Config,
    opts: CalendarFetchOptions<'_>,
) -> AppResult<Vec<CalendarEvent>> {
    if !config.jinshi_enabled {
        return Ok(vec![]);
    }

    let mut events = Vec::new();
    if !config.jin10_mcp_token.trim().is_empty() {
        match fetch_calendar_mcp(http, config).await {
            Ok(mcp_events) if !mcp_events.is_empty() => {
                log::info!("jinshi calendar MCP: {} events", mcp_events.len());
                events = mcp_events;
            }
            Ok(_) => log::debug!("jinshi calendar MCP returned empty"),
            Err(e) => log::warn!("jinshi calendar MCP failed: {e}"),
        }
    }
    if events.is_empty() {
        events = fetch_calendar_http_range(http, config, &opts).await?;
        if !events.is_empty() {
            log::info!("jinshi calendar HTTP: {} events", events.len());
        }
    }

    Ok(filter_calendar_events(events, &opts))
}

async fn fetch_calendar_http_range(
    http: &Client,
    config: &Config,
    opts: &CalendarFetchOptions<'_>,
) -> AppResult<Vec<CalendarEvent>> {
    let base = config.jinshi_rili_api_base.trim_end_matches('/');
    let mut all = Vec::new();
    let mut day = opts.start;
    while day <= opts.end {
        let date = day.format("%Y-%m-%d").to_string();
        match fetch_calendar_day(http, base, &config.jinshi_rili_app_id, &date).await {
            Ok(mut day_events) => all.append(&mut day_events),
            Err(e) => log::debug!("jinshi calendar {date}: {e}"),
        }
        day += Duration::days(1);
    }
    Ok(all)
}

async fn fetch_calendar_day(
    http: &Client,
    base: &str,
    app_id: &str,
    date: &str,
) -> AppResult<Vec<CalendarEvent>> {
    let url = format!("{base}/data/getDay");
    let mut req = http.get(&url).query(&[("date", date)]);
    for (k, v) in RILI_HEADERS {
        req = req.header(k, if k == "x-app-id" && !app_id.is_empty() { app_id } else { v });
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Msg(format!(
            "calendar HTTP {}",
            resp.status().as_u16()
        )));
    }
    let payload: Value = resp.json().await?;
    Ok(parse_calendar_payload(&payload))
}

pub fn parse_calendar_payload(payload: &Value) -> Vec<CalendarEvent> {
    let mut events = Vec::new();
    for arr in extract_event_arrays(payload) {
        for item in arr {
            if let Some(ev) = parse_calendar_item(&item, "data") {
                events.push(ev);
            }
        }
    }
    events
}

fn extract_event_arrays(payload: &Value) -> Vec<&Vec<Value>> {
    let mut out = Vec::new();
    if let Some(arr) = payload.as_array() {
        out.push(arr);
        return out;
    }
    if let Some(data) = payload.get("data") {
        if let Some(arr) = data.as_array() {
            out.push(arr);
        } else if let Some(obj) = data.as_object() {
            for key in ["economics", "economic", "list", "events", "data"] {
                if let Some(arr) = obj.get(key).and_then(|v| v.as_array()) {
                    out.push(arr);
                }
            }
        }
    }
    for key in ["economics", "economic", "list", "events"] {
        if let Some(arr) = payload.get(key).and_then(|v| v.as_array()) {
            out.push(arr);
        }
    }
    out
}

fn parse_calendar_item(raw: &Value, default_type: &str) -> Option<CalendarEvent> {
    let obj = raw.as_object()?;
    let mut country = pick_str(obj, &["country", "country_name", "region"])
        .unwrap_or_default();
    let name = pick_str(
        obj,
        &[
            "name",
            "indicator_name",
            "title",
            "event_content",
            "event",
        ],
    )?;
    if name.trim().is_empty() {
        return None;
    }
    if country.is_empty() {
        country = extract_country_from_title(&name);
    }
    let pub_time = pick_str(
        obj,
        &["pub_time", "publish_time", "time", "event_time", "date"],
    )
    .unwrap_or_default();
    let star = pick_u8(obj, &["star", "importance"]).unwrap_or(1);
    let previous = pick_str(obj, &["previous", "prev", "前值"]);
    let consensus = pick_str(obj, &["consensus", "forecast", "预测值"]);
    let actual = pick_str(obj, &["actual", "act", "今值"]);
    let unit = pick_str(obj, &["unit", "time_period"]);
    let affect = pick_str(obj, &["affect_txt", "affect_text"]).or_else(|| {
        obj.get("affect")
            .and_then(|v| v.as_i64())
            .map(|n| n.to_string())
    });
    let event_type = if pick_str(obj, &["event_content", "event"]).is_some()
        && pick_str(obj, &["indicator_name", "name"]).is_none()
    {
        "event".into()
    } else {
        default_type.into()
    };
    let status = if actual.as_deref().unwrap_or("").trim().is_empty() {
        "scheduled".into()
    } else if is_future_pub_time(&pub_time) {
        "scheduled".into()
    } else {
        "released".into()
    };
    let id = news_content_hash(
        &format!("{country}|{name}"),
        &pub_time,
    );
    Some(CalendarEvent {
        id,
        pub_time,
        country,
        name,
        star,
        previous,
        consensus,
        actual,
        unit,
        affect,
        status,
        event_type,
    })
}

fn pick_str(obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
            if let Some(s) = v.as_str() {
                if !s.trim().is_empty() {
                    return Some(s.trim().to_string());
                }
            } else if v.is_number() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn pick_u8(obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<u8> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
            if let Some(n) = v.as_u64() {
                return Some(n.min(5) as u8);
            }
            if let Some(n) = v.as_i64() {
                return Some(n.clamp(0, 5) as u8);
            }
        }
    }
    None
}

fn is_future_pub_time(pub_time: &str) -> bool {
    if pub_time.is_empty() {
        return false;
    }
    if let Some(dt) = crate::models::parse_dt(pub_time) {
        return dt > Utc::now();
    }
    false
}

fn extract_country_from_title(title: &str) -> String {
    const PREFIXES: &[&str] = &[
        "欧元区", "中国香港", "中国台湾", "中国", "美国", "英国", "德国", "日本", "法国", "意大利",
        "西班牙", "加拿大", "澳大利亚", "新西兰", "韩国", "印度", "巴西", "俄罗斯", "瑞士",
        "瑞典", "挪威", "丹麦", "荷兰", "比利时", "奥地利", "新加坡", "马来西亚", "泰国",
        "越南", "菲律宾", "南非", "墨西哥", "阿根廷", "土耳其", "沙特", "阿联酋", "以色列",
        "波兰", "捷克", "匈牙利", "印尼", "印度尼西亚",
    ];
    for prefix in PREFIXES {
        if title.starts_with(prefix) {
            return prefix.to_string();
        }
    }
    String::new()
}

fn filter_calendar_events(
    events: Vec<CalendarEvent>,
    opts: &CalendarFetchOptions<'_>,
) -> Vec<CalendarEvent> {
    let mut seen = HashSet::new();
    let mut filtered: Vec<CalendarEvent> = events
        .into_iter()
        .filter(|e| e.star >= opts.min_star)
        .filter(|e| {
            opts.country.map(|c| e.country.contains(c)).unwrap_or(true)
        })
        .filter(|e| event_in_range(&e.pub_time, opts.start, opts.end))
        .filter(|e| seen.insert(e.id.clone()))
        .collect();
    filtered.sort_by(|a, b| a.pub_time.cmp(&b.pub_time));
    filtered
}

fn event_in_range(pub_time: &str, start: NaiveDate, end: NaiveDate) -> bool {
    if pub_time.is_empty() {
        return true;
    }
    if let Some(dt) = crate::models::parse_dt(pub_time) {
        let d = dt.date_naive();
        return d >= start && d <= end;
    }
    if let Ok(d) = NaiveDate::parse_from_str(&pub_time[..10.min(pub_time.len())], "%Y-%m-%d") {
        return d >= start && d <= end;
    }
    true
}

async fn fetch_calendar_mcp(http: &Client, config: &Config) -> AppResult<Vec<CalendarEvent>> {
    let token = config.jin10_mcp_token.trim();
    if token.is_empty() {
        return Ok(vec![]);
    }
    let server = config.jin10_mcp_server_url.trim_end_matches('/');
    let mut req_id = 0_i64;

    req_id += 1;
    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "method": "initialize",
        "params": {
            "protocolVersion": config.jin10_mcp_protocol_version,
            "capabilities": {},
            "clientInfo": { "name": "ThisIsMyQuant", "version": "0.1.0" }
        }
    });

    let init_resp = http
        .post(server)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&init_body)
        .send()
        .await?;
    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let init_text = init_resp.text().await?;
    let _ = parse_sse_rpc_result(&init_text)?;

    let notify = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    let mut notify_req = http
        .post(server)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&notify);
    if let Some(ref sid) = session_id {
        notify_req = notify_req.header("Mcp-Session-Id", sid);
    }
    let _ = notify_req.send().await?;

    req_id += 1;
    let call_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "method": "tools/call",
        "params": {
            "name": "list_calendar",
            "arguments": {}
        }
    });
    let mut call_req = http
        .post(server)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&call_body);
    if let Some(ref sid) = session_id {
        call_req = call_req.header("Mcp-Session-Id", sid);
    }
    let call_resp = call_req.send().await?;
    let call_text = call_resp.text().await?;
    let result = parse_sse_rpc_result(&call_text)?;
    Ok(parse_mcp_calendar_result(&result))
}

fn parse_sse_rpc_result(text: &str) -> AppResult<Value> {
    let mut data_lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        } else if line.is_empty() && !data_lines.is_empty() {
            let raw = data_lines.join("\n");
            data_lines.clear();
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                if v.get("error").is_some() {
                    return Err(AppError::Msg(format!("MCP error: {raw}")));
                }
                if v.get("result").is_some() {
                    return Ok(v["result"].clone());
                }
            }
        }
    }
    if !data_lines.is_empty() {
        let raw = data_lines.join("\n");
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            if v.get("result").is_some() {
                return Ok(v["result"].clone());
            }
        }
    }
    Err(AppError::Msg("no MCP result in SSE".into()))
}

fn parse_mcp_calendar_result(result: &Value) -> Vec<CalendarEvent> {
    if let Some(content) = pick_primary_mcp_data(result) {
        return parse_mcp_calendar_value(&content);
    }
    parse_mcp_calendar_value(result)
}

fn pick_primary_mcp_data(result: &Value) -> Option<Value> {
    if result.get("structuredContent").is_some() {
        return Some(result["structuredContent"].clone());
    }
    if let Some(arr) = result.get("content").and_then(|v| v.as_array()) {
        for item in arr {
            if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                        return Some(parsed);
                    }
                }
            }
        }
    }
    None
}

fn parse_mcp_calendar_value(value: &Value) -> Vec<CalendarEvent> {
    let mut events = Vec::new();
    let arrays = extract_event_arrays(value);
    if arrays.is_empty() {
        if let Some(obj) = value.as_object() {
            for (_, v) in obj {
                if let Some(arr) = v.as_array() {
                    for item in arr {
                        if let Some(ev) = parse_mcp_item(item) {
                            events.push(ev);
                        }
                    }
                }
            }
        }
    } else {
        for arr in arrays {
            for item in arr {
                if let Some(ev) = parse_mcp_item(item) {
                    events.push(ev);
                }
            }
        }
    }
    events
}

fn parse_mcp_item(raw: &Value) -> Option<CalendarEvent> {
    let mut ev = parse_calendar_item(raw, "data")?;
    if ev.name.is_empty() {
        ev.name = pick_str(raw.as_object()?, &["title"])?;
    }
    if ev.pub_time.is_empty() {
        ev.pub_time = pick_str(raw.as_object()?, &["pub_time"]).unwrap_or_default();
    }
    Some(ev)
}

pub const DEFAULT_CALENDAR_DAYS_AHEAD: i64 = 14;

pub fn default_calendar_range(days_ahead: i64) -> (NaiveDate, NaiveDate) {
    let today = Utc::now().date_naive();
    (today, today + Duration::days(days_ahead))
}

pub fn default_calendar_range_from_today() -> (NaiveDate, NaiveDate) {
    default_calendar_range(DEFAULT_CALENDAR_DAYS_AHEAD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get_day_array_payload() {
        let payload = serde_json::json!({
            "status": 200,
            "data": [
                {
                    "country": "美国",
                    "pub_time": "2026-06-25 20:30:00",
                    "name": "美国CPI月率",
                    "star": 5,
                    "previous": "0.2%",
                    "consensus": "0.3%",
                    "actual": "",
                    "unit": "%"
                },
                {
                    "country": "中国",
                    "pub_time": "2020-06-20 09:30:00",
                    "indicator_name": "中国官方制造业PMI",
                    "star": 4,
                    "previous": "49.5",
                    "consensus": "49.8",
                    "actual": "50.1"
                }
            ]
        });
        let events = parse_calendar_payload(&payload);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name, "美国CPI月率");
        assert_eq!(events[0].status, "scheduled");
        assert_eq!(events[1].status, "released");
        assert_eq!(events[1].star, 4);
    }

    #[test]
    fn parse_nested_economics_payload() {
        let payload = serde_json::json!({
            "data": {
                "economics": [
                    {
                        "country": "美国",
                        "publish_time": "2026-06-27 02:00:00",
                        "indicator_name": "美联储利率决定(上限)",
                        "star": 5,
                        "previous": "4.50%",
                        "consensus": "4.25%"
                    }
                ]
            }
        });
        let events = parse_calendar_payload(&payload);
        assert_eq!(events.len(), 1);
        assert!(events[0].name.contains("美联储"));
    }

    #[test]
    fn parse_mcp_calendar_payload() {
        let payload = serde_json::json!({
            "data": [{
                "actual": "",
                "affect_txt": "利空",
                "consensus": "3.00",
                "previous": "3.00",
                "pub_time": "2026-06-25 20:30",
                "star": 4,
                "title": "美国5月核心PCE物价指数年率"
            }]
        });
        let events = parse_calendar_payload(&payload);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].country, "美国");
        assert_eq!(events[0].name, "美国5月核心PCE物价指数年率");
        assert_eq!(events[0].status, "scheduled");
        assert_eq!(events[0].affect.as_deref(), Some("利空"));
    }

    #[test]
    fn filter_by_star_and_country() {
        let events = vec![
            CalendarEvent {
                id: "1".into(),
                pub_time: "2026-06-25 20:30:00".into(),
                country: "美国".into(),
                name: "CPI".into(),
                star: 5,
                previous: None,
                consensus: None,
                actual: None,
                unit: None,
                affect: None,
                status: "scheduled".into(),
                event_type: "data".into(),
            },
            CalendarEvent {
                id: "2".into(),
                pub_time: "2026-06-25 10:00:00".into(),
                country: "日本".into(),
                name: "短观".into(),
                star: 2,
                previous: None,
                consensus: None,
                actual: None,
                unit: None,
                affect: None,
                status: "scheduled".into(),
                event_type: "data".into(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let filtered = filter_calendar_events(
            events,
            &CalendarFetchOptions {
                start,
                end,
                min_star: 3,
                country: Some("美国"),
            },
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].country, "美国");
    }
}
