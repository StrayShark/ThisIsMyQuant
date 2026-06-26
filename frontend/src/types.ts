/** 共享类型定义（与后端 core/types.py 对齐）。 */

export type Exchange = "SHFE" | "DCE" | "CZCE" | "CFFEX" | "INE" | "GFEX" | "IB";

export type Interval = "1m" | "5m" | "15m" | "30m" | "1h" | "1d";

export type ReportTrigger =
  | "scheduled"
  | "daily"
  | "realtime"
  | "manual"
  | "anomaly"
  | "tomorrow"
  | "short_term";

export type LiquidityTier = "core" | "watch" | "excluded";

export interface NewsClassificationView {
  symbol: string;
  dimension_code: string;
  dimension_label: string;
  confidence: number;
  method: string;
}

export interface NewsItemView {
  id: string;
  title: string;
  summary: string;
  source: string;
  category_id?: number | null;
  display_time: string;
  url: string;
  classifications: NewsClassificationView[];
}

export interface NewsRecord {
  id: string;
  source: string;
  category_id?: number | null;
  title: string;
  summary: string;
  url: string;
  display_time: string;
  content_hash: string;
  ingested_at: string;
}

export interface CalendarEvent {
  id: string;
  pub_time: string;
  country: string;
  name: string;
  star: number;
  previous?: string | null;
  consensus?: string | null;
  actual?: string | null;
  unit?: string | null;
  affect?: string | null;
  status: string;
  event_type: string;
}

export interface FuturesProduct {
  code: string;
  symbol: string;
  name: string;
  exchange: Exchange;
  liquidity_tier: LiquidityTier;
  liquidity_score?: number | null;
  volume_20d?: number | null;
  turnover_20d?: number | null;
}

export interface FuturesSector {
  code: string;
  name: string;
  description: string;
  jin10_category_id?: number | null;
  drivers?: string[];
  products: FuturesProduct[];
}

export interface Contract {
  symbol: string;
  exchange: Exchange;
  name: string;
  product: string;
  multiplier: number;
  margin_ratio: number;
  listing_date?: string | null;
  expiry_date?: string | null;
}

export interface Tick {
  symbol: string;
  last_price: number;
  volume: number;
  open_interest: number;
  bid_price: number;
  bid_volume: number;
  ask_price: number;
  ask_volume: number;
  timestamp: string;
}

export interface KLine {
  symbol: string;
  interval: Interval;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  turnover: number;
  start_time: string;
}

export interface AnalysisReport {
  id: string;
  symbol: string;
  trigger: ReportTrigger;
  provider: string;
  prompt_version: string;
  context_summary: string;
  content: string;
  created_at: string;
  tags: string[];
  dimension_summary?: Record<string, string[]> | null;
  news_ids?: string[];
  anomaly_reason?: string | null;
}

export interface DimensionView {
  code: string;
  label: string;
}

export interface DimensionFact {
  id: string;
  symbol: string;
  dimension_code: string;
  fact: string;
  source_news_id?: string | null;
  source_report_id?: string | null;
  valid_until?: string | null;
  created_at: string;
}

export interface FollowupMessage {
  id: string;
  report_id: string;
  symbol: string;
  question: string;
  answer: string;
  provider: string;
  created_at: string;
}

/** 应用内可持久化配置（非 .env 密钥项） */
export interface UserPreferences {
  watchlist: string[];
  schedule_enabled: boolean;
  schedule_interval_hours: number;
  schedule_analysis_trigger: string;
  daily_briefing_enabled: boolean;
  daily_briefing_hour: number;
  akshare_enabled: boolean;
  akshare_realtime_enabled: boolean;
  realtime_poll_interval: number;
  jinshi_enabled: boolean;
  jinshi_poll_interval: number;
  default_llm_provider: string;
  news_classify_enabled: boolean;
  news_classify_batch: number;
  anomaly_enabled: boolean;
  anomaly_price_pct: number;
  anomaly_window_secs: number;
  anomaly_cooldown_secs: number;
  backfill_days_daily: number;
  backfill_days_minute: number;
  ticks_enabled: boolean;
  retention_days_klines: number;
  retention_days_ticks: number;
  calendar_reminder_enabled: boolean;
  calendar_reminder_mins: number;
}

