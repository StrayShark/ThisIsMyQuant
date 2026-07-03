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

    /// 成功返回数据，但 message 携带警告（如日历降级为本地缓存）。
    pub fn ok_warn(data: T, message: impl Into<String>) -> Self {
        Self {
            code: 0,
            message: message.into(),
            data: Some(data),
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
pub struct TickUpdateEvent {
    pub symbol: String,
    pub last_price: f64,
    pub volume: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeQuote {
    pub symbol: String,
    pub last_price: f64,
    pub prev_close: f64,
    pub change_pct: f64,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forming_daily: Option<KLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuoteUpdateEvent {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub symbol: String,
    pub last_price: f64,
    pub prev_close: f64,
    pub change_pct: f64,
    pub timestamp: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly_reason: Option<String>,
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
    pub realtime_enabled: bool,
    #[serde(default)]
    pub llm_last_errors: std::collections::HashMap<String, String>,
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
    pub feed_source: String,
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

#[derive(Debug, Clone, Serialize, Default)]
pub struct QuoteCacheStatus {
    pub quote_count: usize,
    pub stale_count: usize,
    pub stale_after_secs: i64,
    pub newest_timestamp: Option<String>,
    pub max_age_secs: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JinshiHealth {
    pub enabled: bool,
    pub connected: bool,
    pub calendar_ready: bool,
    pub calendar_fetched_at: Option<String>,
    pub calendar_cached_events: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProviderSetupView {
    pub name: String,
    pub label: String,
    pub default_base_url: String,
    pub default_model: String,
    pub key_required: bool,
    pub configured: bool,
    pub api_key_masked: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmSetupStatus {
    pub providers: Vec<LlmProviderSetupView>,
    pub setup_required: bool,
    pub default_provider: String,
    pub encryption_ready: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmCredentialInput {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveLlmSetupPayload {
    pub credentials: Vec<LlmCredentialInput>,
    pub default_provider: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppSettingsView {
    pub akshare_enabled: bool,
    pub akshare_realtime_enabled: bool,
    pub realtime_poll_interval: f64,
    pub core_product_count: usize,
    pub jinshi_enabled: bool,
    pub jinshi_poll_interval: f64,
    pub default_llm_provider: String,
    pub llm_providers: Vec<String>,
    pub schedule_analysis_trigger: String,
    pub daily_briefing_enabled: bool,
    pub daily_briefing_hour: u8,
    pub schedule_interval_hours: u64,
    pub schedule_enabled: bool,
    pub scheduler_running: bool,
    pub database_path: String,
    pub preferences_path: String,
    pub news_classify_enabled: bool,
    pub news_classify_batch: usize,
    pub market_feed: String,
    pub anomaly_enabled: bool,
    pub anomaly_price_pct: f64,
    pub anomaly_window_secs: i64,
    pub anomaly_cooldown_secs: u64,
    pub backfill_days_daily: i64,
    pub backfill_days_minute: i64,
    pub encryption_configured: bool,
    pub llm_keys_masked: Vec<(String, String)>,
    pub ollama_configured: bool,
    pub llm_last_errors: std::collections::HashMap<String, String>,
    pub ticks_enabled: bool,
    pub retention_days_klines: i64,
    pub retention_days_ticks: i64,
    pub calendar_reminder_enabled: bool,
    pub database_backend: String,
    pub questdb_configured: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BackfillStatus {
    pub running: bool,
    pub completed: usize,
    pub total: usize,
    pub current_symbol: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DataFetchSummary {
    pub calendar_events: usize,
    pub news_items: usize,
    pub news_labels: usize,
    pub klines_symbols: usize,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ScheduleStatus {
    pub enabled: bool,
    pub interval_hours: u64,
    pub cycle_in_progress: bool,
    pub last_cycle_at: Option<String>,
    pub last_data_fetch: Option<DataFetchSummary>,
    pub last_analysis_completed: usize,
    pub last_analysis_total: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RuntimeStatusView {
    pub poll: Option<PollStatus>,
    pub backfill: BackfillStatus,
    pub feed_source: Option<String>,
    pub schedule: ScheduleStatus,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BatchJobStatus {
    pub running: bool,
    pub total: usize,
    pub completed: usize,
    pub current_symbol: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusDashboardView {
    pub runtime: RuntimeStatusView,
    pub quote_status: QuoteCacheStatus,
    pub llm_health: std::collections::HashMap<String, bool>,
    pub llm_last_errors: std::collections::HashMap<String, String>,
    pub questdb_configured: bool,
    pub questdb_online: bool,
    pub overseas: serde_json::Value,
    pub batch_job: BatchJobStatus,
    pub prompt_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionFlowItem {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub display_time: String,
    pub symbol: Option<String>,
    pub product_name: Option<String>,
    pub sector: Option<String>,
    pub dimension_code: Option<String>,
    pub dimension_label: Option<String>,
    pub impact: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FactorSignal {
    pub label: String,
    pub value: String,
    pub signal: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FactorSnapshot {
    pub sector: String,
    pub symbol: String,
    pub product_name: String,
    pub updated_at: String,
    pub quality: String,
    pub signals: Vec<FactorSignal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertSignalView {
    pub symbol: String,
    pub product_name: String,
    pub sector: String,
    pub severity: String,
    pub reason: String,
    pub change_pct: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportWorkflowItem {
    pub trigger: String,
    pub label: String,
    pub status: String,
    pub report_id: Option<String>,
    pub symbol: Option<String>,
    pub created_at: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverseasLinkView {
    pub local_symbol: String,
    pub local_name: String,
    pub overseas_symbol: String,
    pub overseas_name: String,
    pub driver: String,
    pub transmission: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfessionalDashboardView {
    pub decision_flow: Vec<DecisionFlowItem>,
    pub factors: Vec<FactorSnapshot>,
    pub alerts: Vec<AlertSignalView>,
    pub report_workflow: Vec<ReportWorkflowItem>,
    pub overseas_links: Vec<OverseasLinkView>,
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
