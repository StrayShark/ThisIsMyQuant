//! 资讯规则分类：品种 + 分析维度。

use std::collections::HashMap;

use crate::engine::dimensions::{self, dimension_keywords};
use crate::engine::sectors::{self, LiquidityTier, ProductSector};
use crate::models::{NewsClassification, NewsRecord};

const MIN_CONFIDENCE: f32 = 0.5;

/// 合并规则与 LLM 分类结果，同键取较高置信度；置信度相同时优先 LLM。
pub fn merge_classifications(
    rule: Vec<NewsClassification>,
    llm: Vec<NewsClassification>,
) -> Vec<NewsClassification> {
    let mut map: HashMap<(String, String), NewsClassification> = HashMap::new();
    for label in rule.into_iter().chain(llm) {
        let key = (label.symbol.clone(), label.dimension_code.clone());
        map
            .entry(key)
            .and_modify(|existing| {
                if label.confidence > existing.confidence
                    || (label.confidence == existing.confidence && label.method == "llm")
                {
                    *existing = label.clone();
                }
            })
            .or_insert(label);
    }
    map.into_values().collect()
}

/// 对单条资讯做规则分类，返回待写入 DB 的标签。
pub fn classify(news: &NewsRecord) -> Vec<NewsClassification> {
    let text = format!("{} {}", news.title, news.summary);
    let mut scores: HashMap<(String, String), f32> = HashMap::new();

    for sector in sectors::all_sectors() {
        let category_boost = category_match(news.category_id, sector.jin10_category_id);

        for product in &sector.products {
            if product.default_tier == LiquidityTier::Excluded {
                continue;
            }

            let symbol_score = symbol_match(&text, product, sector);
            let base = (symbol_score + category_boost).min(1.0);

            if base < MIN_CONFIDENCE && category_boost < 0.3 {
                continue;
            }

            for dim_code in dimensions::sector_dimension_codes(&sector.code) {
                let dim_hit = dimension_keywords(dim_code)
                    .iter()
                    .any(|kw| text.contains(*kw));
                let mut score = base;
                if dim_hit {
                    score = (score + 0.4).min(1.0);
                } else if base < MIN_CONFIDENCE {
                    continue;
                }

                if score >= MIN_CONFIDENCE {
                    if !dimension_allowed(&text, dim_code) {
                        continue;
                    }
                    let key = (product.symbol.to_uppercase(), dim_code.to_string());
                    scores
                        .entry(key)
                        .and_modify(|v| *v = v.max(score))
                        .or_insert(score);
                }
            }
        }
    }

    if scores.is_empty() {
        append_macro_only_labels(&text, &mut scores);
    }

    let now = news.ingested_at.clone();
    scores
        .into_iter()
        .map(|((symbol, dimension_code), confidence)| NewsClassification {
            news_id: news.id.clone(),
            symbol,
            dimension_code,
            confidence,
            method: "rule".into(),
            created_at: now.clone(),
        })
        .collect()
}

fn category_match(news_cat: Option<i64>, sector_cat: Option<i64>) -> f32 {
    match (news_cat, sector_cat) {
        (Some(n), Some(s)) if n == s => 0.3,
        _ => 0.0,
    }
}

fn symbol_match(text: &str, product: &sectors::FutureProduct, sector: &ProductSector) -> f32 {
    let mut score = 0.0f32;
    if text.contains(product.name.as_str()) {
        score += 0.5;
    }
    if text.to_lowercase().contains(product.code.as_str()) {
        score += 0.3;
    }
    if sector
        .news_keywords
        .iter()
        .any(|kw| text.contains(kw.as_str()))
    {
        score += 0.5;
    }
    score.min(1.0)
}

/// 中美宏观边界：中国 CPI/LPR 归 macro，美国 CPI/非农归 overseas_finance。
fn dimension_allowed(text: &str, dimension_code: &str) -> bool {
    let china_macro = is_china_macro(text);
    let us_macro = is_us_macro(text);

    match dimension_code {
        "macro" => !us_macro || china_macro,
        "overseas_finance" => !china_macro || us_macro,
        _ => true,
    }
}

fn is_china_macro(text: &str) -> bool {
    text.contains("中国CPI")
        || text.contains("中国PPI")
        || text.contains("LPR")
        || text.contains("央行")
        || text.contains("国家统计局")
        || text.contains("财新")
        || (text.contains("CPI") && text.contains("中国"))
        || (text.contains("PPI") && text.contains("中国"))
        || (text.contains("制造业PMI") && text.contains("中国"))
}

