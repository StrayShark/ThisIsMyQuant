//! 金十资讯入库 + 规则/LLM 分类。

use crate::adapters::{JinshiClient, LlmRouter};
use crate::config::NewsClassifyConfig;
use crate::db::Database;
use crate::engine::{news_classifier, news_llm_classifier, sectors};
use crate::error::AppResult;
use crate::models::{news_content_hash, NewsItem, NewsRecord};

pub struct IngestDeps<'a> {
    pub jinshi: &'a JinshiClient,
    pub db: &'a Database,
    pub llm: Option<&'a LlmRouter>,
    pub classify_cfg: &'a NewsClassifyConfig,
    pub default_llm_provider: &'a str,
}

pub async fn ingest_poll(
    deps: &IngestDeps<'_>,
    limit_per_category: usize,
) -> AppResult<(usize, usize)> {
    if !deps.jinshi.is_connected() {
        return Ok((0, 0));
    }

    let categories = sectors::default_news_category_ids();
    let mut new_items = 0usize;
    let mut new_labels = 0usize;

    for cid in categories {
        let items = deps.jinshi.fetch_category(cid, limit_per_category).await?;
        for item in items {
            let record = news_record_from_item(&item);
            if deps.db.news_hash_exists(&record.content_hash)? {
                continue;
            }
            deps.db.save_news(&record)?;
            new_items += 1;
            let labels = news_classifier::classify(&record);
            new_labels += deps.db.save_classifications(&labels)?;
        }
    }

    let llm_labels = classify_pending_with_llm(deps).await?;
    new_labels += llm_labels;

    if new_items > 0 || llm_labels > 0 {
        log::info!(
            "news ingest: {new_items} new, {new_labels} classifications (incl. LLM {llm_labels})"
        );
    }
    Ok((new_items, new_labels))
}

async fn classify_pending_with_llm(deps: &IngestDeps<'_>) -> AppResult<usize> {
    if !deps.classify_cfg.enabled {
        return Ok(0);
    }
    let Some(llm) = deps.llm else {
        return Ok(0);
    };
    if llm.available_providers().is_empty() {
        return Ok(0);
    }

    let batch = deps.classify_cfg.batch_size.clamp(1, 20);
    let pending = deps.db.get_unclassified_news(batch as i64)?;
    if pending.is_empty() {
        return Ok(0);
    }

    let provider = classify_provider(deps);
    let labels = news_llm_classifier::classify_batch(llm, provider, &pending).await?;
    if labels.is_empty() {
        log::debug!(
            "LLM classify returned no labels for {} items",
            pending.len()
        );
        return Ok(0);
    }
    let merged: Vec<_> = pending
        .iter()
        .flat_map(|news| {
            let rule = news_classifier::classify(news);
            let llm_for_news: Vec<_> = labels
                .iter()
                .filter(|l| l.news_id == news.id)
                .cloned()
                .collect();
            news_classifier::merge_classifications(rule, llm_for_news)
        })
        .collect();
    deps.db.save_classifications(&merged)
}

fn classify_provider<'a>(deps: &'a IngestDeps<'_>) -> Option<&'a str> {
    if deps.classify_cfg.provider.is_empty() {
        if deps.default_llm_provider.is_empty() {
            None
        } else {
            Some(deps.default_llm_provider)
        }
    } else {
        Some(&deps.classify_cfg.provider)
    }
}

fn news_record_from_item(item: &NewsItem) -> NewsRecord {
    let hash = news_content_hash(&item.title, &item.summary);
    NewsRecord {
        id: format!("news-{}", &hash[..16]),
        source: item.source.clone(),
        category_id: item.category_id,
        title: item.title.clone(),
        summary: item.summary.clone(),
        url: item.url.clone(),
        display_time: item.display_time.clone(),
        content_hash: hash,
        ingested_at: chrono::Utc::now().to_rfc3339(),
    }
}
