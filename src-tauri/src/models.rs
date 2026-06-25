use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub trace_id: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".into(),
            data: Some(data),
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
            data: None,
            trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub symbol: String,
    pub exchange: String,
    pub name: String,
    pub product: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    #[serde(default = "default_margin")]
    pub margin_ratio: f64,
    pub listing_date: Option<String>,
    pub expiry_date: Option<String>,
}

fn default_multiplier() -> f64 {
    10.0
}
fn default_margin() -> f64 {
    0.1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: String,
    pub last_price: f64,
    pub volume: i64,
    pub open_interest: i64,
    pub bid_price: f64,
    pub bid_volume: i64,
    pub ask_price: f64,
    pub ask_volume: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLine {
    pub symbol: String,
    pub interval: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub turnover: f64,
    pub start_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub id: String,
    pub symbol: String,
    pub trigger: String,
    pub provider: String,
    pub prompt_version: String,
    pub context_summary: String,
    pub content: String,
    pub created_at: String,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension_summary: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub news_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionFact {
    pub id: String,
    pub symbol: String,
    pub dimension_code: String,
    pub fact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_news_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_report_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowupMessage {
    pub id: String,
    pub report_id: String,
    pub symbol: String,
    pub question: String,
    pub answer: String,
    pub provider: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionView {
    pub code: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KlineUpdateEvent {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub symbol: String,
    pub interval: String,
    pub data: KlineBarData,
}

#[derive(Debug, Clone, Serialize)]
pub struct KlineBarData {
    pub t: i64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationEvent {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub level: String,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub feeds: std::collections::HashMap<String, bool>,
    pub llm: std::collections::HashMap<String, bool>,
    pub db: bool,
    pub akshare: AkshareHealth,
    pub poll: Option<PollStatus>,
    pub news_poll: Option<NewsPollStatus>,
    pub realtime: RealtimeHealth,
    pub jinshi: JinshiHealth,
}

#[derive(Debug, Clone, Serialize)]
pub struct AkshareHealth {
    pub history: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PollStatus {
    pub running: bool,
    pub interval: f64,
    pub symbols: Vec<String>,
    pub symbol_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewsPollStatus {
    pub running: bool,
    pub interval: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeHealth {
    pub available: bool,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JinshiHealth {
    pub enabled: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub pub_time: String,
    pub country: String,
    pub name: String,
    pub star: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affect: Option<String>,
    pub status: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub summary: String,
    pub source: String,
    pub category_id: Option<i64>,
    pub display_time: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsRecord {
    pub id: String,
    pub source: String,
    pub category_id: Option<i64>,
    pub title: String,
    pub summary: String,
    pub url: String,
    pub display_time: String,
    pub content_hash: String,
    pub ingested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsClassification {
    pub news_id: String,
    pub symbol: String,
    pub dimension_code: String,
    pub confidence: f32,
    pub method: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsClassificationView {
    pub symbol: String,
    pub dimension_code: String,
    pub dimension_label: String,
    pub confidence: f32,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItemView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub category_id: Option<i64>,
    pub display_time: String,
    pub url: String,
    pub classifications: Vec<NewsClassificationView>,
}

pub fn news_content_hash(title: &str, summary: &str) -> String {
    use sha2::{Digest, Sha256};
    let normalized = format!(
        "{}|{}",
        title.trim().to_lowercase(),
        summary.trim().to_lowercase()
    );
    let digest = Sha256::digest(normalized.as_bytes());
    format!("{:x}", digest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquiditySnapshot {
    pub symbol: String,
    pub volume_20d: f64,
    pub turnover_20d: f64,
    pub score: f64,
    pub tier: String,
    pub scored_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppSettingsView {
    pub akshare_enabled: bool,
    pub akshare_realtime_enabled: bool,
    pub realtime_poll_interval: f64,
    pub watchlist: Vec<String>,
    pub jinshi_enabled: bool,
    pub jinshi_poll_interval: f64,
    pub default_llm_provider: String,
    pub llm_providers: Vec<String>,
    pub daily_analysis_cron: String,
    pub realtime_analysis_interval: u64,
    pub scheduler_daily_running: bool,
    pub scheduler_realtime_running: bool,
    pub database_path: String,
    pub news_classify_enabled: bool,
    pub news_classify_batch: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TriggerAnalysisResult {
    pub report_id: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisDeltaEvent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisDoneEvent {
    pub status: String,
    pub report_id: String,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension_summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FollowupDeltaEvent {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FollowupDoneEvent {
    pub status: String,
    pub report_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_id: Option<String>,
}

pub fn dt_to_iso(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

pub fn parse_dt(s: &str) -> Option<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|n| n.and_utc())
        })
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
                .ok()
                .map(|n| n.and_utc())
        })
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|n| n.and_utc())
        })
}
