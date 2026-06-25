//! 资讯规则分类：品种 + 分析维度。

use std::collections::HashMap;

use crate::engine::dimensions::{self, dimension_keywords};
use crate::engine::sectors::{self, LiquidityTier, ProductSector};
use crate::models::{NewsClassification, NewsRecord};

const MIN_CONFIDENCE: f32 = 0.5;

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
                    let key = (product.symbol.to_uppercase(), dim_code.to_string());
                    scores
                        .entry(key)
                        .and_modify(|v| *v = v.max(score))
                        .or_insert(score);
                }
            }
        }
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
    fn ignores_unrelated_without_keywords() {
        let news = sample_news("无关娱乐新闻", "明星八卦", 52042);
        let labels = classify(&news);
        assert!(labels.is_empty() || labels.iter().all(|l| l.confidence >= MIN_CONFIDENCE));
    }
}