fn is_us_macro(text: &str) -> bool {
    text.contains("美联储")
        || text.contains("FOMC")
        || text.contains("Fed")
        || text.contains("鲍威尔")
        || text.contains("Powell")
        || text.contains("非农")
        || text.contains("NFP")
        || text.contains("美国CPI")
        || text.contains("美国PPI")
        || text.contains("美国就业")
        || text.contains("美国通胀")
        || (text.contains("CPI") && !text.contains("中国"))
        || (text.contains("PPI") && !text.contains("中国"))
}

fn append_macro_only_labels(text: &str, scores: &mut HashMap<(String, String), f32>) {
    let china = is_china_macro(text);
    let us = is_us_macro(text);
    if !china && !us {
        return;
    }

    for sector in sectors::all_sectors() {
        let dims = dimensions::sector_dimension_codes(&sector.code);
        for product in &sector.products {
            if product.default_tier == LiquidityTier::Excluded {
                continue;
            }
            let sym = product.symbol.to_uppercase();
            if china && dims.contains(&"macro") && dimension_allowed(text, "macro") {
                scores
                    .entry((sym.clone(), "macro".into()))
                    .or_insert(0.72);
            }
            if us && dims.contains(&"overseas_finance") && dimension_allowed(text, "overseas_finance") {
                scores
                    .entry((sym, "overseas_finance".into()))
                    .or_insert(0.72);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_news(title: &str, summary: &str, category_id: i64) -> NewsRecord {
        NewsRecord {
            id: "n1".into(),
            source: "jin10".into(),
            category_id: Some(category_id),
            title: title.into(),
            summary: summary.into(),
            url: String::new(),
            display_time: Utc::now().to_rfc3339(),
            content_hash: "hash".into(),
            ingested_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn classifies_steel_demand_news() {
        let news = sample_news(
            "螺纹钢社会库存下降",
            "钢厂铁水产量回升，地产需求仍偏弱",
            52018,
        );
        let labels = classify(&news);
        assert!(labels.iter().any(|l| l.symbol == "RB0" && l.dimension_code == "inventory"));
        assert!(labels.iter().any(|l| l.symbol == "RB0" && l.dimension_code == "demand"));
    }

    #[test]
    fn classifies_us_macro_for_metals() {
        let news = sample_news(
            "美国CPI超预期 美联储维持高利率",
            "黄金承压，美元指数走强，铜价震荡",
            52019,
        );
        let labels = classify(&news);
        assert!(
            labels
                .iter()
                .any(|l| l.symbol == "AU0" && l.dimension_code == "overseas_finance"),
            "expected AU0 overseas_finance, got {:?}",
            labels
        );
    }

    #[test]
    fn classifies_us_cpi_for_metals() {
        let news = sample_news(
            "美国CPI超预期，美联储降息预期降温",
            "核心PPI仍偏高，鲍威尔暗示利率将在更高水平维持更久",
            52019,
        );
        let labels = classify(&news);
        assert!(
            labels
                .iter()
                .any(|l| l.dimension_code == "overseas_finance" && (l.symbol == "AU0" || l.symbol == "CU0")),
            "expected overseas_finance label for metals, got {:?}",
            labels
        );
    }

    #[test]
    fn classifies_china_cpi_as_macro_not_overseas() {
        let news = sample_news(
            "中国CPI同比上涨0.3%",
            "国家统计局公布5月通胀数据",
            52042,
        );
        let labels = classify(&news);
        assert!(
            labels.iter().any(|l| l.dimension_code == "macro"),
            "expected macro for 中国CPI, got {:?}",
            labels
        );
        assert!(
            !labels.iter().any(|l| l.dimension_code == "overseas_finance"),
            "中国CPI should not be overseas_finance, got {:?}",
            labels
        );
    }

    #[test]
    fn classifies_lpr_as_macro() {
        let news = sample_news(
            "央行下调LPR 10个基点",
            "1年期与5年期LPR同步下调，支持实体经济",
            52042,
        );
        let labels = classify(&news);
        assert!(
            labels.iter().any(|l| l.dimension_code == "macro"),
            "LPR should map to macro, got {:?}",
            labels
        );
        assert!(
            !labels.iter().any(|l| l.dimension_code == "overseas_finance"),
            "LPR should not be overseas_finance, got {:?}",
            labels
        );
    }

    #[test]
    fn merge_prefers_higher_confidence() {
        let rule = vec![NewsClassification {
            news_id: "n1".into(),
            symbol: "AU0".into(),
            dimension_code: "macro".into(),
            confidence: 0.6,
            method: "rule".into(),
            created_at: "t".into(),
        }];
        let llm = vec![NewsClassification {
            news_id: "n1".into(),
            symbol: "AU0".into(),
            dimension_code: "macro".into(),
            confidence: 0.85,
            method: "llm".into(),
            created_at: "t".into(),
        }];
        let merged = merge_classifications(rule, llm);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].confidence, 0.85);
        assert_eq!(merged[0].method, "llm");
    }
}
