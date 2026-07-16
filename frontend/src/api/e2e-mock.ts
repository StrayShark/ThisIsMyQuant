/** Playwright E2E 用的 API Mock，模拟 Rust Command 响应。 */
import { STATIC_FUTURES_CATALOG } from "@/data/futures";
import type {
  AnalysisReport,
  AppSettings,
  CalendarEvent,
  Contract,
  DimensionFact,
  DimensionView,
  FollowupMessage,
  Interval,
  KLine,
  PlaceSimOrderRequest,
  ReplayState,
  SimAccount,
  SimAccountSnapshot,
  SimContractRule,
  SimEquitySnapshot,
  SimJournalEntry,
  SimOrder,
  SimPosition,
  SimRiskRule,
  SimTrade,
  UserPreferences,
  AStockDashboardView,
  StockBar,
  StockBoardDetailView,
  StockDetailView,
  StockScreenTemplate,
  StockScreenerResultView,
  StockFinancialMetric,
  StockWatchlist,
  StockSymbol,
  StockPaperAccount,
  StockPaperPortfolioView,
  StockPaperOrder,
  StockPaperOrderEstimate,
  MarketAsset,
  MarketOverview,
  MarketLeaderboard,
  MarketFilters,
  MarketAssetSearchResult,
  WatchlistGroup,
  WatchlistItem,
  WatchlistSummary,
  MarketEvent,
  MarketEventListResult,
  DatabaseDomainSummary,
  DataDomainActionResult,
  DataDomainCode,
  AiSummaryRequest,
  AiReportSummary,
  AiTaskListResult,
} from "@/types";

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
    technical: ["短期偏多，关注突破前高"],
  },
  news_ids: ["news-e2e-1"],
};

const MOCK_REPORTS: AnalysisReport[] = [
  MOCK_REPORT,
  {
    id: "e2e-report-au",
    symbol: "au0",
    trigger: "scheduled",
    provider: "doubao",
    prompt_version: "v2",
    context_summary: "last=520 change%=-0.8 MA5=525",
    content: "黄金偏空，美元走强压制金价。",
    created_at: new Date(Date.now() - 3600000).toISOString(),
    tags: [],
    dimension_summary: {
      overseas_finance: ["美联储鹰派预期，金价承压偏空"],
      flow: ["ETF 持仓小幅流出"],
    },
    news_ids: [],
  },
  {
    id: "e2e-report-cu",
    symbol: "cu0",
    trigger: "daily",
    provider: "doubao",
    prompt_version: "v2",
    context_summary: "last=78000 change%=2.1 MA5=77200",
    content: "沪铜偏多，供应扰动支撑价格。",
    created_at: new Date(Date.now() - 7200000).toISOString(),
    tags: [],
    dimension_summary: {
      domestic_supply: ["冶炼厂检修，供应偏紧"],
      technical: ["突破均线，偏多格局"],
    },
    news_ids: [],
  },
];

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

let mockReplayState: ReplayState = {
  running: false,
  symbol: "",
  date: "",
  interval: "",
  account_id: null,
  current_index: 0,
  total_bars: 0,
  current_bar_time: null,
  current_price: 0,
  speed: 1,
  completed: false,
};

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