export interface LlmProviderSetup {
  name: string;
  label: string;
  default_base_url: string;
  default_model: string;
  key_required: boolean;
  configured: boolean;
  api_key_masked: string;
  base_url: string;
  model: string;
}

export interface LlmSetupStatus {
  providers: LlmProviderSetup[];
  setup_required: boolean;
  default_provider: string;
  encryption_ready: boolean;
}

export interface LlmCredentialInput {
  provider: string;
  api_key: string;
  base_url?: string;
  model?: string;
}

export interface AppSettings {
  akshare_enabled: boolean;
  akshare_realtime_enabled: boolean;
  realtime_poll_interval: number;
  watchlist: string[];
  jinshi_enabled: boolean;
  jinshi_poll_interval: number;
  default_llm_provider: string;
  llm_providers: string[];
  schedule_analysis_trigger: string;
  daily_briefing_enabled: boolean;
  daily_briefing_hour: number;
  schedule_interval_hours: number;
  schedule_enabled: boolean;
  scheduler_running: boolean;
  database_path: string;
  news_classify_enabled: boolean;
  news_classify_batch: number;
  market_feed?: string;
  anomaly_enabled?: boolean;
  anomaly_price_pct?: number;
  anomaly_window_secs?: number;
  anomaly_cooldown_secs?: number;
  backfill_days_daily?: number;
  backfill_days_minute?: number;
  encryption_configured?: boolean;
  llm_keys_masked?: [string, string][];
  ollama_configured?: boolean;
  llm_last_errors?: Record<string, string>;
  ticks_enabled?: boolean;
  retention_days_klines?: number;
  retention_days_ticks?: number;
  calendar_reminder_enabled?: boolean;
  database_backend?: string;
  questdb_configured?: boolean;
}

export interface BatchJobStatus {
  running: boolean;
  total: number;
  completed: number;
  current_symbol?: string | null;
  errors: string[];
}

export interface StatusDashboard {
  runtime: RuntimeStatus;
  llm_health: Record<string, boolean>;
  llm_last_errors: Record<string, string>;
  questdb_configured: boolean;
  questdb_online: boolean;
  overseas_stub: Record<string, unknown>;
  batch_job: BatchJobStatus;
  prompt_version: string;
}

export interface BackfillStatus {
  running: boolean;
  completed: number;
  total: number;
  current_symbol?: string | null;
  last_error?: string | null;
}

export interface DataFetchSummary {
  calendar_events: number;
  news_items: number;
  news_labels: number;
  klines_symbols: number;
}

export interface ScheduleStatus {
  enabled: boolean;
  interval_hours: number;
  cycle_in_progress: boolean;
  last_cycle_at?: string | null;
  last_data_fetch?: DataFetchSummary | null;
  last_analysis_completed: number;
  last_analysis_total: number;
  last_error?: string | null;
}

export interface RuntimeStatus {
  poll?: {
    running: boolean;
    interval: number;
    symbols: string[];
    symbol_count: number;
    feed_source: string;
  } | null;
  backfill: BackfillStatus;
  feed_source?: string | null;
  schedule: ScheduleStatus;
}

/** 后端统一响应体。 */
export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
  trace_id: string;
}

/** WS 推送的 K 线增量消息。 */
export interface WsKlineMessage {
  type: "kline";
  symbol: string;
  interval: Interval;
  data: { t: number; o: number; h: number; l: number; c: number; v: number };
}

export interface WsTickMessage {
  type: "tick";
  symbol: string;
  data: { last: number; vol: number; ts: number };
}

export interface WsNotification {
  type: "notification";
  level: "info" | "warn" | "error";
  title: string;
  body: string;
  link?: string;
}

export type WsMessage = WsKlineMessage | WsTickMessage | WsNotification | { type: "ping" | "pong" | "system" };
