/** Playwright E2E 用的 API Mock，模拟 Rust Command 响应。 */
import { STATIC_FUTURES_CATALOG } from "@/data/futures";
import type { AnalysisReport, AppSettings, Contract, DimensionFact, DimensionView, FollowupMessage, Interval, KLine } from "@/types";

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
  news_ids: [],
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

const MOCK_CALENDAR = [
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
      jinshi: { enabled: true, connected: true },
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
    llm_providers: ["doubao"],
    daily_analysis_cron: "0 17",
    realtime_analysis_interval: 300,
    scheduler_daily_running: true,
    scheduler_realtime_running: true,
    database_path: "data/quant.db",
    news_classify_enabled: true,
    news_classify_batch: 10,
  }),

  marketSubscribe: async () => ({ subscribed: ["rb2510"] }),

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