const MOCK_STOCK_SYMBOLS: StockSymbol[] = [
  {
    ts_code: "600000.SH",
    symbol: "600000",
    name: "浦发银行",
    exchange: "SH",
    market: "主板",
    industry: "银行",
    list_date: "1999-11-10",
    status: "active",
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  {
    ts_code: "000001.SZ",
    symbol: "000001",
    name: "平安银行",
    exchange: "SZ",
    market: "主板",
    industry: "银行",
    list_date: "1991-04-03",
    status: "active",
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  {
    ts_code: "000333.SZ",
    symbol: "000333",
    name: "美的集团",
    exchange: "SZ",
    market: "主板",
    industry: "家用电器",
    list_date: "2013-09-18",
    status: "active",
    source: "mock",
    updated_at: new Date().toISOString(),
  },
];

const MOCK_MARKET_ASSETS: MarketAsset[] = [
  ...(STATIC_FUTURES_CATALOG ?? []).flatMap((sector) =>
    sector.products.map(
      (p): MarketAsset => ({
        symbol: p.symbol,
        name: p.name,
        market: "futures",
        sector: sector.name,
        exchange: p.exchange,
        category: p.code,
        price: 3200,
        change_pct: 0.5,
        change_amount: 16,
        turnover: 1_000_000,
        volume: 5000,
        quality: "live",
        source: "akshare",
        updated_at: new Date().toISOString(),
        watched: false,
      })
    )
  ),
  ...MOCK_STOCK_SYMBOLS.map(
    (s): MarketAsset => ({
      symbol: s.ts_code,
      name: s.name,
      market: "stock",
      industry: s.industry,
      exchange: s.exchange,
      price: 12.5,
      change_pct: 0.8,
      change_amount: 0.1,
      turnover: 1_200_000_000,
      volume: 100_000_000,
      quality: "live",
      source: "mock",
      updated_at: new Date().toISOString(),
      watched: false,
    })
  ),
];

const MOCK_STOCK_BARS: StockBar[] = Array.from({ length: 30 }, (_, i) => {
  const base = 10 + i * 0.05;
  const date = new Date(Date.now() - (29 - i) * 86400000);
  const tradeDate = date.toISOString().slice(0, 10).replace(/-/g, "");
  return {
    ts_code: "600000.SH",
    trade_date: tradeDate,
    open: base,
    high: base + 0.1,
    low: base - 0.08,
    close: base + 0.03,
    pre_close: base - 0.02,
    pct_chg: 0.3,
    volume: 100000 + i * 1000,
    amount: 1000000 + i * 10000,
    turnover_rate: 0.01,
    adjustment: "none",
    source: "mock",
    updated_at: new Date().toISOString(),
  };
});

const MOCK_A_STOCK_DASHBOARD: AStockDashboardView = {
  indices: [
    { index_code: "000001.SH", name: "上证指数", close: 3050.23, pct_chg: 0.42, amount: 420000000000, trade_date: "20260709", source: "mock", updated_at: new Date().toISOString() },
    { index_code: "399001.SZ", name: "深证成指", close: 9780.12, pct_chg: 0.18, amount: 560000000000, trade_date: "20260709", source: "mock", updated_at: new Date().toISOString() },
    { index_code: "399006.SZ", name: "创业板指", close: 1850.45, pct_chg: -0.25, amount: 210000000000, trade_date: "20260709", source: "mock", updated_at: new Date().toISOString() },
    { index_code: "000688.SH", name: "科创50", close: 720.88, pct_chg: 0.65, amount: 89000000000, trade_date: "20260709", source: "mock", updated_at: new Date().toISOString() },
  ],
  breadth: {
    trade_date: "20260709",
    up_count: 2850,
    down_count: 1980,
    flat_count: 120,
    limit_up_count: 45,
    limit_down_count: 8,
    total_amount: 1189000000000,
    prev_amount: 1120000000000,
    amount_change_pct: 6.16,
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  boards: [
    { board_code: "BK0475", board_name: "银行", board_type: "industry", pct_chg: 1.2, amount: 45000000000, net_flow: 1200000000, up_count: 35, down_count: 2, trade_date: "20260709" },
    { board_code: "BK0484", board_name: "半导体", board_type: "industry", pct_chg: 0.85, amount: 62000000000, net_flow: 800000000, up_count: 42, down_count: 18, trade_date: "20260709" },
    { board_code: "BK0428", board_name: "电力", board_type: "industry", pct_chg: -0.6, amount: 28000000000, net_flow: -500000000, up_count: 15, down_count: 28, trade_date: "20260709" },
  ],
  trade_date: "20260709",
  source: "mock",
  updated_at: new Date().toISOString(),
  quality: { status: "available", message: null, last_success_at: "20260709" },
};

const MOCK_STOCK_DETAIL: StockDetailView = {
  symbol: MOCK_STOCK_SYMBOLS[0],
  latest_bar: MOCK_STOCK_BARS[MOCK_STOCK_BARS.length - 1],
  latest_valuation: {
    ts_code: "600000.SH",
    trade_date: "20260709",
    pe_ttm: 4.5,
    pb: 0.42,
    ps_ttm: 1.8,
    dividend_yield: 5.2,
    market_cap: 285000000000,
    float_market_cap: 285000000000,
    pe_percentile: 12.5,
    pb_percentile: 8.3,
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  factor_scores: {
    ts_code: "600000.SH",
    factor_date: "20260709",
    momentum: 55.2,
    quality: 62.1,
    valuation: 88.5,
    growth: 45.3,
    volatility: 30.8,
    liquidity: 78.4,
    capital_flow: null,
    score: 60.05,
    factor_version: "v1",
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  latest_financial: {
    ts_code: "600000.SH",
    report_period: "2025-12-31",
    report_type: "年报",
    revenue: 170000000000,
    revenue_yoy: -3.5,
    net_profit: 42000000000,
    net_profit_yoy: 1.2,
    roe: 7.8,
    gross_margin: null,
    debt_ratio: 92.1,
    operating_cash_flow: null,
    eps: 1.43,
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  related_boards: [],
  quality: { status: "available", message: null, last_success_at: "20260709" },
};

const MOCK_STOCK_FINANCIALS: StockFinancialMetric[] = [
  {
    ts_code: "600000.SH",
    report_period: "2025-12-31",
    report_type: "年报",
    revenue: 170000000000,
    revenue_yoy: -3.5,
    net_profit: 42000000000,
    net_profit_yoy: 1.2,
    roe: 7.8,
    gross_margin: null,
    debt_ratio: 92.1,
    operating_cash_flow: null,
    eps: 1.43,
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  {
    ts_code: "600000.SH",
    report_period: "2025-09-30",
    report_type: "三季报",
    revenue: 125000000000,
    revenue_yoy: -4.1,
    net_profit: 31000000000,
    net_profit_yoy: 0.8,
    roe: 5.9,
    gross_margin: null,
    debt_ratio: 92.3,
    operating_cash_flow: null,
    eps: 1.06,
    source: "mock",
    updated_at: new Date().toISOString(),
  },
];

let MOCK_STOCK_WATCHLISTS: StockWatchlist[] = [];

const MOCK_BOARD_DETAIL: StockBoardDetailView = {
  board: { board_code: "BK0475", board_name: "银行", board_type: "industry", source: "mock", updated_at: new Date().toISOString() },
  snapshot: {
    board_code: "BK0475",
    trade_date: "20260709",
    pct_chg: 1.2,
    amount: 45000000000,
    turnover_rate: 0.8,
    net_flow: 1200000000,
    up_count: 35,
    down_count: 2,
    source: "mock",
    updated_at: new Date().toISOString(),
  },
  top_stocks: MOCK_STOCK_SYMBOLS.slice(0, 2).map((s) => ({
    ...s,
    close: 12.5,
    pct_chg: 1.2,
    amount: 1200000000,
    market_cap: 285000000000,
    pe_ttm: 4.5,
    pb: 0.42,
    trade_date: "20260709",
  })),
  bottom_stocks: [],
  members: MOCK_STOCK_SYMBOLS.slice(0, 2).map((s) => ({
    ...s,
    close: 12.5,
    pct_chg: 1.2,
    amount: 1200000000,
    market_cap: 285000000000,
    pe_ttm: 4.5,
    pb: 0.42,
    trade_date: "20260709",
  })),
};

let MOCK_WATCHLIST_ITEMS: WatchlistItem[] = [
  {
    id: "item-e2e-1",
    group_id: "group-e2e-1",
    asset_type: "futures",
    symbol: "RB0",
    name: "螺纹钢",
    notes: null,
    alert_price: null,
    alert_pct: null,
    sort_order: 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
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
      poll: { running: true, interval: 5, symbol_count: 26 },
      realtime: { available: true, source: "market_poll" },
      realtime_enabled: true,
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

  listReports: async () => MOCK_REPORTS,

  getReport: async (id: string) => ({ ...MOCK_REPORT, id }),

  getSettings: async (): Promise<AppSettings> => ({
    akshare_enabled: true,
    akshare_realtime_enabled: true,
    realtime_poll_interval: 5,
    core_product_count: 26,
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
    preferences_path: "data/user_preferences.json",
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
      {
        name: "kimi",
        label: "Kimi / Moonshot",
        default_base_url: "https://api.moonshot.cn/v1",
        default_model: "kimi-k2-0711-preview",
        key_required: true,
        configured: false,
        api_key_masked: "（未配置）",
        base_url: "https://api.moonshot.cn/v1",
        model: "kimi-k2-0711-preview",
      },
    ],
  }),

  saveLlmSetup: async (_payload: {
    credentials: import("@/types").LlmCredentialInput[];
    default_provider: string;
  }) => e2eMockApi.getLlmSetup(),

  getUserPreferences: async () => {
    const prefs = {
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
      quote_color_scheme: "green_up" as const,
      theme: "light" as const,
    };
    return prefs;
  },

  saveUserPreferences: async (prefs: UserPreferences) => prefs,

  exportKlinesCsv: async () =>
    "symbol,interval,start_time,open,high,low,close,volume,turnover\nrb0,1d,2024-01-01T00:00:00Z,100,101,99,100.5,1000,0\n",

  exportReportsCsv: async () =>
    "id,symbol,trigger,provider,prompt_version,created_at,context_summary,content\ne2e-1,rb0,manual,doubao,v4,2024-01-01,ok,content\n",

  importKlinesCsv: async () => ({ imported: 1 }),

  getProfessionalDashboard: async () => ({
    decision_flow: [
      {
        id: "news-e2e-1",
        title: "螺纹钢需求偏弱，地产新开工下滑",
        summary: "终端需求恢复缓慢，贸易商观望情绪较浓。",
        source: "jin10",
        display_time: new Date().toISOString(),
        symbol: "RB0",
        product_name: "螺纹钢",
        sector: "黑色建材",
        dimension_code: "demand",
        dimension_label: "需求",
        impact: "bearish",
        confidence: 0.9,
      },
      {
        id: "news-e2e-2",
        title: "红海扰动延续，集运运价维持高位",
        summary: "绕航导致有效运力收缩，欧线合约波动加大。",
        source: "jin10",
        display_time: new Date(Date.now() - 1800000).toISOString(),
        symbol: "EC0",
        product_name: "集运欧线",
        sector: "航运运价",
        dimension_code: "geopolitics",
        dimension_label: "地缘",
        impact: "bullish",
        confidence: 0.82,
      },
    ],
    factors: [
      {
        sector: "黑色建材",
        symbol: "RB0",
        product_name: "螺纹钢",
        updated_at: new Date().toISOString(),
        quality: "live+history",
        signals: [
          {
            label: "价格动量",
            value: "+1.24%",
            signal: "bullish",
            detail: "来自实时主力报价或最近日 K 收盘变化",
          },
          {
            label: "成交活跃度",
            value: "156000",
            signal: "tracked",
            detail: "使用主力连续日线成交量",
          },
        ],
      },
      {
        sector: "能源化工",
        symbol: "SC0",
        product_name: "原油",
        updated_at: new Date().toISOString(),
        quality: "history",
        signals: [
          {
            label: "核心驱动",
            value: "原油价格 / 装置开工率 / 化工库存",
            signal: "watch",
            detail: "承接后续库存、利润、到港等专源",
          },
        ],
      },
    ],
    alerts: [
      {
        symbol: "RB0",
        product_name: "螺纹钢",
        sector: "黑色建材",
        severity: "medium",
        reason: "主力报价较昨收上涨1.24%",
        change_pct: 1.24,
        timestamp: new Date().toISOString(),
      },
    ],
    report_workflow: [
      {
        trigger: "tomorrow",
        label: "盘前计划",
        status: "ready",
        report_id: "e2e-report-1",
        symbol: "RB0",
        created_at: new Date().toISOString(),
        summary: "last=3200 change%=1.2 MA5=3180",
      },
      {
        trigger: "anomaly",
        label: "盘中异动",
        status: "pending",
        report_id: null,
        symbol: null,
        created_at: null,
        summary: "等待定时任务或手动触发生成",
      },
    ],
    overseas_links: [
      {
        local_symbol: "SC0",
        local_name: "原油",
        overseas_symbol: "CL=F",
        overseas_name: "WTI 原油",
        driver: "能源",
        transmission: "外盘原油影响内盘原油、燃油、沥青和聚酯链",
        status: "tracked",
      },
      {
        local_symbol: "AU0",
        local_name: "黄金",
        overseas_symbol: "GC=F",
        overseas_name: "COMEX 黄金",
        driver: "贵金属",
        transmission: "COMEX 黄金、美元与美债实际利率传导沪金沪银",
        status: "tracked",
      },
    ],
  }),

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

  triggerBatchAnalysis: async (params?: { symbols?: string[]; trigger?: string }) => ({
    started: true,
    total: params?.symbols?.length ?? 26,
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
        symbols: ["RB0"],
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
    quote_status: {
      quote_count: 3,
      stale_count: 0,
      stale_after_secs: 15,
      newest_timestamp: new Date().toISOString(),
      max_age_secs: 2,
    },
    llm_health: { doubao: true, ollama: false },
    llm_last_errors: {},
    questdb_configured: false,
    questdb_online: false,
    overseas: {
      status: "ok",
      message: "Yahoo Finance 海外期货参考源",
      symbols: [
        { symbol: "CL=F", name: "WTI 原油" },
        { symbol: "GC=F", name: "COMEX 黄金" },
      ],
    },
    batch_job: { running: false, total: 0, completed: 0, current_symbol: null, errors: [] },
    prompt_version: "v4",
  }),

  probeOllama: async () => false,

  marketSubscribe: async () => ({ subscribed: ["RB0"] }),
  getRealtimeQuotes: async () => [],

  marketUnsubscribe: async () => ({ unsubscribed: ["RB0"] }),

  getRuntimeStatus: async () => ({
    poll: {
      running: true,
      interval: 5,
      symbols: ["RB0"],
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


  listSimAccounts: async () => MOCK_SIM_ACCOUNTS,
  createSimAccount: async (payload: { name: string; initial_balance: number }) => ({
    ...MOCK_SIM_ACCOUNT,
    name: payload.name,
    initial_balance: payload.initial_balance,
    cash_balance: payload.initial_balance,
    equity: payload.initial_balance,
  }),
  resetSimAccount: async () => MOCK_SIM_ACCOUNT,
  getSimAccountSnapshot: async (): Promise<SimAccountSnapshot> => ({
    account: MOCK_SIM_ACCOUNT,
    positions: MOCK_SIM_POSITIONS,
    risk_ratio: 0.12,
    today_pnl: 1200,
    pending_orders: 1,
  }),
  listSimPositions: async () => MOCK_SIM_POSITIONS,
  listSimOrders: async (params?: { account_id?: string; status?: string; limit?: number }) => {
    const order = {
      id: "order-e2e-new",
      account_id: "acc-e2e-1",
      symbol: "RB0",
      name: "螺纹钢",
      side: "buy" as const,
      offset: "open" as const,
      order_type: "condition" as const,
      price: null,
      trigger_price: 3150,
      stop_loss_price: 3100,
      take_profit_price: 3300,
      oco_group_id: null,
      parent_order_id: null,
      tif: "GTC" as const,
      condition_operator: null,
      trailing_distance_ticks: null,
      quantity: 1,
      filled_quantity: 0,
      status: "open" as const,
      reason: null,
      source: "manual" as const,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    if (params?.status === "open") return [...MOCK_SIM_ORDERS, order];
    return [...MOCK_SIM_ORDERS, order];
  },
  listSimTrades: async () => MOCK_SIM_TRADES,
  listSimEquityCurve: async () => MOCK_EQUITY_CURVE,
  getSimPerformance: async () => MOCK_PERFORMANCE,
  placeSimOrder: async (payload: PlaceSimOrderRequest): Promise<SimOrder> => ({
    id: "order-e2e-new",
    account_id: payload.account_id,
    symbol: payload.symbol,
    name: "螺纹钢",
    side: payload.side,
    offset: payload.offset,
    order_type: payload.order_type,
    price: payload.price ?? null,
    trigger_price: payload.trigger_price ?? null,
    stop_loss_price: payload.stop_loss_price ?? null,
    take_profit_price: payload.take_profit_price ?? null,
    oco_group_id: payload.oco_group_id ?? null,
    parent_order_id: payload.parent_order_id ?? null,
    tif: payload.tif ?? null,
    condition_operator: payload.condition_operator ?? null,
    trailing_distance_ticks: payload.trailing_distance_ticks ?? null,
    quantity: payload.quantity,
    filled_quantity: 0,
    status: "open",
    reason: null,
    source: "manual",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }),
  cancelSimOrder: async (orderId: string) => {
    const order = MOCK_SIM_ORDERS.find((o) => o.id === orderId);
    return { ...(order ?? MOCK_SIM_ORDERS[0]), status: "cancelled" as const };
  },
  estimateSimOrder: async () => ({
    margin_required: 3200,
    commission_estimate: 6,
    slippage_estimate: 2,
    total_cost: 3208,
  }),
  listSimContractRules: async () => MOCK_SIM_CONTRACT_RULES,
  saveSimContractRule: async (payload: SimContractRule) => payload,
  deleteSimContractRule: async (symbol: string) => symbol,
  listSimRiskRules: async () => MOCK_SIM_RISK_RULES,
  saveSimRiskRule: async (payload: SimRiskRule) => payload,
  deleteSimRiskRule: async (id: string) => id,
  forceLiquidate: async () => [],
  saveSimJournalEntry: async (payload: Partial<SimJournalEntry>) => ({
    id: "journal-e2e-1",
    account_id: payload.account_id ?? "acc-e2e-1",
    symbol: payload.symbol ?? "RB0",
    trade_id: payload.trade_id ?? null,
    report_id: payload.report_id ?? null,
    title: payload.title ?? "E2E 复盘",
    thesis: payload.thesis ?? null,
    execution_review: payload.execution_review ?? null,
    emotion_tags: payload.emotion_tags ?? null,
    score: payload.score ?? null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }),
  listSimJournalEntries: async () => MOCK_JOURNAL_ENTRIES,
  startMarketReplay: async () => {
    mockReplayState = {
      running: true,
      symbol: "RB0",
      date: "2024-01-15",
      interval: "1m",
      account_id: "default",
      current_index: 0,
      total_bars: 30,
      current_bar_time: new Date().toISOString(),
      current_price: 3200,
      speed: 1,
      completed: false,
    };
    return mockReplayState;
  },
  stopMarketReplay: async () => {
    mockReplayState = { ...mockReplayState, running: false };
    return mockReplayState;
  },
  stepMarketReplay: async () => {
    mockReplayState = {
      ...mockReplayState,
      running: true,
      current_index: 1,
      current_bar_time: new Date().toISOString(),
      current_price: 3202,
    };
    return mockReplayState;
  },
  getReplayState: async () => mockReplayState,
  getReplayKlines: async () => {
    const bars = MOCK_KLINES.slice(0, 5);
    return {
      current_index: 4,
      total_bars: 30,
      bars,
    };
  },
  getDatabaseSummary: async () => ({
    path: "data/quant.db",
    total_size_bytes: 1024 * 1024 * 12,
    tables: [
      { name: "klines", row_count: 45200, size_bytes: 1024 * 1024 * 4 },
      { name: "sim_trades", row_count: 12, size_bytes: 1024 * 16 },
      { name: "reports", row_count: 8, size_bytes: 1024 * 64 },
    ],
  }),
  backupDatabase: async () => "data/quant.db.bak",
  prepareDatabaseRestore: async (backupPath: string) =>
    `已校验备份并复制到恢复候选：${backupPath}.restore.pending.db`,

  getDatabaseDomainSummary: async (): Promise<DatabaseDomainSummary> => ({
    path: "data/quant.db",
    total_size_bytes: 1024 * 1024 * 12,
    updated_at: new Date().toISOString(),
    domains: [
      {
        code: "quotes" as DataDomainCode,
        name: "行情报价",
        description: "实时行情与主力连续报价数据",
        record_count: 12480,
        size_bytes: 1024 * 1024 * 1,
        time_range: { start: new Date(Date.now() - 86400000).toISOString(), end: new Date().toISOString() },
        last_updated: new Date().toISOString(),
        source: "akshare",
        quality: "live",
      },
      {
        code: "klines" as DataDomainCode,
        name: "K线数据",
        description: "期货与股票历史K线",
        record_count: 45200,
        size_bytes: 1024 * 1024 * 4,
        time_range: { start: "2024-01-01T00:00:00Z", end: new Date().toISOString() },
        last_updated: new Date(Date.now() - 3600000).toISOString(),
        source: "akshare",
        quality: "history",
      },
      {
        code: "news" as DataDomainCode,
        name: "资讯",
        description: "市场资讯与分类标签",
        record_count: 3890,
        size_bytes: 1024 * 1024 * 2,
        time_range: { start: new Date(Date.now() - 7 * 86400000).toISOString(), end: new Date().toISOString() },
        last_updated: new Date(Date.now() - 1800000).toISOString(),
        source: "jin10",
        quality: "live",
      },
      {
        code: "calendar" as DataDomainCode,
        name: "财经日历",
        description: "全球财经事件与数据公布",
        record_count: 420,
        size_bytes: 1024 * 256,
        time_range: { start: new Date().toISOString(), end: new Date(Date.now() + 7 * 86400000).toISOString() },
        last_updated: new Date(Date.now() - 7200000).toISOString(),
        source: "jin10",
        quality: "history",
      },
      {
        code: "reports" as DataDomainCode,
        name: "研报",
        description: "品种研报与AI摘要",
        record_count: 86,
        size_bytes: 1024 * 1024 * 1,
        time_range: { start: new Date(Date.now() - 30 * 86400000).toISOString(), end: new Date().toISOString() },
        last_updated: new Date(Date.now() - 86400000).toISOString(),
        source: "llm",
        quality: "local",
      },
      {
        code: "simulation" as DataDomainCode,
        name: "模拟交易",
        description: "模拟账户、委托、成交与持仓",
        record_count: 12,
        size_bytes: 1024 * 64,
        last_updated: new Date().toISOString(),
        source: "local",
        quality: "local",
      },
      {
        code: "watchlist" as DataDomainCode,
        name: "自选",
        description: "用户自选分组与标的",
        record_count: 24,
        size_bytes: 1024 * 32,
        last_updated: new Date().toISOString(),
        source: "local",
        quality: "local",
      },
      {
        code: "stocks" as DataDomainCode,
        name: "A股",
        description: "A股股票、指数、财报与估值数据",
        record_count: 5600,
        size_bytes: 1024 * 1024 * 3,
        time_range: { start: "2024-01-01T00:00:00Z", end: new Date().toISOString() },
        last_updated: new Date(Date.now() - 3600000).toISOString(),
        source: "akshare",
        quality: "history",
      },
      {
        code: "settings" as DataDomainCode,
        name: "设置",
        description: "用户偏好与系统配置",
        record_count: 8,
        size_bytes: 1024 * 16,
        last_updated: new Date().toISOString(),
        source: "local",
        quality: "local",
      },
    ],
  }),

  syncDataDomain: async (domain: DataDomainCode): Promise<DataDomainActionResult> => ({
    success: true,
    domain,
    action: "sync",
    message: `数据域 ${domain} 同步任务已启动（mock）`,
  }),

  exportDataDomain: async (domain: DataDomainCode): Promise<DataDomainActionResult> => ({
    success: true,
    domain,
    action: "export",
    message: `数据域 ${domain} 导出成功（mock）`,
    path: `data/export/${domain}.csv`,
  }),

  cleanupDataDomain: async (domain: DataDomainCode): Promise<DataDomainActionResult> => ({
    success: true,
    domain,
    action: "cleanup",
    message: `数据域 ${domain} 已清理（mock）`,
  }),

  generateAiSummary: async (_payload: AiSummaryRequest): Promise<AiReportSummary> => ({
    id: `ai-summary-e2e-${Date.now()}`,
    task_type: _payload.task_type,
    target_symbol: _payload.target_symbol ?? null,
    content: "E2E 模拟 AI 摘要：市场维持震荡，关注事件对相关板块的情绪影响。",
    sources: [
      {
        type: "news",
        title: "E2E 模拟资讯源",
        display_time: new Date().toISOString(),
      },
    ],
    data_date: new Date().toISOString().slice(0, 10),
    disclaimer: "仅供研究与复盘，不构成投资建议",
    provider: "doubao",
    created_at: new Date().toISOString(),
  }),

  listAiTasks: async (): Promise<AiTaskListResult> => ({
    tasks: [
      {
        id: "ai-task-e2e-1",
        task_type: "market_summary",
        status: "done",
        target_symbol: null,
        provider: "doubao",
        error: null,
        created_at: new Date(Date.now() - 3600000).toISOString(),
        updated_at: new Date().toISOString(),
      },
    ],
    running: 0,
  }),

  // A 股 mock
  listStockSymbols: async (params?: { query?: string; industry?: string }) => {
    let result = MOCK_STOCK_SYMBOLS;
    if (params?.query) {
      const q = params.query.toLowerCase();
      result = result.filter(
        (s) =>
          s.ts_code.toLowerCase().includes(q) ||
          s.symbol.toLowerCase().includes(q) ||
          s.name.toLowerCase().includes(q)
      );
    }
    if (params?.industry) {
      result = result.filter((s) => s.industry === params.industry);
    }
    return result;
  },
  getAStockDashboard: async () => MOCK_A_STOCK_DASHBOARD,
  getStockKlines: async () => MOCK_STOCK_BARS,
  getStockDetail: async () => MOCK_STOCK_DETAIL,
  listStockIndustries: async () => MOCK_A_STOCK_DASHBOARD.boards,
  getStockIndustryDetail: async () => MOCK_BOARD_DETAIL,
  runStockScreener: async (): Promise<StockScreenerResultView> => ({
    id: "screen-e2e-1",
    name: "E2E 筛选结果",
    criteria_json: "{}",
    trade_date: "20260709",
    report_period: "2025-12-31",
    rows: MOCK_STOCK_SYMBOLS.map((s) => ({
      ...s,
      close: 12.5,
      pct_chg: 0.8,
      amount: 1200000000,
      market_cap: 285000000000,
      pe_ttm: 4.5,
      pb: 0.42,
      trade_date: "20260709",
    })),
    count: MOCK_STOCK_SYMBOLS.length,
  }),
  saveStockScreen: async (): Promise<StockScreenTemplate> => ({
    id: "template-e2e-1",
    name: "E2E 模板",
    criteria_json: "{}",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }),
  listStockScreenTemplates: async (): Promise<StockScreenTemplate[]> => [
    {
      id: "template-e2e-1",
      name: "E2E 低估值模板",
      criteria_json: JSON.stringify({ min_pe_ttm: 0, max_pe_ttm: 10 }),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  ],
  deleteStockScreenTemplate: async () => null,
  summarizeStockScreen: async (): Promise<AnalysisReport> => ({
    ...MOCK_REPORT,
    id: "stock-screen-summary-e2e",
    content: "E2E 筛选总结：筛选条件集中在低估值银行板块，结果整体 PE 较低、ROE 稳定，需关注资产质量与宏观利率风险。",
    tags: ["a-stock", "screen-summary"],
  }),
  listStockFinancials: async (): Promise<StockFinancialMetric[]> => MOCK_STOCK_FINANCIALS,
  listStockWatchlists: async (): Promise<StockWatchlist[]> => MOCK_STOCK_WATCHLISTS,
  saveStockWatchlist: async (payload: { id?: string; name: string; symbols: string[] }): Promise<StockWatchlist> => {
    const existingIndex = payload.id ? MOCK_STOCK_WATCHLISTS.findIndex((w) => w.id === payload.id) : -1;
    const watchlist: StockWatchlist = {
      id: payload.id ?? `watchlist-${Date.now()}`,
      name: payload.name,
      symbols: payload.symbols,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    if (existingIndex >= 0) {
      MOCK_STOCK_WATCHLISTS[existingIndex] = watchlist;
    } else {
      MOCK_STOCK_WATCHLISTS.push(watchlist);
    }
    return watchlist;
  },
  deleteStockWatchlist: async (id: string) => {
    MOCK_STOCK_WATCHLISTS = MOCK_STOCK_WATCHLISTS.filter((w) => w.id !== id);
    return null;
  },
  triggerStockDataSync: async () => ({
    task_id: "task-e2e-1",
    scope: "all",
    status: "queued",
    message: "A 股同步任务已加入队列（mock）",
  }),

  // A 股模拟组合 mock
  listStockPaperAccounts: async (): Promise<StockPaperAccount[]> => [MOCK_STOCK_PAPER_ACCOUNT],
  createStockPaperAccount: async (payload: { name: string; initial_balance: number }): Promise<StockPaperAccount> => ({
    ...MOCK_STOCK_PAPER_ACCOUNT,
    id: "stock-paper-acc-new",
    name: payload.name,
    initial_balance: payload.initial_balance,
    cash_balance: payload.initial_balance,
    total_equity: payload.initial_balance,
  }),
  getStockPaperPortfolio: async (): Promise<StockPaperPortfolioView> => MOCK_STOCK_PAPER_PORTFOLIO,
  placeStockPaperOrder: async (payload: { side: string; price: number; quantity: number }): Promise<StockPaperOrder> => ({
    id: "stock-paper-order-new",
    account_id: "stock-paper-acc-1",
    ts_code: "600000.SH",
    name: "浦发银行",
    side: payload.side,
    order_type: "limit",
    price: payload.price,
    quantity: payload.quantity,
    filled_quantity: payload.quantity,
    status: "filled",
    reason: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }),
  cancelStockPaperOrder: async (): Promise<StockPaperOrder> => ({
    ...MOCK_STOCK_PAPER_PORTFOLIO.orders[0],
    status: "cancelled",
  }),
  estimateStockPaperOrder: async (params: {
    price: number;
    quantity: number;
    side: string;
  }): Promise<StockPaperOrderEstimate> => {
    const amount = params.price * params.quantity;
    return {
      estimated_amount: amount,
      commission: Math.max(amount * 0.0003, 5),
      stamp_tax: params.side === "sell" ? amount * 0.001 : 0,
      transfer_fee: amount * 0.00001,
      total_cost: amount + Math.max(amount * 0.0003, 5) + (params.side === "sell" ? amount * 0.001 : 0) + amount * 0.00001,
    };
  },

  generateStockSummary: async (): Promise<AnalysisReport> => ({
    ...MOCK_REPORT,
    id: "stock-summary-e2e",
    content: "E2E 模拟个股速览：浦发银行基本面稳健，估值处于历史低位，关注资产质量变化。",
    tags: ["a-stock", "summary"],
  }),

  generateStockPortfolioReview: async (): Promise<AnalysisReport> => ({
    ...MOCK_REPORT,
    id: "stock-portfolio-review-e2e",
    content: "E2E 模拟组合复盘：当前持仓集中在银行板块，波动较低，建议关注行业景气度变化。",
    tags: ["a-stock", "portfolio-review"],
  }),

  generateTradeReview: async () => MOCK_REPORT,

  // CMC 重构：统一市场 API mock
  getMarketOverview: async (): Promise<MarketOverview> => ({
    futures_sectors: STATIC_FUTURES_CATALOG.map((s) => ({
      code: s.code,
      name: s.name,
      pct_chg: 0.5,
    })),
    a_stock_indices: [
      { code: "000001.SH", name: "上证指数", close: 3200, pct_chg: 0.3 },
      { code: "399001.SZ", name: "深证成指", close: 10500, pct_chg: -0.1 },
    ],
    market_breadth: {
      up_count: 2100,
      down_count: 1900,
      total_amount: 8_500_000_000_000,
    },
    watchlist_move_count: 3,
    data_source_health: {
      akshare: "live",
      jin10: "live",
      baostock: "history",
    },
    updated_at: new Date().toISOString(),
  }),

  listMarketAssets: async (
    params: MarketFilters & { sort_by?: string; sort_desc?: boolean; limit?: number; offset?: number }
  ): Promise<MarketAssetSearchResult> => {
    let assets = [...MOCK_MARKET_ASSETS];
    const market = params.market ?? "all";
    if (market !== "all") {
      assets = assets.filter((a) => a.market === market);
    }
    if (params.sector) {
      assets = assets.filter((a) => a.sector === params.sector);
    }
    if (params.industry) {
      assets = assets.filter((a) => a.industry === params.industry);
    }
    if (params.quality) {
      assets = assets.filter((a) => a.quality === params.quality);
    }
    if (params.query) {
      const q = params.query.toLowerCase();
      assets = assets.filter(
        (a) => a.symbol.toLowerCase().includes(q) || a.name.toLowerCase().includes(q)
      );
    }
    if (params.watched) {
      const watchedSymbols = new Set(MOCK_WATCHLIST_ITEMS.map((i) => i.symbol));
      assets = assets.filter((a) => watchedSymbols.has(a.symbol));
    }

    const sortBy = params.sort_by ?? "turnover";
    const desc = params.sort_desc ?? true;
    assets.sort((a, b) => {
      const av = (a as unknown as Record<string, unknown>)[sortBy] ?? 0;
      const bv = (b as unknown as Record<string, unknown>)[sortBy] ?? 0;
      if (typeof av === "string" && typeof bv === "string") {
        return desc ? bv.localeCompare(av) : av.localeCompare(bv);
      }
      const an = Number(av) || 0;
      const bn = Number(bv) || 0;
      return desc ? (bn > an ? 1 : bn < an ? -1 : 0) : (an > bn ? 1 : an < bn ? -1 : 0);
    });

    const limit = params.limit ?? 100;
    const offset = params.offset ?? 0;
    return { assets: assets.slice(offset, offset + limit), total: assets.length };
  },

  getMarketLeaderboard: async (_params: {
    category: string;
    market?: string;
    limit?: number;
  }): Promise<MarketLeaderboard> => ({
    category: "gainers",
    label: "涨幅榜",
    assets: STATIC_FUTURES_CATALOG[0].products.slice(0, 5).map(
      (p): MarketAsset => ({
        symbol: p.symbol,
        name: p.name,
        market: "futures",
        sector: STATIC_FUTURES_CATALOG[0].name,
        exchange: p.exchange,
        price: 3200,
        change_pct: 1.2,
        change_amount: 38,
        turnover: 2_000_000,
        volume: 10_000,
        quality: "live",
        source: "akshare",
        updated_at: new Date().toISOString(),
        watched: false,
      })
    ),
    updated_at: new Date().toISOString(),
  }),

  getAssetSparkline: async (_params: { symbol: string; market: string; points?: number }): Promise<number[]> =>
    Array.from({ length: 24 }, (_, i) => 3100 + Math.sin(i * 0.5) * 50 + i * 2),

  searchAssets: async (query: string, limit = 8): Promise<MarketAssetSearchResult> => {
    const q = query.trim().toLowerCase();
    if (!q) return { assets: [], total: 0 };
    const assets = MOCK_MARKET_ASSETS.filter(
      (a) => a.symbol.toLowerCase().includes(q) || a.name.toLowerCase().includes(q)
    ).slice(0, limit);
    return { assets, total: assets.length };
  },

  // 统一自选 API mock
  listWatchlistGroups: async (): Promise<WatchlistGroup[]> => [
    {
      id: "group-e2e-1",
      name: "默认自选",
      sort_order: 0,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  ],

  createWatchlistGroup: async (payload: { name: string; sort_order?: number }): Promise<WatchlistGroup> => ({
    id: `group-e2e-${Date.now()}`,
    name: payload.name,
    sort_order: payload.sort_order ?? 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }),

  updateWatchlistGroup: async (payload: { id: string; name: string; sort_order?: number }): Promise<WatchlistGroup> => ({
    id: payload.id,
    name: payload.name,
    sort_order: payload.sort_order ?? 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }),

  deleteWatchlistGroup: async (_id: string): Promise<void> => undefined,

  listWatchlistItems: async (groupId?: string): Promise<WatchlistItem[]> => {
    if (!groupId || groupId === "all") return MOCK_WATCHLIST_ITEMS;
    return MOCK_WATCHLIST_ITEMS.filter((i) => i.group_id === groupId);
  },

  addWatchlistItem: async (payload: {
    group_id: string;
    asset_type: "futures" | "stock";
    symbol: string;
    name: string;
    notes?: string;
    alert_price?: number;
    alert_pct?: number;
  }): Promise<WatchlistItem> => {
    const item: WatchlistItem = {
      id: `item-e2e-${Date.now()}`,
      group_id: payload.group_id,
      asset_type: payload.asset_type,
      symbol: payload.symbol,
      name: payload.name,
      notes: payload.notes ?? null,
      alert_price: payload.alert_price ?? null,
      alert_pct: payload.alert_pct ?? null,
      sort_order: 0,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    MOCK_WATCHLIST_ITEMS.push(item);
    return item;
  },

  updateWatchlistItem: async (payload: {
    id: string;
    group_id: string;
    asset_type: "futures" | "stock";
    symbol: string;
    name: string;
    notes?: string;
    alert_price?: number;
    alert_pct?: number;
    sort_order?: number;
  }): Promise<WatchlistItem> => {
    const index = MOCK_WATCHLIST_ITEMS.findIndex((i) => i.id === payload.id);
    const item: WatchlistItem = {
      id: payload.id,
      group_id: payload.group_id,
      asset_type: payload.asset_type,
      symbol: payload.symbol,
      name: payload.name,
      notes: payload.notes ?? null,
      alert_price: payload.alert_price ?? null,
      alert_pct: payload.alert_pct ?? null,
      sort_order: payload.sort_order ?? 0,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    if (index >= 0) {
      MOCK_WATCHLIST_ITEMS[index] = item;
    }
    return item;
  },

  removeWatchlistItem: async (id: string): Promise<void> => {
    MOCK_WATCHLIST_ITEMS = MOCK_WATCHLIST_ITEMS.filter((i) => i.id !== id);
  },

  getWatchlistSummary: async (): Promise<WatchlistSummary> => ({
    total_count: 3,
    futures_count: 2,
    stock_count: 1,
    move_count: 1,
    event_count: 2,
  }),

  getWatchlistEvents: async (): Promise<MarketEvent[]> => [
    {
      id: "event-e2e-1",
      title: "螺纹钢库存数据公布",
      source: "calendar",
      source_id: null,
      source_url: null,
      display_time: new Date().toISOString(),
      importance: "high",
      event_type: "data_release",
      affected_symbols: ["RB0"],
      affected_sectors: ["黑色建材"],
      direction: "neutral",
      summary: null,
      created_at: new Date().toISOString(),
    },
  ],

  // CMC 重构：P1 事件资讯中心
  listMarketEvents: async (params?: {
    source?: string | null;
    symbol?: string | null;
    sector?: string | null;
    importance?: string | null;
    event_type?: string | null;
    start?: string | null;
    end?: string | null;
    limit?: number | null;
  }): Promise<MarketEventListResult> => {
    let events: MarketEvent[] = [
      {
        id: "event-e2e-1",
        title: "螺纹钢库存数据公布",
        source: "calendar",
        source_id: "cal-e2e-1",
        source_url: null,
        display_time: new Date().toISOString(),
        importance: "high",
        event_type: "data_release",
        affected_symbols: ["RB0"],
        affected_sectors: ["黑色建材"],
        direction: "neutral",
        summary: "前值: 500 | 预期: 480 | 公布: 475",
        created_at: new Date().toISOString(),
      },
      {
        id: "event-e2e-2",
        title: "螺纹钢需求偏弱，地产新开工下滑",
        source: "jin10",
        source_id: "news-e2e-1",
        source_url: "",
        display_time: new Date(Date.now() - 3600000).toISOString(),
        importance: "medium",
        event_type: "industry",
        affected_symbols: ["RB0"],
        affected_sectors: ["黑色建材"],
        direction: "bearish",
        summary: "终端需求恢复缓慢，贸易商观望情绪较浓。",
        created_at: new Date(Date.now() - 3600000).toISOString(),
      },
      {
        id: "event-e2e-3",
        title: "美国CPI月率",
        source: "calendar",
        source_id: "cal-e2e-2",
        source_url: null,
        display_time: new Date(Date.now() + 86400000).toISOString(),
        importance: "high",
        event_type: "macro",
        affected_symbols: [],
        affected_sectors: [],
        direction: "neutral",
        summary: "前值: 0.2% | 预期: 0.3%",
        created_at: new Date().toISOString(),
      },
    ];

    if (params?.source && params.source !== "all") {
      events = events.filter((e) => e.source === params.source);
    }
    if (params?.symbol) {
      const sym = params.symbol.toUpperCase();
      events = events.filter((e) =>
        e.affected_symbols.some((s) => s.toUpperCase() === sym)
      );
    }
    if (params?.sector) {
      const sec = params.sector;
      events = events.filter((e) =>
        e.affected_sectors.some((s) => s.includes(sec))
      );
    }
    if (params?.importance && params.importance !== "all") {
      events = events.filter((e) => e.importance === params.importance);
    }
    if (params?.event_type && params.event_type !== "all") {
      events = events.filter((e) => e.event_type === params.event_type);
    }

    events.sort((a, b) => new Date(b.display_time).getTime() - new Date(a.display_time).getTime());

    const by_source: Record<string, number> = {
      jin10: 0,
      calendar: 0,
      announcement: 0,
      earnings: 0,
      industry: 0,
    };
    for (const e of events) {
      by_source[e.source] = (by_source[e.source] ?? 0) + 1;
    }

    return {
      events: events.slice(0, params?.limit ?? 50),
      total: events.length,
      by_source,
    };
  },
};

const MOCK_SIM_ACCOUNT: SimAccount = {
  id: "acc-e2e-1",
  name: "E2E 模拟账户",
  currency: "CNY",
  initial_balance: 1_000_000,
  cash_balance: 923_400,
  equity: 987_600,
  margin_used: 64_000,
  realized_pnl: 2_400,
  unrealized_pnl: -840,
  status: "active",
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

const MOCK_SIM_ACCOUNTS: SimAccount[] = [MOCK_SIM_ACCOUNT];

const MOCK_SIM_CONTRACT_RULES: SimContractRule[] = [
  {
    symbol: "RB0",
    name: "螺纹钢",
    exchange: "SHFE",
    contract_multiplier: 10,
    price_tick: 1,
    margin_rate_long: 0.1,
    margin_rate_short: 0.1,
    commission_mode: "per_hand",
    commission_open: 3,
    commission_close: 3,
    commission_close_today: 0,
    min_order_qty: 1,
    lot_size: 1,
    max_order_qty: 100,
    daily_price_limit_up: 0,
    daily_price_limit_down: 0,
    default_slippage_ticks: 0,
    is_custom: false,
    updated_at: new Date().toISOString(),
  },
];

const MOCK_SIM_RISK_RULES: SimRiskRule[] = [
  {
    id: "risk-e2e-1",
    account_id: "acc-e2e-1",
    scope: "account",
    symbol: null,
    rule_type: "risk_ratio",
    threshold: 0.9,
    action: "block_open",
    enabled: true,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

const MOCK_SIM_ORDERS: SimOrder[] = [
  {
    id: "order-e2e-1",
    account_id: "acc-e2e-1",
    symbol: "RB0",
    name: "螺纹钢",
    side: "buy",
    offset: "open",
    order_type: "limit",
    price: 3200,
    trigger_price: null,
    stop_loss_price: null,
    take_profit_price: null,
    oco_group_id: null,
    parent_order_id: null,
    tif: null,
    condition_operator: null,
    trailing_distance_ticks: null,
    quantity: 2,
    filled_quantity: 2,
    status: "filled",
    reason: null,
    source: "manual",
    created_at: new Date(Date.now() - 3600000).toISOString(),
    updated_at: new Date(Date.now() - 3500000).toISOString(),
  },
  {
    id: "order-e2e-2",
    account_id: "acc-e2e-1",
    symbol: "RB0",
    name: "螺纹钢",
    side: "sell",
    offset: "close",
    order_type: "limit",
    price: 3250,
    trigger_price: null,
    stop_loss_price: null,
    take_profit_price: null,
    oco_group_id: null,
    parent_order_id: null,
    tif: null,
    condition_operator: null,
    trailing_distance_ticks: null,
    quantity: 1,
    filled_quantity: 0,
    status: "open",
    reason: null,
    source: "manual",
    created_at: new Date(Date.now() - 600000).toISOString(),
    updated_at: new Date(Date.now() - 600000).toISOString(),
  },
];

const MOCK_SIM_TRADES: SimTrade[] = [
  {
    id: "trade-e2e-1",
    order_id: "order-e2e-1",
    account_id: "acc-e2e-1",
    symbol: "RB0",
    name: "螺纹钢",
    side: "buy",
    offset: "open",
    price: 3200,
    quantity: 2,
    commission: 6,
    slippage: 2,
    realized_pnl: 0,
    traded_at: new Date(Date.now() - 3500000).toISOString(),
  },
];

const MOCK_SIM_POSITIONS: SimPosition[] = [
  {
    account_id: "acc-e2e-1",
    symbol: "RB0",
    name: "螺纹钢",
    position_side: "long",
    today_qty: 2,
    history_qty: 0,
    total_qty: 2,
    avg_price: 3200,
    margin: 6400,
    unrealized_pnl: -840,
    updated_at: new Date().toISOString(),
  },
];

const MOCK_EQUITY_CURVE: SimEquitySnapshot[] = Array.from({ length: 7 }, (_, i) => {
  const t = new Date(Date.now() - (6 - i) * 86400000).toISOString();
  return {
    account_id: "acc-e2e-1",
    snapshot_at: t,
    equity: 980000 + i * 1200,
    cash_balance: 920000 + i * 600,
    margin_used: 60000 + i * 800,
    realized_pnl: 1000 + i * 200,
    unrealized_pnl: -500 + i * 50,
    risk_ratio: 0.1 + i * 0.002,
  };
});

const MOCK_PERFORMANCE = {
  account_id: "acc-e2e-1",
  total_return: 12000,
  total_return_pct: 0.012,
  total_pnl: 12000,
  max_drawdown: 3000,
  max_drawdown_pct: 0.003,
  win_rate: 0.55,
  profit_loss_ratio: 1.5,
  avg_win: 800,
  avg_loss: 533.33,
  total_trades: 20,
  winning_trades: 11,
  losing_trades: 9,
  risk_return_ratio: 2.0,
  symbol_contribution: { RB0: 8000, AU0: 4000 },
  hourly_contribution: { "09:00": 5000, "10:30": 7000 },
  avg_holding_hours: 4.5,
  overnight_count: 1,
};

const MOCK_STOCK_PAPER_ACCOUNT: StockPaperAccount = {
  id: "stock-paper-acc-1",
  name: "A股模拟组合",
  initial_balance: 1000000,
  cash_balance: 923400,
  market_value: 987600,
  total_equity: 987600,
  total_cost: 64000,
  realized_pnl: 0,
  unrealized_pnl: -840,
  status: "active",
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

const MOCK_STOCK_PAPER_PORTFOLIO: StockPaperPortfolioView = {
  account: MOCK_STOCK_PAPER_ACCOUNT,
  positions: [
    {
      account_id: "stock-paper-acc-1",
      ts_code: "600000.SH",
      name: "浦发银行",
      quantity: 100,
      available_quantity: 100,
      avg_cost: 10.0,
      total_cost: 1000,
      market_value: 1020,
      unrealized_pnl: 20,
      updated_at: new Date().toISOString(),
    },
  ],
  orders: [
    {
      id: "stock-paper-order-1",
      account_id: "stock-paper-acc-1",
      ts_code: "600000.SH",
      name: "浦发银行",
      side: "buy",
      order_type: "limit",
      price: 10.0,
      quantity: 100,
      filled_quantity: 100,
      status: "filled",
      reason: null,
      created_at: new Date(Date.now() - 86400000).toISOString(),
      updated_at: new Date().toISOString(),
    },
  ],
  trades: [
    {
      id: "stock-paper-trade-1",
      order_id: "stock-paper-order-1",
      account_id: "stock-paper-acc-1",
      ts_code: "600000.SH",
      name: "浦发银行",
      side: "buy",
      price: 10.0,
      quantity: 100,
      commission: 5,
      traded_at: new Date(Date.now() - 86400000).toISOString(),
    },
  ],
};

const MOCK_JOURNAL_ENTRIES: SimJournalEntry[] = [
  {
    id: "journal-e2e-1",
    account_id: "acc-e2e-1",
    symbol: "RB0",
    trade_id: "trade-e2e-1",
    report_id: null,
    title: "螺纹钢多头试仓",
    thesis: "需求偏弱但去库延续，轻仓试多。",
    execution_review: "按计划入场，止损设在前低。",
    emotion_tags: "谨慎,耐心",
    score: 7,
    created_at: new Date(Date.now() - 3400000).toISOString(),
    updated_at: new Date(Date.now() - 3400000).toISOString(),
  },
];
