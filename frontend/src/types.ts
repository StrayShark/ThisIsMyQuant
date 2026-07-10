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
  /** green_up=绿涨红跌，red_up=红涨绿跌 */
  quote_color_scheme: "green_up" | "red_up";
  /** dark | light | system | cursor | matrix */
  theme: "dark" | "light" | "system" | "cursor" | "matrix";
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
  core_product_count: number;
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
  preferences_path?: string;
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
  quote_status: {
    quote_count: number;
    stale_count: number;
    stale_after_secs: number;
    newest_timestamp?: string | null;
    max_age_secs?: number | null;
  };
  llm_health: Record<string, boolean>;
  llm_last_errors: Record<string, string>;
  questdb_configured: boolean;
  questdb_online: boolean;
  overseas: Record<string, unknown>;
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

export interface DecisionFlowItem {
  id: string;
  title: string;
  summary: string;
  source: string;
  display_time: string;
  symbol?: string | null;
  product_name?: string | null;
  sector?: string | null;
  dimension_code?: string | null;
  dimension_label?: string | null;
  impact: "bullish" | "bearish" | "neutral" | string;
  confidence: number;
}

export interface FactorSignal {
  label: string;
  value: string;
  signal: string;
  detail: string;
}

export interface FactorSnapshot {
  sector: string;
  symbol: string;
  product_name: string;
  updated_at: string;
  quality: string;
  signals: FactorSignal[];
}

export interface AlertSignalView {
  symbol: string;
  product_name: string;
  sector: string;
  severity: string;
  reason: string;
  change_pct: number;
  timestamp: string;
}

export interface ReportWorkflowItem {
  trigger: string;
  label: string;
  status: string;
  report_id?: string | null;
  symbol?: string | null;
  created_at?: string | null;
  summary: string;
}

export interface OverseasLinkView {
  local_symbol: string;
  local_name: string;
  overseas_symbol: string;
  overseas_name: string;
  driver: string;
  transmission: string;
  status: string;
}

