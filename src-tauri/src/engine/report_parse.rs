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

fn parse_dimension_summary(json_body: &str) -> Option<Value> {
    let parsed: Value = serde_json::from_str(json_body).ok()?;
    if let Some(inner) = parsed.get("dimension_summary") {
        return Some(inner.clone());
    }
    // 允许直接返回 { "demand": [...], ... }
    if parsed.is_object() && parsed.get("dimension_summary").is_none() {
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
