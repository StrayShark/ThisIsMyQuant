/** Playwright E2E 用的 API Mock，模拟 Rust Command 响应。 */
import { STATIC_FUTURES_CATALOG } from "@/data/futures";
import type { AnalysisReport, AppSettings, CalendarEvent, Contract, DimensionFact, DimensionView, FollowupMessage, Interval, KLine, UserPreferences } from "@/types";

const MOCK_KLINES: KLine[] = Array.from({ length: 30 }, (_, i) => {
  const base = 3100 + i * 2;
  const t = new Date(Date.now() - (29 - i) * 86400000).toISOString();
  return {
    symbol: "rb0",
    interval: "1d" as Interval,
    open: base,
    high: base + 10,
    low: base - 8,
    close: base + 4,
    volume: 100000 + i * 1000,
    turnover: 0,
    start_time: t,
  };
});

const MOCK_REPORT: AnalysisReport = {
  id: "e2e-report-1",
  symbol: "rb0",
  trigger: "manual",
  provider: "doubao",
  prompt_version: "v2",
  context_summary: "last=3200 change%=1.2 MA5=3180",
  content: "E2E 模拟分析报告：趋势震荡，关注 3150 支撑。",
  created_at: new Date().toISOString(),
  tags: [],
  dimension_summary: {
    demand: ["地产需求偏弱，基建托底有限"],
    inventory: ["社会库存小幅去化"],
  },
  news_ids: ["news-e2e-1"],
};

const MOCK_CONTRACTS: Contract[] = [
  {
    symbol: "RB0",
    exchange: "SHFE",
    name: "螺纹钢",
    product: "rb",
    multiplier: 10,
    margin_ratio: 0.1,
  },
];

const MOCK_DIMENSIONS: DimensionView[] = [
  { code: "demand", label: "需求" },
  { code: "inventory", label: "库存" },
  { code: "domestic_supply", label: "国内供给" },
  { code: "overseas_finance", label: "国外金融环境" },
];

const MOCK_CALENDAR: CalendarEvent[] = [
  {
    id: "cal-e2e-1",
    pub_time: new Date(Date.now() + 86400000).toISOString().slice(0, 16).replace("T", " "),
    country: "美国",
    name: "美国CPI月率",
    star: 5,
    previous: "0.2%",
    consensus: "0.3%",
    actual: null,
    unit: "%",
    status: "scheduled",
    event_type: "data",
  },
  {
    id: "cal-e2e-2",
    pub_time: new Date(Date.now() + 2 * 86400000).toISOString().slice(0, 16).replace("T", " "),
    country: "中国",
    name: "中国官方制造业PMI",
    star: 4,
    previous: "49.5",
    consensus: "49.8",
    actual: "50.1",
    unit: "",
    status: "released",
    event_type: "data",
  },
];

const MOCK_NEWS = [
  {
    id: "news-e2e-1",
    title: "螺纹钢需求偏弱，地产新开工下滑",
    summary: "终端需求恢复缓慢，贸易商观望情绪较浓。",
    source: "jin10",
    display_time: new Date().toISOString(),
    url: "",
    classifications: [
      {
        symbol: "RB0",
        dimension_code: "demand",
        dimension_label: "需求",
        confidence: 0.9,
        method: "rule",
      },
    ],
  },
];

const MOCK_FACTS: DimensionFact[] = [
  {
    id: "fact-1",
    symbol: "RB0",
    dimension_code: "demand",
    fact: "地产需求偏弱，基建托底有限",
    source_report_id: "e2e-report-1",
    created_at: new Date().toISOString(),
  },
];

const MOCK_FOLLOWUPS: FollowupMessage[] = [
  {
    id: "fu-1",
    report_id: "e2e-report-1",
    symbol: "RB0",
    question: "库存下降是否支撑价格？",
    answer: "短期去库对价格有一定支撑，但需关注需求持续性。",
    provider: "doubao",
    created_at: new Date(Date.now() - 3600000).toISOString(),
  },
];

