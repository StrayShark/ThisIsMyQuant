/** 共享类型定义（与后端 core/types.py 对齐）。 */

export type Exchange = "SHFE" | "DCE" | "CZCE" | "CFFEX" | "INE" | "GFEX" | "IB";

export type Interval = "1m" | "5m" | "15m" | "30m" | "1h" | "1d";

export type ReportTrigger = "daily" | "realtime" | "manual" | "anomaly";

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

export interface AppSettings {
  akshare_enabled: boolean;
  akshare_realtime_enabled: boolean;
  realtime_poll_interval: number;
  watchlist: string[];
  jinshi_enabled: boolean;
  jinshi_poll_interval: number;
  default_llm_provider: string;
  llm_providers: string[];
  daily_analysis_cron: string;
  realtime_analysis_interval: number;
  scheduler_daily_running: boolean;
  scheduler_realtime_running: boolean;
  database_path: string;
  news_classify_enabled: boolean;
  news_classify_batch: number;
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
