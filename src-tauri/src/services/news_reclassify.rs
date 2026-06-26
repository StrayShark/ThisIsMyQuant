//! 新闻重分类 API。

use crate::adapters::LlmRouter;
use crate::db::Database;
use crate::engine::{news_classifier, news_llm_classifier};
use crate::error::AppResult;
use crate::models::NewsRecord;

pub async fn reclassify_news(
    db: &Database,
    llm: &LlmRouter,
    news_ids: &[String],
    provider: Option<&str>,
    use_llm: bool,
) -> AppResult<usize> {
    if news_ids.is_empty() {
        return Ok(0);
    }
    let mut records: Vec<NewsRecord> = Vec::new();
    for id in news_ids {
        if let Ok(Some(item)) = db.get_news_by_id(id) {
            records.push(item);
        }
    }
    if records.is_empty() {
        return Ok(0);
    }

    db.delete_classifications_for_news(news_ids)?;

    let mut count = 0usize;
    if use_llm && !llm.available_providers().is_empty() {
        let labels = news_llm_classifier::classify_batch(llm, provider, &records).await?;
        for news in &records {
            let rule = news_classifier::classify(news);
            let llm_for: Vec<_> = labels
                .iter()
                .filter(|l| l.news_id == news.id)
                .cloned()
                .collect();
            let merged = news_classifier::merge_classifications(rule, llm_for);
            count += db.save_classifications(&merged)?;
        }
    } else {
        for news in &records {
            let labels = news_classifier::classify(news);
            count += db.save_classifications(&labels)?;
        }
    }
    Ok(count)
}
