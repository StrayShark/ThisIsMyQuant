use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::NaiveDate;
use reqwest::Client;
use serde_json::Value;

use crate::adapters::jinshi_calendar::{fetch_calendar_range, CalendarFetchOptions};
use crate::config::Config;
use crate::error::AppResult;
use crate::engine::sectors::{get_sector_by_symbol, normalize_product};
use crate::models::{CalendarEvent, NewsItem};

const DEFAULT_HEADERS: [(&str, &str); 3] = [
    ("x-app-id", "fiXF2nOnDycGutVA"),
    ("x-version", "1.0"),
    ("User-Agent", "Mozilla/5.0 (compatible; ThisIsMyQuant/0.1)"),
];

#[derive(Clone)]
pub struct JinshiClient {
    http: Client,
    base_url: String,
    config: Config,
    enabled: bool,
    cache_ttl: f64,
    connected: bool,
    cache: std::sync::Arc<Mutex<HashMap<i64, (f64, Vec<NewsItem>)>>>,
    calendar_cache: std::sync::Arc<Mutex<HashMap<String, (f64, Vec<CalendarEvent>)>>>,
}

impl JinshiClient {
    pub fn new(http: Client, config: &Config) -> Self {
        Self {
            http,
            base_url: config.jinshi_api_base.trim_end_matches('/').to_string(),
            config: config.clone(),
            enabled: config.jinshi_enabled,
            cache_ttl: config.jinshi_cache_ttl,
            connected: false,
            cache: std::sync::Arc::new(Mutex::new(HashMap::new())),
            calendar_cache: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }
        match self.fetch_category(52042, 1).await {
            Ok(items) => {
                self.connected = !items.is_empty();
                log::info!("Jinshi ready, sample={}", items.len());
            }
            Err(e) => {
                log::error!("Jinshi connect failed: {e}");
                self.connected = false;
            }
        }
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.enabled && self.connected
    }

    pub async fn fetch_for_symbol(&self, symbol: &str, limit: usize) -> AppResult<Vec<NewsItem>> {
        if !self.enabled {
            return Ok(vec![]);
        }
        let sector = get_sector_by_symbol(symbol);
        let mut category_ids = vec![52042_i64];
        if let Some(s) = &sector {
            if let Some(cid) = s.jin10_category_id {
                category_ids.insert(0, cid);
            }
        }
        let keywords: Vec<String> = sector
            .map(|s| s.news_keywords.clone())
            .unwrap_or_else(|| vec![normalize_product(symbol)]);

        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for cid in category_ids {
            let items = self.fetch_category(cid, limit).await?;
            for item in items {
                if seen.contains(&item.title) {
                    continue;
                }
                if match_keywords(&item, &keywords) {
                    seen.insert(item.title.clone());
                    results.push(item);
                }
                if results.len() >= limit {
                    break;
                }
            }
            if results.len() >= limit {
                break;
            }
        }
        results.truncate(limit);
        Ok(results)
    }

    pub async fn fetch_latest(&self, limit: usize) -> AppResult<Vec<NewsItem>> {
        if !self.enabled {
            return Ok(vec![]);
        }
        self.fetch_category(52042, limit).await
    }

    pub async fn fetch_calendar_events(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        min_star: u8,
        country: Option<&str>,
    ) -> AppResult<Vec<CalendarEvent>> {
        if !self.enabled {
            return Ok(vec![]);
        }
        let cache_key = format!(
            "{start}|{end}|{min_star}|{}",
            country.unwrap_or("*")
        );
        let now = now_secs();
        if let Ok(cache) = self.calendar_cache.lock() {
            if let Some((ts, items)) = cache.get(&cache_key) {
                if now - ts < self.cache_ttl {
                    return Ok(items.clone());
                }
            }
        }
        let events = fetch_calendar_range(
            &self.http,
            &self.config,
            CalendarFetchOptions {
                start,
                end,
                min_star,
                country,
            },
        )
        .await?;
        if let Ok(mut cache) = self.calendar_cache.lock() {
            cache.insert(cache_key, (now, events.clone()));
        }
        Ok(events)
    }

    pub async fn warm_cache(&self, category_ids: &[i64], limit: usize) -> AppResult<()> {
        for cid in category_ids {
            let _ = self.fetch_category(*cid, limit).await?;
        }
        Ok(())
    }

    pub async fn fetch_category(&self, category_id: i64, limit: usize) -> AppResult<Vec<NewsItem>> {
        let now = now_secs();
        if let Ok(cache) = self.cache.lock() {
            if let Some((ts, items)) = cache.get(&category_id) {
                if now - ts < self.cache_ttl {
                    return Ok(items.iter().take(limit).cloned().collect());
                }
            }
        }

        let url = format!("{}/api/home/item", self.base_url);
        let mut req = self.http.get(&url).query(&[
            ("category_id", category_id.to_string()),
            ("page", "1".into()),
            ("limit", limit.to_string()),
            ("platform", "pc".into()),
        ]);
        for (k, v) in DEFAULT_HEADERS {
            req = req.header(k, v);
        }
        let resp = req.send().await?;
        let payload: Value = resp.json().await?;
        if payload.get("status").and_then(|v| v.as_i64()) != Some(200) {
            return Ok(vec![]);
        }
        let items: Vec<NewsItem> = payload
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|raw| parse_item(raw, category_id))
                    .filter(|i| !i.title.is_empty() && (!i.summary.is_empty() || !i.title.is_empty()))
                    .collect()
            })
            .unwrap_or_default();

        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(category_id, (now, items.clone()));
        }
        Ok(items.into_iter().take(limit).collect())
    }
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn match_keywords(item: &NewsItem, keywords: &[String]) -> bool {
    let text = format!("{} {}", item.title, item.summary);
    keywords.iter().any(|kw| text.contains(kw.as_str()))
}

fn parse_item(raw: &Value, category_id: i64) -> NewsItem {
    let data = raw.get("data").and_then(|d| d.as_object());
    let mut desc = data
        .and_then(|d| d.get("desc"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let bullet_list: Vec<String> = data
        .and_then(|d| d.get("list"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if desc.is_empty() && !bullet_list.is_empty() {
        desc = bullet_list[0].clone();
    } else if !bullet_list.is_empty() && desc.len() < 80 {
        desc = bullet_list.iter().take(3).cloned().collect::<Vec<_>>().join("；");
    }
    let url = data
        .and_then(|d| d.get("url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    NewsItem {
        title: raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        summary: desc.chars().take(500).collect(),
        source: "jin10".into(),
        category_id: Some(category_id),
        display_time: raw
            .get("display_time")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url,
    }
}
