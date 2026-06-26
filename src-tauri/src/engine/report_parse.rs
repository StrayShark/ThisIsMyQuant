//! 从 LLM 报告正文中解析 dimension_summary JSON 块。

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ParsedReport {
    pub content: String,
    pub dimension_summary: Option<Value>,
}

/// 提取 ```json ... ``` 中的 dimension_summary，并从正文中移除该块。
pub fn parse_llm_report(raw: &str) -> ParsedReport {
    if let Some((json_body, start, end)) = extract_json_fence(raw) {
        let dimension_summary = parse_dimension_summary(&json_body);
        let mut content = String::new();
        content.push_str(raw[..start].trim());
        let tail = raw[end..].trim();
        if !content.is_empty() && !tail.is_empty() {
            content.push_str("\n\n");
        }
        content.push_str(tail);
        return ParsedReport {
            content: content.trim().to_string(),
            dimension_summary,
        };
    }

    // 无 ```json fence 时，尝试从正文截取 JSON 对象
    if let Some((json_body, start, end)) = extract_loose_json_object(raw) {
        if let Some(dimension_summary) = parse_dimension_summary(&json_body) {
            let mut content = String::new();
            content.push_str(raw[..start].trim());
            let tail = raw[end..].trim();
            if !content.is_empty() && !tail.is_empty() {
                content.push_str("\n\n");
            }
            content.push_str(tail);
            return ParsedReport {
                content: content.trim().to_string(),
                dimension_summary: Some(dimension_summary),
            };
        }
    }

    ParsedReport {
        content: raw.trim().to_string(),
        dimension_summary: None,
    }
}

fn extract_json_fence(raw: &str) -> Option<(String, usize, usize)> {
    let lower = raw.to_lowercase();
    let start_marker = "```json";
    let start = lower.find(start_marker)?;
    let content_start = start + start_marker.len();
    let rest = &raw[content_start..];
    let end_rel = rest.find("```")?;
    let json_body = rest[..end_rel].trim().to_string();
    let end = content_start + end_rel + 3;
    Some((json_body, start, end))
}

fn extract_loose_json_object(raw: &str) -> Option<(String, usize, usize)> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    let json_body = raw[start..=end].trim().to_string();
    Some((json_body, start, end + 1))
}

fn parse_dimension_summary(json_body: &str) -> Option<Value> {
    if let Some(v) = try_parse_dimension_summary(json_body) {
        return Some(v);
    }
    // 容错：截取首个含 dimension_summary 的 JSON 对象
    if let Some(start) = json_body.find('{') {
        if let Some(end) = json_body.rfind('}') {
            if end > start {
                if let Some(v) = try_parse_dimension_summary(&json_body[start..=end]) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn try_parse_dimension_summary(json_body: &str) -> Option<Value> {
    let parsed: Value = serde_json::from_str(json_body.trim()).ok()?;
    if let Some(inner) = parsed.get("dimension_summary") {
        return Some(inner.clone());
    }
    if parsed.is_object() {
        return Some(parsed);
    }
    None
}

pub fn collect_news_ids(ctx: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(arr) = ctx.get("news").and_then(|v| v.as_array()) {
        for n in arr {
            if let Some(id) = n.get("id").and_then(|v| v.as_str()) {
                if !id.is_empty() && !ids.contains(&id.to_string()) {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_json_fence_and_strips_content() {
        let raw = r#"```json
{"dimension_summary":{"demand":["地产偏弱"],"inventory":["库存下降"]}}
```

## 趋势研判
震荡为主。"#;
        let parsed = parse_llm_report(raw);
        assert!(parsed.content.contains("趋势研判"));
        assert!(!parsed.content.contains("```json"));
        let ds = parsed.dimension_summary.unwrap();
        assert_eq!(ds["demand"][0], "地产偏弱");
    }

    #[test]
    fn parses_loose_json_without_fence() {
        let raw = r#"前置说明
{"dimension_summary":{"demand":["需求偏弱"]}}
正文"#;
        let parsed = parse_llm_report(raw);
        assert!(parsed.dimension_summary.is_some());
    }

    #[test]
    fn plain_content_without_json() {
        let parsed = parse_llm_report("仅 Markdown 正文");
        assert!(parsed.dimension_summary.is_none());
        assert_eq!(parsed.content, "仅 Markdown 正文");
    }

    #[test]
    fn collect_ids_from_context() {
        let ctx = json!({
            "news": [
                {"id": "news-abc", "title": "t"},
                {"id": "", "title": "x"},
                {"title": "no id"},
            ]
        });
        assert_eq!(collect_news_ids(&ctx), vec!["news-abc"]);
    }
}