export interface ProfessionalDashboard {
  decision_flow: DecisionFlowItem[];
  factors: FactorSnapshot[];
  alerts: AlertSignalView[];
  report_workflow: ReportWorkflowItem[];
  overseas_links: OverseasLinkView[];
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

export interface RealtimeQuote {
  symbol: string;
  last_price: number;
  bid_price?: number;
  ask_price?: number;
  bid_volume?: number;
  ask_volume?: number;
  prev_close: number;
  change_pct: number;
  timestamp: string;
}

export interface WsQuoteMessage {
  type: "quote";
  symbol: string;
  last_price: number;
  bid_price?: number;
  ask_price?: number;
  bid_volume?: number;
  ask_volume?: number;
  prev_close: number;
  change_pct: number;
  timestamp: string;
}

export interface WsNotification {
  type: "notification";
  level: "info" | "warn" | "error";
  title: string;
  body: string;
  link?: string;
}

export type SimOrderSide = "buy" | "sell";
export type SimOrderOffset = "open" | "close" | "close_today" | "close_yesterday";
export type SimOrderType =
  | "market"
  | "limit"
  | "stop"
  | "stop_limit"
  | "take_profit"
  | "take_profit_limit"
  | "trailing_stop"
  | "condition";
export type SimOrderStatus = "pending" | "open" | "partially_filled" | "filled" | "cancelled" | "rejected";
export type SimPositionSide = "long" | "short";

export interface SimAccount {
  id: string;
  name: string;
  currency: string;
  initial_balance: number;
  cash_balance: number;
  equity: number;
  margin_used: number;
  realized_pnl: number;
  unrealized_pnl: number;
  status: "active" | "frozen" | "closed";
  created_at: string;
  updated_at: string;
}

export interface SimContractRule {
  symbol: string;
  name: string;
  exchange: Exchange;
  contract_multiplier: number;
  price_tick: number;
  margin_rate_long: number;
  margin_rate_short: number;
  commission_mode: "per_hand" | "per_amount" | "mixed";
  commission_open: number;
  commission_close: number;
  commission_close_today: number;
  min_order_qty: number;
  lot_size: number;
  max_order_qty: number;
  daily_price_limit_up: number;
  daily_price_limit_down: number;
  default_slippage_ticks: number;
  is_custom: boolean;
  updated_at: string;
}

export interface SimRiskRule {
  id: string;
  account_id: string;
  scope: "account" | "symbol";
  symbol?: string | null;
  rule_type: "max_lots" | "symbol_margin_ratio" | "risk_ratio" | "loss_limit";
  threshold: number;
  action: "reject" | "block_open" | "force_liquidate";
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface SimRiskEvent {
  id: string;
  account_id: string;
  rule_id: string;
  triggered_at: string;
  description: string;
  action_taken: string;
}

export interface SimOrder {
  id: string;
  account_id: string;
  symbol: string;
  name: string;
  side: SimOrderSide;
  offset: SimOrderOffset;
  order_type: SimOrderType;
  price: number | null;
  trigger_price: number | null;
  stop_loss_price: number | null;
  take_profit_price: number | null;
  oco_group_id: string | null;
  parent_order_id: string | null;
  tif: string | null;
  condition_operator: string | null;
  trailing_distance_ticks: number | null;
  quantity: number;
  filled_quantity: number;
  status: SimOrderStatus;
  reason: string | null;
  source: string;
  created_at: string;
  updated_at: string;
}

export interface SimTrade {
  id: string;
  order_id: string;
  account_id: string;
  symbol: string;
  name: string;
  side: SimOrderSide;
  offset: SimOrderOffset;
  price: number;
  quantity: number;
  commission: number;
  slippage: number;
  realized_pnl: number;
  traded_at: string;
}

export interface SimPosition {
  account_id: string;
  symbol: string;
  name: string;
  position_side: SimPositionSide;
  today_qty: number;
  history_qty: number;
  total_qty: number;
  avg_price: number;
  margin: number;
  unrealized_pnl: number;
  updated_at: string;
}

export interface SimEquitySnapshot {
  account_id: string;
  snapshot_at: string;
  equity: number;
  cash_balance: number;
  margin_used: number;
  realized_pnl: number;
  unrealized_pnl: number;
  risk_ratio: number;
}

export interface SimJournalEntry {
  id: string;
  account_id: string;
  symbol?: string | null;
  trade_id?: string | null;
  report_id?: string | null;
  title: string;
  thesis?: string | null;
  execution_review?: string | null;
  emotion_tags?: string | null;
  score?: number | null;
  created_at: string;
  updated_at: string;
}

export interface SimAccountSnapshot {
  account: SimAccount;
  positions: SimPosition[];
  risk_ratio: number;
  today_pnl: number;
  pending_orders: number;
}

export interface PlaceSimOrderRequest {
  account_id: string;
  symbol: string;
  side: SimOrderSide;
  offset: SimOrderOffset;
  order_type: SimOrderType;
  price?: number | null;
  trigger_price?: number | null;
  stop_loss_price?: number | null;
  take_profit_price?: number | null;
  oco_group_id?: string | null;
  parent_order_id?: string | null;
  tif?: string | null;
  condition_operator?: string | null;
  trailing_distance_ticks?: number | null;
  quantity: number;
}

export interface SimPerformance {
  account_id: string;
  total_return: number;
  total_return_pct: number;
  total_pnl: number;
  max_drawdown: number;
  max_drawdown_pct: number;
  win_rate: number;
  profit_loss_ratio: number;
  avg_win: number;
  avg_loss: number;
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  risk_return_ratio: number;
  symbol_contribution: Record<string, number>;
  hourly_contribution: Record<string, number>;
  avg_holding_hours: number;
  overnight_count: number;
}

export interface SimOrderEstimate {
  margin_required: number;
  commission_estimate: number;
  slippage_estimate: number;
  total_cost: number;
}

export interface ReplayState {
  running: boolean;
  symbol: string;
  date: string;
  interval: string;
  account_id?: string | null;
  current_index: number;
  total_bars: number;
  current_bar_time?: string | null;
  current_price: number;
  speed: number;
  completed: boolean;
}

export interface ReplayKlinePayload {
  current_index: number;
  total_bars: number;
  bars: KLine[];
}

export interface DatabaseTableStats {
  name: string;
  row_count: number;
  size_bytes: number;
  last_updated?: string | null;
}

export interface DatabaseSummary {
  path: string;
  total_size_bytes: number;
  tables: DatabaseTableStats[];
}

export interface WsSimOrderUpdateMessage {
  type: "sim-order-update";
  account_id: string;
}

export interface WsSimAccountUpdateMessage {
  type: "sim-account-update";
  account_id: string;
}

export interface WsAnomalyPositionRiskMessage {
  type: "anomaly-position-risk";
  symbol: string;
  account_id: string;
  account_name: string;
  position_side: string;
  position_qty: number;
  avg_price: number;
  current_price: number;
  unrealized_pnl: number;
  pnl_change_if_hit: number;
  risk_ratio: number;
  description: string;
}

export type WsMessage =
  | WsKlineMessage
  | WsTickMessage
  | WsQuoteMessage
  | WsNotification
  | WsSimOrderUpdateMessage
  | WsSimAccountUpdateMessage
  | WsAnomalyPositionRiskMessage
  | { type: "ping" | "pong" | "system" };

// ============================================================================
// A 股（股票市场）类型
// ============================================================================

export interface StockSymbol {
  ts_code: string;
  symbol: string;
  name: string;
  exchange: string;
  market?: string | null;
  industry?: string | null;
  list_date?: string | null;
  status: string;
  source: string;
  updated_at: string;
}

export interface StockBar {
  ts_code: string;
  trade_date: string;
  open?: number | null;
  high?: number | null;
  low?: number | null;
  close?: number | null;
  pre_close?: number | null;
  pct_chg?: number | null;
  volume?: number | null;
  amount?: number | null;
  turnover_rate?: number | null;
  adj_factor?: number | null;
  adjustment: string;
  source: string;
  updated_at: string;
}

export interface StockIndexBar {
  index_code: string;
  trade_date: string;
  open?: number | null;
  high?: number | null;
  low?: number | null;
  close?: number | null;
  pct_chg?: number | null;
  volume?: number | null;
  amount?: number | null;
  source: string;
  updated_at: string;
}

export interface StockBoard {
  board_code: string;
  board_name: string;
  board_type: string;
  source: string;
  updated_at: string;
}

export interface StockBoardMember {
  board_code: string;
  ts_code: string;
  weight?: number | null;
  source: string;
  updated_at: string;
}

export interface StockBoardSnapshot {
  board_code: string;
  trade_date: string;
  pct_chg?: number | null;
  amount?: number | null;
  turnover_rate?: number | null;
  net_flow?: number | null;
  up_count?: number | null;
  down_count?: number | null;
  source: string;
  updated_at: string;
}

export interface StockFinancialMetric {
  ts_code: string;
  report_period: string;
  report_type?: string | null;
  revenue?: number | null;
  revenue_yoy?: number | null;
  net_profit?: number | null;
  net_profit_yoy?: number | null;
  roe?: number | null;
  gross_margin?: number | null;
  debt_ratio?: number | null;
  operating_cash_flow?: number | null;
  eps?: number | null;
  source: string;
  updated_at: string;
}

export interface StockValuationSnapshot {
  ts_code: string;
  trade_date: string;
  pe_ttm?: number | null;
  pb?: number | null;
  ps_ttm?: number | null;
  dividend_yield?: number | null;
  market_cap?: number | null;
  float_market_cap?: number | null;
  pe_percentile?: number | null;
  pb_percentile?: number | null;
  source: string;
  updated_at: string;
}

export interface StockFactorSnapshot {
  ts_code: string;
  factor_date: string;
  momentum?: number | null;
  quality?: number | null;
  valuation?: number | null;
  growth?: number | null;
  volatility?: number | null;
  liquidity?: number | null;
  capital_flow?: number | null;
  score?: number | null;
  factor_version: string;
  source: string;
  updated_at: string;
}

export interface StockScreenTemplate {
  id: string;
  name: string;
  criteria_json: string;
  created_at: string;
  updated_at: string;
}

export interface StockScreenResult {
  id: string;
  template_id?: string | null;
  name: string;
  criteria_json: string;
  result_json: string;
  trade_date?: string | null;
  report_period?: string | null;
  source_summary?: string | null;
  created_at: string;
}

export interface StockWatchlist {
  id: string;
  name: string;
  symbols: string[];
  created_at: string;
  updated_at: string;
}

export interface StockDataQuality {
  status: string;
  message?: string | null;
  last_success_at?: string | null;
}

export interface StockIndexQuote {
  index_code: string;
  name: string;
  close?: number | null;
  pct_chg?: number | null;
  amount?: number | null;
  trade_date?: string | null;
  source: string;
  updated_at: string;
}

export interface StockMarketBreadth {
  trade_date?: string | null;
  up_count: number;
  down_count: number;
  flat_count: number;
  limit_up_count: number;
  limit_down_count: number;
  total_amount?: number | null;
  prev_amount?: number | null;
  amount_change_pct?: number | null;
  source: string;
  updated_at: string;
}

export interface StockBoardView {
  board_code: string;
  board_name: string;
  board_type: string;
  pct_chg?: number | null;
  amount?: number | null;
  net_flow?: number | null;
  up_count?: number | null;
  down_count?: number | null;
  trade_date?: string | null;
}

export interface StockSymbolSnapshot {
  ts_code: string;
  symbol: string;
  name: string;
  exchange: string;
  industry?: string | null;
  close?: number | null;
  pct_chg?: number | null;
  amount?: number | null;
  market_cap?: number | null;
  pe_ttm?: number | null;
  pb?: number | null;
  trade_date?: string | null;
}

export interface StockBoardDetailView {
  board: StockBoard;
  snapshot?: StockBoardSnapshot | null;
  top_stocks: StockSymbolSnapshot[];
  bottom_stocks: StockSymbolSnapshot[];
  members: StockSymbolSnapshot[];
}

export interface AStockDashboardView {
  indices: StockIndexQuote[];
  breadth: StockMarketBreadth;
  boards: StockBoardView[];
  trade_date?: string | null;
  source: string;
  updated_at: string;
  quality: StockDataQuality;
}

export interface StockDetailView {
  symbol: StockSymbol;
  latest_bar?: StockBar | null;
  latest_valuation?: StockValuationSnapshot | null;
  latest_financial?: StockFinancialMetric | null;
  factor_scores?: StockFactorSnapshot | null;
  related_boards: StockBoard[];
  quality: StockDataQuality;
}

export interface StockScreenerResultView {
  id: string;
  name: string;
  criteria_json: string;
  trade_date?: string | null;
  report_period?: string | null;
  rows: StockSymbolSnapshot[];
  count: number;
}

export interface StockDataSyncStatus {
  task_id: string;
  scope: string;
  status: string;
  message: string;
}

export interface StockSymbolsQuery {
  query?: string | null;
  industry?: string | null;
  limit?: number | null;
}

export interface StockScreenerRequest {
  criteria_json: string;
  name?: string | null;
  save_template?: boolean | null;
}

// A 股模拟组合

export interface StockPaperAccount {
  id: string;
  name: string;
  initial_balance: number;
  cash_balance: number;
  market_value: number;
  total_equity: number;
  total_cost: number;
  realized_pnl: number;
  unrealized_pnl: number;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface StockPaperOrder {
  id: string;
  account_id: string;
  ts_code: string;
  name: string;
  side: string;
  order_type: string;
  price?: number | null;
  quantity: number;
  filled_quantity: number;
  status: string;
  reason?: string | null;
  created_at: string;
  updated_at: string;
}

export interface StockPaperPosition {
  account_id: string;
  ts_code: string;
  name: string;
  quantity: number;
  available_quantity: number;
  avg_cost: number;
  total_cost: number;
  market_value: number;
  unrealized_pnl: number;
  updated_at: string;
}

export interface StockPaperTrade {
  id: string;
  order_id: string;
  account_id: string;
  ts_code: string;
  name: string;
  side: string;
  price: number;
  quantity: number;
  commission: number;
  traded_at: string;
}

export interface StockPaperPortfolioView {
  account: StockPaperAccount;
  positions: StockPaperPosition[];
  orders: StockPaperOrder[];
  trades: StockPaperTrade[];
}

export interface StockPaperOrderEstimate {
  estimated_amount: number;
  commission: number;
  stamp_tax: number;
  transfer_fee: number;
  total_cost: number;
}

export interface CreateStockPaperAccountRequest {
  name: string;
  initial_balance: number;
}

export interface PlaceStockPaperOrderRequest {
  account_id: string;
  ts_code: string;
  side: string;
  order_type: string;
  price?: number | null;
  quantity: number;
}

export interface CancelStockPaperOrderRequest {
  account_id: string;
  order_id: string;
}