export const e2eMockApi = {
  health: async () => ({
    data: {
      status: "ok",
      feeds: { akshare: true },
      akshare: { history: true },
      jinshi: {
        enabled: true,
        connected: true,
        calendar_ready: true,
        calendar_fetched_at: new Date().toISOString(),
        calendar_cached_events: 2,
      },
      poll: { running: true, interval: 5, symbol_count: 3 },
      realtime: { available: true, source: "market_poll" },
      llm: { doubao: true },
      db: true,
    },
  }),

  listContracts: async () => MOCK_CONTRACTS,

  listProducts: async (params?: { tier?: string }) => {
    const tier = params?.tier ?? "core";
    if (tier === "all") {
      return STATIC_FUTURES_CATALOG.map((s) => ({
        ...s,
        products: s.products.map((p) => ({ ...p, liquidity_tier: "core" as const })),
      }));
    }
    return STATIC_FUTURES_CATALOG;
  },

  getKlines: async () => MOCK_KLINES,

  listDimensions: async () => MOCK_DIMENSIONS,

  listNews: async (params?: { dimension?: string }) => {
    if (params?.dimension && params.dimension !== "demand") return [];
    return MOCK_NEWS;
  },

  listCalendarEvents: async () => MOCK_CALENDAR,

  listUnclassifiedNews: async () => [],

  listNewsByIds: async () => MOCK_NEWS,

  listDimensionFacts: async () => MOCK_FACTS,

  listFollowups: async () => MOCK_FOLLOWUPS,

  listReports: async () => [MOCK_REPORT],

  getReport: async (id: string) => ({ ...MOCK_REPORT, id }),

  getSettings: async (): Promise<AppSettings> => ({
    akshare_enabled: true,
    akshare_realtime_enabled: true,
    realtime_poll_interval: 5,
    watchlist: ["rb2510", "au2512", "IF2512"],
    jinshi_enabled: true,
    jinshi_poll_interval: 300,
    default_llm_provider: "doubao",
    llm_providers: ["doubao", "ollama"],
    schedule_analysis_trigger: "scheduled",
    daily_briefing_enabled: true,
    daily_briefing_hour: 17,
    schedule_interval_hours: 6,
    schedule_enabled: true,
    scheduler_running: true,
    database_path: "data/quant.db",
    news_classify_enabled: true,
    news_classify_batch: 10,
    market_feed: "akshare_poll",
    anomaly_enabled: true,
    encryption_configured: false,
    ollama_configured: true,
    llm_last_errors: {},
    ticks_enabled: true,
    retention_days_klines: 365,
    retention_days_ticks: 14,
    calendar_reminder_enabled: true,
    database_backend: "sqlite",
    questdb_configured: false,
  }),

  reloadConfig: async () => e2eMockApi.getSettings(),

  getLlmSetup: async () => ({
    setup_required: false,
    default_provider: "doubao",
    encryption_ready: true,
    providers: [
      {
        name: "doubao",
        label: "豆包 Doubao",
        default_base_url: "https://ark.cn-beijing.volces.com/api/v3",
        default_model: "doubao-seed-2.0-pro",
        key_required: true,
        configured: true,
        api_key_masked: "****demo",
        base_url: "https://ark.cn-beijing.volces.com/api/v3",
        model: "doubao-seed-2.0-pro",
      },
      {
        name: "deepseek",
        label: "DeepSeek",
        default_base_url: "https://api.deepseek.com",
        default_model: "deepseek-chat",
        key_required: true,
        configured: false,
        api_key_masked: "（未配置）",
        base_url: "https://api.deepseek.com",
        model: "deepseek-chat",
      },
    ],
  }),

  saveLlmSetup: async (_payload: {
    credentials: import("@/types").LlmCredentialInput[];
    default_provider: string;
  }) => e2eMockApi.getLlmSetup(),

  getUserPreferences: async () => ({
    watchlist: ["rb2510", "au2512", "if2512"],
    schedule_enabled: true,
    schedule_interval_hours: 6,
    schedule_analysis_trigger: "scheduled",
    daily_briefing_enabled: true,
    daily_briefing_hour: 17,
    akshare_enabled: true,
    akshare_realtime_enabled: true,
    realtime_poll_interval: 5,
    jinshi_enabled: true,
    jinshi_poll_interval: 300,
    default_llm_provider: "doubao",
    news_classify_enabled: true,
    news_classify_batch: 10,
    anomaly_enabled: true,
    anomaly_price_pct: 1.5,
    anomaly_window_secs: 300,
    anomaly_cooldown_secs: 900,
    backfill_days_daily: 120,
    backfill_days_minute: 5,
    ticks_enabled: true,
    retention_days_klines: 365,
    retention_days_ticks: 14,
    calendar_reminder_enabled: true,
    calendar_reminder_mins: 30,
  }),

  saveUserPreferences: async (prefs: UserPreferences) => prefs,

  exportKlinesCsv: async () =>
    "symbol,interval,start_time,open,high,low,close,volume,turnover\nrb0,1d,2024-01-01T00:00:00Z,100,101,99,100.5,1000,0\n",

  exportReportsCsv: async () =>
    "id,symbol,trigger,provider,prompt_version,created_at,context_summary,content\ne2e-1,rb0,manual,doubao,v4,2024-01-01,ok,content\n",

  importKlinesCsv: async () => ({ imported: 1 }),

  reclassifyNews: async () => ({ labels_saved: 0 }),

  triggerComprehensiveAnalysis: async () => ({
    started: true,
    total: 3,
    includes_data_fetch: true,
  }),

  triggerDataFetch: async () => ({
    calendar_events: 12,
    news_items: 5,
    news_labels: 8,
    klines_symbols: 3,
  }),

  getScheduleStatus: async () => ({
    enabled: true,
    interval_hours: 6,
    cycle_in_progress: false,
    last_cycle_at: null,
    last_data_fetch: null,
    last_analysis_completed: 0,
    last_analysis_total: 0,
    last_error: null,
  }),

  triggerBatchAnalysis: async (params: { symbols: string[] }) => ({
    started: true,
    total: params.symbols.length,
  }),

  getBatchStatus: async () => ({
    running: false,
    total: 0,
    completed: 0,
    current_symbol: null,
    errors: [],
  }),

  getStatusDashboard: async () => ({
    runtime: {
      poll: {
        running: true,
        interval: 5,
        symbols: ["rb2510"],
        symbol_count: 1,
        feed_source: "akshare_poll",
      },
      backfill: { running: false, completed: 3, total: 3, current_symbol: null, last_error: null },
      feed_source: "akshare_poll",
      schedule: {
        enabled: true,
        interval_hours: 6,
        cycle_in_progress: false,
        last_cycle_at: null,
        last_data_fetch: null,
        last_analysis_completed: 0,
        last_analysis_total: 3,
        last_error: null,
      },
    },
    llm_health: { doubao: true, ollama: false },
    llm_last_errors: {},
    questdb_configured: false,
    questdb_online: false,
    overseas_stub: { status: "stub", message: "海外期货 stub" },
    batch_job: { running: false, total: 0, completed: 0, current_symbol: null, errors: [] },
    prompt_version: "v4",
  }),

  probeOllama: async () => false,

  marketSubscribe: async () => ({ subscribed: ["rb2510"] }),

  marketUnsubscribe: async () => ({ unsubscribed: ["rb2510"] }),

  getRuntimeStatus: async () => ({
    poll: {
      running: true,
      interval: 5,
      symbols: ["rb2510"],
      symbol_count: 1,
      feed_source: "akshare_poll",
    },
    backfill: { running: false, completed: 3, total: 3, current_symbol: null, last_error: null },
    feed_source: "akshare_poll",
    schedule: {
      enabled: true,
      interval_hours: 6,
      cycle_in_progress: false,
      last_cycle_at: null,
      last_data_fetch: null,
      last_analysis_completed: 0,
      last_analysis_total: 0,
      last_error: null,
    },
  }),

  getSymbolContext: async (symbol: string) => ({
    product_name: "螺纹钢",
    main_symbol: symbol,
    name: "黑色",
    related_products: [{ symbol: "I0", name: "铁矿石" }],
    drivers: ["地产", "基建"],
  }),

  triggerAnalysis: async (params: { symbol: string }) => ({
    report_id: "e2e-trigger-1",
    symbol: params.symbol,
  }),

  streamAnalysis: async () => {
    const text = "E2E 流式分析片段。";
    const encoder = new TextEncoder();
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({ text })}\n\n`)
        );
        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({ status: "ok" })}\n\n`)
        );
        controller.close();
      },
    });
    return stream.getReader();
  },

  streamFollowup: async () => {
    const text = "E2E 追问回复：库存去化与需求偏弱有关。";
    const encoder = new TextEncoder();
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({ text })}\n\n`)
        );
        controller.enqueue(
          encoder.encode(`data: ${JSON.stringify({ status: "ok" })}\n\n`)
        );
        controller.close();
      },
    });
    return stream.getReader();
  },
};
