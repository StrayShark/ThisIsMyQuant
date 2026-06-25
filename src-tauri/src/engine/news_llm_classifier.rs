//! LLM 批量资讯分类（规则未命中时的第二层）。

use serde_json::{json, Value};

use crate::adapters::LlmRouter;
use crate::engine::dimensions::{self, sector_dimension_codes};
use crate::engine::sectors::{self, LiquidityTier};
use crate::error::AppResult;
use crate::models::{NewsClassification, NewsRecord};

pub const LLM_CLASSIFY_SYSTEM: &str =
    "你是期货资讯分类器。根据新闻标题与摘要，将其关联到具体期货品种（symbol）与分析维度（dimension）。\
只输出合法 JSON，不要 markdown 代码块，不要解释。";

pub async fn classify_batch(
    llm: &LlmRouter,
    provider: Option<&str>,
    records: &[NewsRecord],
) -> AppResult<Vec<NewsClassification>> {
    if records.is_empty() {
        return Ok(vec![]);
    }

    let prompt = build_batch_prompt(records);
    let raw = llm
        .complete_json(&prompt, LLM_CLASSIFY_SYSTEM, provider)
        .await?;
    Ok(parse_llm_response(&raw, records))
}

fn build_batch_prompt(records: &[NewsRecord]) -> String {
    let products: Vec<Value> = sectors::all_products()
        .into_iter()
        .filter(|p| p.default_tier != LiquidityTier::Excluded)
        .map(|p| {
            let sector = sectors::get_sector_by_symbol(&p.symbol);
            let dims: Vec<&str> = sector
                .as_ref()
                .map(|s| sector_dimension_codes(&s.code))
                .unwrap_or_default();
            json!({
                "symbol": p.symbol,
                "name": p.name,
                "sector": sector.as_ref().map(|s| s.name.clone()),
                "dimensions": dims,
            })
        })
        .collect();

    let items: Vec<Value> = records
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "title": r.title,
                "summary": r.summary,
                "category_id": r.category_id,
            })
        })
        .collect();

    format!(
        "将下列资讯分类到品种 symbol 与分析维度 dimension。\n\
维度说明：{dim_labels}\n\n\
候选品种（symbol + 可用 dimensions）：\n{products}\n\n\
待分类资讯：\n{items}\n\n\
输出 JSON 格式（仅此结构）：\n\
{{\"labels\":[{{\"news_id\":\"...\",\"symbol\":\"RB0\",\"dimension\":\"demand\",\"confidence\":0.85}}]}}\n\
规则：每条资讯至少 0 个标签；confidence 0-1；symbol 必须来自候选；dimension 必须来自该品种 dimensions。",
        dim_labels = dimension_legend(),
        products = serde_json::to_string_pretty(&products).unwrap_or_default(),
        items = serde_json::to_string_pretty(&items).unwrap_or_default(),
    )
}

fn dimension_legend() -> String {
    dimensions::all_dimensions()
        .iter()
        .map(|d| format!("{}={}", d.code, d.label))
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_llm_response(raw: &str, records: &[NewsRecord]) -> Vec<NewsClassification> {
    let json_str = extract_json_object(raw);
    let Ok(parsed) = serde_json::from_str::<Value>(&json_str) else {
        log::warn!("LLM classify: invalid JSON: {}", raw.chars().take(200).collect::<String>());
        return vec![];
    };

    let record_map: std::collections::HashMap<&str, &NewsRecord> =
        records.iter().map(|r| (r.id.as_str(), r)).collect();

    let mut out = Vec::new();
    let Some(labels) = parsed.get("labels").and_then(|v| v.as_array()) else {
        return out;
    };

    for label in labels {
        let news_id = label
            .get("news_id")
            .or_else(|| label.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let symbol = label
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_uppercase();
        let dimension = label
            .get("dimension")
            .or_else(|| label.get("dimension_code"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let confidence = label
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.75) as f32;

        if news_id.is_empty() || symbol.is_empty() || dimension.is_empty() {
            continue;
        }
        if !is_valid_label(&symbol, &dimension) {
            continue;
        }
        let ingested_at = record_map
            .get(news_id)
            .map(|r| r.ingested_at.clone())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        out.push(NewsClassification {
            news_id: news_id.to_string(),
            symbol,
            dimension_code: dimension,
            confidence: confidence.clamp(0.0, 1.0),
            method: "llm".into(),
            created_at: ingested_at,
        });
    }
    out
}

fn is_valid_label(symbol: &str, dimension: &str) -> bool {
    let Some(product) = sectors::get_product_by_symbol(symbol) else {
        return false;
    };
    if product.default_tier == LiquidityTier::Excluded {
        return false;
    }
    let sector = sectors::get_sector_by_symbol(symbol);
    sector
        .map(|s| {
            sector_dimension_codes(&s.code)
                .iter()
                .any(|d| *d == dimension)
        })
        .unwrap_or(false)
}

fn extract_json_object(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') {
        return trimmed.to_string();
    }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[start..=end].to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn record(id: &str) -> NewsRecord {
        NewsRecord {
            id: id.into(),
            source: "jin10".into(),
            category_id: Some(52018),
            title: "钢厂减产".into(),
            summary: "螺纹钢供给收缩".into(),
            url: String::new(),
            display_time: Utc::now().to_rfc3339(),
            content_hash: "x".into(),
            ingested_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn parses_llm_labels_json() {
        let raw = r#"{"labels":[{"news_id":"n1","symbol":"RB0","dimension":"domestic_supply","confidence":0.9}]}"#;
        let labels = parse_llm_response(raw, &[record("n1")]);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].symbol, "RB0");
        assert_eq!(labels[0].method, "llm");
    }
}
