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
fn default_one_i64() -> i64 {
    1
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
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_volume: i64,
    pub ask_volume: i64,
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
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_volume: i64,
    pub ask_volume: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimAccount {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub initial_balance: f64,
    pub cash_balance: f64,
    pub equity: f64,
    pub margin_used: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimContractRule {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub contract_multiplier: f64,
    pub price_tick: f64,
    pub margin_rate_long: f64,
    pub margin_rate_short: f64,
    pub commission_mode: String,
    pub commission_open: f64,
    pub commission_close: f64,
    pub commission_close_today: f64,
    #[serde(default)]
    pub min_order_qty: i64,
    #[serde(default = "default_one_i64")]
    pub lot_size: i64,
    #[serde(default)]
    pub max_order_qty: i64,
    #[serde(default)]
    pub daily_price_limit_up: f64,
    #[serde(default)]
    pub daily_price_limit_down: f64,
    #[serde(default)]
    pub default_slippage_ticks: f64,
    #[serde(default)]
    pub is_custom: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRiskRule {
    pub id: String,
    pub account_id: String,
    pub scope: String, // "account" | "symbol"
    pub symbol: Option<String>,
    pub rule_type: String, // "max_lots" | "symbol_margin_ratio" | "risk_ratio" | "loss_limit"
    pub threshold: f64,
    pub action: String, // "reject" | "block_open" | "force_liquidate"
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRiskEvent {
    pub id: String,
    pub account_id: String,
    pub rule_id: String,
    pub triggered_at: String,
    pub description: String,
    pub action_taken: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimOrder {
    pub id: String,
    pub account_id: String,
    pub symbol: String,
    pub name: String,
    pub side: String,
    pub offset: String,
    pub order_type: String,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    #[serde(default)]
    pub oco_group_id: Option<String>,
    #[serde(default)]
    pub parent_order_id: Option<String>,
    #[serde(default)]
    pub tif: Option<String>,
    #[serde(default)]
    pub condition_operator: Option<String>,
    #[serde(default)]
    pub trailing_distance_ticks: Option<f64>,
    #[serde(default)]
    pub trailing_reference_price: Option<f64>,
    pub quantity: i64,
    pub filled_quantity: i64,
    pub status: String,
    pub reason: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimTrade {
    pub id: String,
    pub order_id: String,
    pub account_id: String,
    pub symbol: String,
    pub name: String,
    pub side: String,
    pub offset: String,
    pub price: f64,
    pub quantity: i64,
    pub commission: f64,
    pub slippage: f64,
    pub realized_pnl: f64,
    pub traded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimPosition {
    pub account_id: String,
    pub symbol: String,
    pub name: String,
    pub position_side: String,
    pub today_qty: i64,
    pub history_qty: i64,
    pub total_qty: i64,
    pub avg_price: f64,
    pub margin: f64,
    pub unrealized_pnl: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimEquitySnapshot {
    pub account_id: String,
    pub snapshot_at: String,
    pub equity: f64,
    pub cash_balance: f64,
    pub margin_used: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub risk_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimJournalEntry {
    pub id: String,
    pub account_id: String,
    pub symbol: Option<String>,
    pub trade_id: Option<String>,
    pub report_id: Option<String>,
    pub title: String,
    pub thesis: Option<String>,
    pub execution_review: Option<String>,
    pub emotion_tags: Option<String>,
    pub score: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaceSimOrderRequest {
    pub account_id: String,
    pub symbol: String,
    pub side: String,
    pub offset: String,
    pub order_type: String,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub oco_group_id: Option<String>,
    pub parent_order_id: Option<String>,
    pub tif: Option<String>,
    pub condition_operator: Option<String>,
    pub trailing_distance_ticks: Option<f64>,
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimOrderEstimate {
    pub margin_required: f64,
    pub commission_estimate: f64,
    pub slippage_estimate: f64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimAccountSnapshot {
    pub account: SimAccount,
    pub positions: Vec<SimPosition>,
    pub risk_ratio: f64,
    pub today_pnl: f64,
    pub pending_orders: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayState {
    pub running: bool,
    pub symbol: String,
    pub date: String,
    pub interval: String,
    pub account_id: Option<String>,
    pub current_index: i32,
    pub total_bars: i32,
    pub current_bar_time: Option<String>,
    pub current_price: f64,
    pub speed: i32,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySession {
    pub id: String,
    pub symbol: String,
    pub interval: String,
    pub replay_date: String,
    pub current_index: i32,
    pub speed: i32,
    pub running: bool,
    pub account_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayKlinePayload {
    pub current_index: i32,
    pub total_bars: i32,
    pub bars: Vec<KLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimPerformance {
    pub account_id: String,
    pub total_return: f64,
    pub total_return_pct: f64,
    pub total_pnl: f64,
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub win_rate: f64,
    pub profit_loss_ratio: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub risk_return_ratio: f64,
    pub symbol_contribution: std::collections::HashMap<String, f64>,
    pub hourly_contribution: std::collections::HashMap<String, f64>,
    pub avg_holding_hours: f64,
    pub overnight_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableStats {
    pub name: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseSummary {
    pub path: String,
    pub total_size_bytes: i64,
    pub tables: Vec<DatabaseTableStats>,
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

// ============================================================================
// A 股（股票市场）数据模型
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSymbol {
    pub ts_code: String,
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub market: Option<String>,
    pub industry: Option<String>,
    pub list_date: Option<String>,
    pub status: String,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBar {
    pub ts_code: String,
    pub trade_date: String,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub pre_close: Option<f64>,
    pub pct_chg: Option<f64>,
    pub volume: Option<f64>,
    pub amount: Option<f64>,
    pub turnover_rate: Option<f64>,
    pub adj_factor: Option<f64>,
    pub adjustment: String,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockIndexBar {
    pub index_code: String,
    pub trade_date: String,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub pct_chg: Option<f64>,
    pub volume: Option<f64>,
    pub amount: Option<f64>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBoard {
    pub board_code: String,
    pub board_name: String,
    pub board_type: String,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBoardMember {
    pub board_code: String,
    pub ts_code: String,
    pub weight: Option<f64>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBoardSnapshot {
    pub board_code: String,
    pub trade_date: String,
    pub pct_chg: Option<f64>,
    pub amount: Option<f64>,
    pub turnover_rate: Option<f64>,
    pub net_flow: Option<f64>,
    pub up_count: Option<i64>,
    pub down_count: Option<i64>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockFinancialMetric {
    pub ts_code: String,
    pub report_period: String,
    pub report_type: Option<String>,
    pub revenue: Option<f64>,
    pub revenue_yoy: Option<f64>,
    pub net_profit: Option<f64>,
    pub net_profit_yoy: Option<f64>,
    pub roe: Option<f64>,
    pub gross_margin: Option<f64>,
    pub debt_ratio: Option<f64>,
    pub operating_cash_flow: Option<f64>,
    pub eps: Option<f64>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockValuationSnapshot {
    pub ts_code: String,
    pub trade_date: String,
    pub pe_ttm: Option<f64>,
    pub pb: Option<f64>,
    pub ps_ttm: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub market_cap: Option<f64>,
    pub float_market_cap: Option<f64>,
    pub pe_percentile: Option<f64>,
    pub pb_percentile: Option<f64>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockFactorSnapshot {
    pub ts_code: String,
    pub factor_date: String,
    pub momentum: Option<f64>,
    pub quality: Option<f64>,
    pub valuation: Option<f64>,
    pub growth: Option<f64>,
    pub volatility: Option<f64>,
    pub liquidity: Option<f64>,
    pub capital_flow: Option<f64>,
    pub score: Option<f64>,
    pub factor_version: String,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScreenTemplate {
    pub id: String,
    pub name: String,
    pub criteria_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScreenResult {
    pub id: String,
    pub template_id: Option<String>,
    pub name: String,
    pub criteria_json: String,
    pub result_json: String,
    pub trade_date: Option<String>,
    pub report_period: Option<String>,
    pub source_summary: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockWatchlist {
    pub id: String,
    pub name: String,
    pub symbols: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// A 股 Command 请求 / 响应 DTO
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct StockSymbolsQuery {
    pub query: Option<String>,
    pub industry: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StockBarsRequest {
    pub ts_code: String,
    pub adjustment: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StockDetailQuery {
    pub ts_code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StockIndustriesQuery {
    pub board_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StockIndustryDetailQuery {
    pub board_code: String,
    pub trade_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StockDataSyncRequest {
    pub scope: String,
    pub symbols: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockDataSyncStatus {
    pub task_id: String,
    pub scope: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockIndexQuote {
    pub index_code: String,
    pub name: String,
    pub close: Option<f64>,
    pub pct_chg: Option<f64>,
    pub amount: Option<f64>,
    pub trade_date: Option<String>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockMarketBreadth {
    pub trade_date: Option<String>,
    pub up_count: i64,
    pub down_count: i64,
    pub flat_count: i64,
    pub limit_up_count: i64,
    pub limit_down_count: i64,
    pub total_amount: Option<f64>,
    pub prev_amount: Option<f64>,
    pub amount_change_pct: Option<f64>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockBoardView {
    pub board_code: String,
    pub board_name: String,
    pub board_type: String,
    pub pct_chg: Option<f64>,
    pub amount: Option<f64>,
    pub net_flow: Option<f64>,
    pub up_count: Option<i64>,
    pub down_count: Option<i64>,
    pub trade_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockBoardDetailView {
    pub board: StockBoard,
    pub snapshot: Option<StockBoardSnapshot>,
    pub top_stocks: Vec<StockSymbolSnapshot>,
    pub bottom_stocks: Vec<StockSymbolSnapshot>,
    pub members: Vec<StockSymbolSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockSymbolSnapshot {
    pub ts_code: String,
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub industry: Option<String>,
    pub close: Option<f64>,
    pub pct_chg: Option<f64>,
    pub amount: Option<f64>,
    pub market_cap: Option<f64>,
    pub pe_ttm: Option<f64>,
    pub pb: Option<f64>,
    pub trade_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AStockDashboardView {
    pub indices: Vec<StockIndexQuote>,
    pub breadth: StockMarketBreadth,
    pub boards: Vec<StockBoardView>,
    pub trade_date: Option<String>,
    pub source: String,
    pub updated_at: String,
    pub quality: StockDataQuality,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockDataQuality {
    pub status: String,
    pub message: Option<String>,
    pub last_success_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockDetailView {
    pub symbol: StockSymbol,
    pub latest_bar: Option<StockBar>,
    pub latest_valuation: Option<StockValuationSnapshot>,
    pub latest_financial: Option<StockFinancialMetric>,
    pub factor_scores: Option<StockFactorSnapshot>,
    pub related_boards: Vec<StockBoard>,
    pub quality: StockDataQuality,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StockScreenerRequest {
    pub criteria_json: String,
    pub name: Option<String>,
    pub save_template: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockScreenerResultView {
    pub id: String,
    pub name: String,
    pub criteria_json: String,
    pub trade_date: Option<String>,
    pub report_period: Option<String>,
    pub rows: Vec<StockSymbolSnapshot>,
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveStockWatchlistRequest {
    pub id: Option<String>,
    pub name: String,
    pub symbols: Vec<String>,
}

// ============================================================================
// A 股模拟组合数据模型
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPaperAccount {
    pub id: String,
    pub name: String,
    pub initial_balance: f64,
    pub cash_balance: f64,
    pub market_value: f64,
    pub total_equity: f64,
    pub total_cost: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPaperOrder {
    pub id: String,
    pub account_id: String,
    pub ts_code: String,
    pub name: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<f64>,
    pub quantity: i64,
    pub filled_quantity: i64,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPaperPosition {
    pub account_id: String,
    pub ts_code: String,
    pub name: String,
    pub quantity: i64,
    pub available_quantity: i64,
    pub avg_cost: f64,
    pub total_cost: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPaperTrade {
    pub id: String,
    pub order_id: String,
    pub account_id: String,
    pub ts_code: String,
    pub name: String,
    pub side: String,
    pub price: f64,
    pub quantity: i64,
    pub commission: f64,
    pub traded_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateStockPaperAccountRequest {
    pub name: String,
    pub initial_balance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaceStockPaperOrderRequest {
    pub account_id: String,
    pub ts_code: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<f64>,
    pub quantity: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelStockPaperOrderRequest {
    pub account_id: String,
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockPaperPortfolioView {
    pub account: StockPaperAccount,
    pub positions: Vec<StockPaperPosition>,
    pub orders: Vec<StockPaperOrder>,
    pub trades: Vec<StockPaperTrade>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockPaperOrderEstimate {
    pub estimated_amount: f64,
    pub commission: f64,
    pub stamp_tax: f64,
    pub transfer_fee: f64,
    pub total_cost: f64,
}

// ============================================================================
// CMC 重构：统一市场资产与自选（P0）
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAsset {
    pub symbol: String,
    pub name: String,
    pub market: String, // "futures" | "stock"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turnover: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sparkline: Option<Vec<f64>>,
    pub quality: String,
    pub source: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watched: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_qty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketOverview {
    pub futures_sectors: Vec<MarketSectorBrief>,
    pub a_stock_indices: Vec<MarketIndexBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_breadth: Option<MarketBreadthBrief>,
    pub watchlist_move_count: i64,
    pub data_source_health: std::collections::HashMap<String, String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketSectorBrief {
    pub code: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pct_chg: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketIndexBrief {
    pub code: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pct_chg: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketBreadthBrief {
    pub up_count: i64,
    pub down_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketAssetQuery {
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub quality: Option<String>,
    pub watched: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_turnover: Option<f64>,
    pub query: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAssetSearchResult {
    pub assets: Vec<MarketAsset>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketLeaderboardQuery {
    pub category: String,
    pub market: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketLeaderboard {
    pub category: String,
    pub label: String,
    pub assets: Vec<MarketAsset>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetSparklineQuery {
    pub symbol: String,
    pub market: String,
    pub points: Option<i64>,
}

// 统一自选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistGroup {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: String,
    pub group_id: String,
    pub asset_type: String, // "futures" | "stock"
    pub symbol: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_pct: Option<f64>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveWatchlistGroupRequest {
    pub id: Option<String>,
    pub name: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveWatchlistItemRequest {
    pub id: Option<String>,
    pub group_id: String,
    pub asset_type: String,
    pub symbol: String,
    pub name: String,
    pub notes: Option<String>,
    pub alert_price: Option<f64>,
    pub alert_pct: Option<f64>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WatchlistSummary {
    pub total_count: i64,
    pub futures_count: i64,
    pub stock_count: i64,
    pub move_count: i64,
    pub event_count: i64,
}

// 统一事件资讯（P1 预留）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub id: String,
    pub title: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub display_time: String,
    pub importance: String,
    pub event_type: String,
    pub affected_symbols: Vec<String>,
    pub affected_sectors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub created_at: String,
}

// 引用式 AI（P1 预留）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSource {
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiReportSummary {
    pub id: String,
    pub task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    pub content: String,
    pub sources: Vec<AiSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_date: Option<String>,
    pub disclaimer: String,
    pub provider: String,
    pub created_at: String,
}

// ============================================================================
// CMC 重构：P1 事件资讯中心
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct MarketEventQuery {
    pub source: Option<String>,
    pub symbol: Option<String>,
    pub sector: Option<String>,
    pub importance: Option<String>,
    pub event_type: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketEventListResult {
    pub events: Vec<MarketEvent>,
    pub total: i64,
    pub by_source: std::collections::HashMap<String, i64>,
}

// ============================================================================
// CMC 重构：P1 数据库资产中心
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDomainTimeRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDomain {
    pub code: String,
    pub name: String,
    pub description: String,
    pub record_count: i64,
    pub size_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<DataDomainTimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    pub source: String,
    pub quality: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataDomainActionRequest {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataDomainActionResult {
    pub success: bool,
    pub domain: String,
    pub action: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseDomainSummary {
    pub path: String,
    pub total_size_bytes: i64,
    pub domains: Vec<DataDomain>,
    pub updated_at: String,
}

// ============================================================================
// CMC 重构：P1 引用式 AI
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct AiSummaryRequest {
    pub task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_assets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTask {
    pub id: String,
    pub task_type: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_symbol: Option<String>,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiTaskListResult {
    pub tasks: Vec<AiTask>,
    pub running: i64,
}
