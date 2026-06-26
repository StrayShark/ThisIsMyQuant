//! 从 dimension_summary 抽取结构化事实，并构建 Copilot 追问 prompt。

use chrono::{Duration, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::engine::dimensions;
use crate::models::{AnalysisReport, DimensionFact, NewsItemView};

pub const FOLLOWUP_SYSTEM_PROMPT: &str = "你是一名专业的期货市场分析师助手。\
用户会基于已有分析报告与维度事实进行追问。请结合提供的报告正文、维度要点、相关资讯，\
给出简洁、有针对性的回答。若信息不足请明确说明，不要编造数据。\
务必声明：本分析仅供参考，不构成投资建议。";

/// 将 dimension_summary JSON 转为 dimension_facts 行。
pub fn facts_from_dimension_summary(
    symbol: &str,
    report_id: &str,
    summary: &Value,
    created_at: &str,
) -> Vec<DimensionFact> {
    let sym = symbol.to_uppercase();
    let valid_until = chrono::DateTime::parse_from_rfc3339(created_at)
        .ok()
        .map(|dt| (dt.with_timezone(&Utc) + Duration::days(7)).to_rfc3339());

    let obj = summary.as_object();
    if obj.is_none() {
        return vec![];
    }

    let mut facts = Vec::new();
    for (dim_code, val) in obj.unwrap() {
        let points: Vec<String> = match val {
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .filter(|s| !s.trim().is_empty())
                .collect(),
            Value::String(s) if !s.trim().is_empty() => vec![s.clone()],
            _ => continue,
        };
        for point in points {
            facts.push(DimensionFact {
                id: Uuid::new_v4().to_string(),
                symbol: sym.clone(),
                dimension_code: dim_code.clone(),
                fact: point,
                source_news_id: None,
                source_report_id: Some(report_id.to_string()),
                valid_until: valid_until.clone(),
                created_at: created_at.to_string(),
            });
        }
    }
    facts
}

pub fn render_followup_prompt(
    report: &AnalysisReport,
    facts: &[DimensionFact],
    news: &[NewsItemView],
    question: &str,
) -> String {
    let facts_block = if facts.is_empty() {
        "（暂无结构化维度事实）".into()
    } else {
        let mut by_dim: std::collections::BTreeMap<&str, Vec<&str>> =
            std::collections::BTreeMap::new();
        for f in facts {
            by_dim
                .entry(f.dimension_code.as_str())
                .or_default()
                .push(f.fact.as_str());
        }
        by_dim
            .into_iter()
            .map(|(code, points)| {
                let label = dimensions::dimension_label(code);
                let lines: Vec<String> = points
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| format!("  {}. {}", i + 1, p))
                    .collect();
                format!("### [{code}] {label}\n{}", lines.join("\n"))
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    let news_block = if news.is_empty() {
        "（无关联资讯）".into()
    } else {
        news.iter()
            .enumerate()
            .map(|(i, n)| {
                format!(
                    "{}. [{}] {} — {}",
                    i + 1,
                    n.display_time,
                    n.title,
                    n.summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let dim_summary_block = report
        .dimension_summary
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "（无）".into());

    format!(
        "用户正在追问品种【{symbol}】的分析报告（报告 ID: {report_id}，生成于 {created_at}）。\n\n\
        ## 原报告摘要\n{context_summary}\n\n\
        ## 原报告 dimension_summary\n{dim_summary}\n\n\
        ## 结构化维度事实（最近入库）\n{facts_block}\n\n\
        ## 关联资讯\n{news_block}\n\n\
        ## 原报告正文（节选）\n{content_excerpt}\n\n\
        ## 用户追问\n{question}\n\n\
        请直接回答用户追问，可引用上述报告与事实，语言简洁专业。",
        symbol = report.symbol,
        report_id = report.id,
        created_at = report.created_at,
        context_summary = report.context_summary,
        dim_summary = dim_summary_block,
        facts_block = facts_block,
        news_block = news_block,
        content_excerpt = truncate_chars(&report.content, 2000),
        question = question.trim(),
    )
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_facts_from_summary_object() {
        let summary = json!({"demand": ["地产偏弱", "基建托底"], "inventory": ["去库"]});
        let facts = facts_from_dimension_summary("rb0", "rpt-1", &summary, "2026-06-24T10:00:00Z");
        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].symbol, "RB0");
        assert_eq!(facts[0].dimension_code, "demand");
        assert!(facts[0].source_report_id.as_deref() == Some("rpt-1"));
    }

    #[test]
    fn followup_prompt_contains_question() {
        let report = AnalysisReport {
            id: "r1".into(),
            symbol: "rb0".into(),
            trigger: "manual".into(),
            provider: "doubao".into(),
            prompt_version: "v2".into(),
            context_summary: "summary".into(),
            content: "正文".into(),
            created_at: "2026-06-24T10:00:00Z".into(),
            tags: vec![],
            dimension_summary: None,
            news_ids: vec![],
            anomaly_reason: None,
        };
        let prompt = render_followup_prompt(&report, &[], &[], "库存为何下降？");
        assert!(prompt.contains("库存为何下降？"));
        assert!(prompt.contains("正文"));
    }
}
