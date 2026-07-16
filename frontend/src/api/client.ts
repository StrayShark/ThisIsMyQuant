import type {
  AnalysisReport,
  ApiResponse,
  AppSettings,
  Contract,
  DimensionFact,
  DimensionView,
  FollowupMessage,
  FuturesSector,
  Interval,
  KLine,
  CalendarEvent,
  NewsItemView,
  NewsRecord,
  StockSymbol,
  AStockDashboardView,
  StockBar,
  StockDetailView,
  StockBoardView,
  StockBoardDetailView,
  StockScreenerResultView,
  StockScreenTemplate,
  StockDataSyncStatus,
  StockFinancialMetric,
  StockWatchlist,
  StockPaperAccount,
  StockPaperPortfolioView,
  StockPaperOrder,
  StockPaperOrderEstimate,
  CreateStockPaperAccountRequest,
  PlaceStockPaperOrderRequest,
  CancelStockPaperOrderRequest,
  MarketOverview,
  MarketLeaderboard,
  MarketFilters,
  MarketAssetSearchResult,
  WatchlistGroup,
  WatchlistItem,
  WatchlistSummary,
  MarketEvent,
  MarketEventQuery,
  MarketEventListResult,
  DatabaseDomainSummary,
  DataDomainActionResult,
  DataDomainCode,
  AiSummaryRequest,
  AiReportSummary,
  AiTaskListResult,
} from "@/types";
import { e2eMockApi } from "@/api/e2e-mock";
import { normalizeAppearance } from "@/lib/appearance";
import { useAppStore } from "@/app/store";

const E2E_MOCK = import.meta.env.VITE_E2E_MOCK === "true";

async function isTauri(): Promise<boolean> {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

function unwrap<T>(res: ApiResponse<T>): T {
  if (res.code !== 0 || res.data === null) {
    throw new Error(res.message || "API error");
  }
  return res.data;
}

async function unwrapCalendar(res: ApiResponse<CalendarEvent[]>): Promise<CalendarEvent[]> {
  if (res.code !== 0 || res.data === null) {
    throw new Error(res.message || "API error");
  }
  if (res.message && res.message !== "ok") {
    useAppStore.getState().showToast(res.message);
  }
  return res.data;
}

export function resetApiClient() {
  /* no-op */
}

const liveApi = {
  health: async () => {
    const data = unwrap(await invoke<ApiResponse<Record<string, unknown>>>("get_health"));
    return { data };
  },

  listContracts: async (exchange?: string) =>
    unwrap(await invoke<ApiResponse<Contract[]>>("list_contracts", { exchange: exchange ?? null })),

  listProducts: async (params?: { tier?: string }) =>
    unwrap(
      await invoke<ApiResponse<FuturesSector[]>>("list_products", {
        tier: params?.tier ?? "core",
      })
    ),

  listNews: async (params?: { symbol?: string; dimension?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<NewsItemView[]>>("list_news", {
        symbol: params?.symbol ?? null,
        dimension: params?.dimension ?? null,
        limit: params?.limit ?? null,
      })
    ),

  listNewsByIds: async (ids: string[]) =>
    unwrap(await invoke<ApiResponse<NewsItemView[]>>("list_news_by_ids", { ids })),

  listCalendarEvents: async (params?: {
    start?: string;
    end?: string;
    min_star?: number;
    country?: string;
    keyword?: string;
  }) =>
    unwrapCalendar(
      await invoke<ApiResponse<CalendarEvent[]>>("list_calendar_events", {
        start: params?.start ?? null,
        end: params?.end ?? null,
        min_star: params?.min_star ?? null,
        country: params?.country ?? null,
        keyword: params?.keyword ?? null,
      })
    ),

  listUnclassifiedNews: async (limit?: number) =>
    unwrap(
      await invoke<ApiResponse<NewsRecord[]>>("list_unclassified_news", {
        limit: limit ?? null,
      })
    ),

  listDimensions: async (symbol?: string) =>
    unwrap(
      await invoke<ApiResponse<DimensionView[]>>("list_dimensions", {
        symbol: symbol ?? null,
      })
    ),

  listDimensionFacts: async (params?: { symbol?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<DimensionFact[]>>("list_dimension_facts", {
        symbol: params?.symbol ?? null,
        limit: params?.limit ?? null,
      })
    ),

  listFollowups: async (params?: { report_id?: string; symbol?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<FollowupMessage[]>>("list_followups", {
        report_id: params?.report_id ?? null,
        symbol: params?.symbol ?? null,
        limit: params?.limit ?? null,
      })
    ),

  getKlines: async (params: {
    symbol: string;
    interval: Interval;
    start?: string;
    end?: string;
    limit?: number;
  }) =>
    unwrap(
      await invoke<ApiResponse<KLine[]>>("get_klines", {
        symbol: params.symbol,
        interval: params.interval,
        start: params.start ?? null,
        end: params.end ?? null,
        limit: params.limit ?? null,
      })
    ),

  listReports: async (params: { symbol?: string; trigger?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<AnalysisReport[]>>("list_reports", {
        symbol: params.symbol ?? null,
        trigger: params.trigger ?? null,
        limit: params.limit ?? null,
      })
    ),

  getReport: async (id: string) =>
    unwrap(await invoke<ApiResponse<AnalysisReport>>("get_report", { report_id: id })),

  marketSubscribe: async (symbols: string[]) =>
    unwrap(
      await invoke<ApiResponse<{ subscribed: string[] }>>("market_subscribe", { symbols })
    ),

  marketUnsubscribe: async (symbols: string[]) =>
    unwrap(
      await invoke<ApiResponse<{ unsubscribed: string[] }>>("market_unsubscribe", { symbols })
    ),

  getRealtimeQuotes: async (symbols?: string[]) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").RealtimeQuote[]>>("get_realtime_quotes", {
        symbols: symbols ?? null,
      })
    ),

  getRuntimeStatus: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").RuntimeStatus>>("get_runtime_status")),

  getSymbolContext: async (symbol: string) =>
    unwrap(
      await invoke<ApiResponse<Record<string, unknown>>>("get_symbol_context", { symbol })
    ),

  getSettings: async () => unwrap(await invoke<ApiResponse<AppSettings>>("get_settings")),

  getLlmSetup: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").LlmSetupStatus>>("get_llm_setup")),

  saveLlmSetup: async (payload: {
    credentials: import("@/types").LlmCredentialInput[];
    default_provider: string;
  }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").LlmSetupStatus>>("save_llm_setup", {
        payload,
      })
    ),

  getUserPreferences: async () => {
    const prefs = unwrap(
      await invoke<ApiResponse<import("@/types").UserPreferences>>("get_user_preferences")
    );
    return { ...prefs, ...normalizeAppearance(prefs) };
  },

  saveUserPreferences: async (prefs: import("@/types").UserPreferences) => {
    const saved = unwrap(
      await invoke<ApiResponse<import("@/types").UserPreferences>>("save_user_preferences", {
        prefs,
      })
    );
    return { ...saved, ...normalizeAppearance(saved) };
  },

  reloadConfig: async () => unwrap(await invoke<ApiResponse<AppSettings>>("reload_config")),

  exportKlinesCsv: async (params: { symbol: string; interval: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<string>>("export_klines_csv", {
        symbol: params.symbol,
        interval: params.interval,
        limit: params.limit ?? null,
      })
    ),

  exportReportsCsv: async (params?: { symbol?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<string>>("export_reports_csv", {
        symbol: params?.symbol ?? null,
        limit: params?.limit ?? null,
      })
    ),

  importKlinesCsv: async (params: { csv: string; symbol: string; interval: string }) =>
    unwrap(
      await invoke<ApiResponse<{ imported: number }>>("import_klines_csv", {
        csv: params.csv,
        symbol: params.symbol,
        interval: params.interval,
      })
    ),

  getProfessionalDashboard: async () =>
    unwrap(
      await invoke<ApiResponse<import("@/types").ProfessionalDashboard>>(
        "get_professional_dashboard"
      )
    ),

  reclassifyNews: async (params: { news_ids: string[]; provider?: string; use_llm?: boolean }) =>
    unwrap(
      await invoke<ApiResponse<{ labels_saved: number }>>("reclassify_news", {
        news_ids: params.news_ids,
        provider: params.provider ?? null,
        use_llm: params.use_llm ?? null,
      })
    ),

  triggerBatchAnalysis: async (params?: {
    symbols?: string[];
    trigger?: string;
    provider?: string;
  }) =>
    unwrap(
      await invoke<ApiResponse<{ started: boolean; total: number }>>("trigger_batch_analysis", {
        symbols: params?.symbols ?? null,
        trigger: params?.trigger ?? null,
        provider: params?.provider ?? null,
      })
    ),

  triggerComprehensiveAnalysis: async () =>
    unwrap(
      await invoke<ApiResponse<{ started: boolean; total: number; includes_data_fetch: boolean }>>(
        "trigger_comprehensive_analysis"
      )
    ),

  triggerDataFetch: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").DataFetchSummary>>("trigger_data_fetch")),

  getScheduleStatus: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").ScheduleStatus>>("get_schedule_status")),

  getBatchStatus: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").BatchJobStatus>>("get_batch_status")),

  getStatusDashboard: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").StatusDashboard>>("get_status_dashboard")),

  probeOllama: async () => unwrap(await invoke<ApiResponse<boolean>>("probe_ollama")),

  triggerAnalysis: async (params: { symbol: string; trigger?: string; provider?: string }) =>
    unwrap(
      await invoke<ApiResponse<{ report_id: string; symbol: string }>>("trigger_analysis", {
        symbol: params.symbol,
        trigger: params.trigger ?? null,
        provider: params.provider ?? null,
      })
    ),

  streamAnalysis: async (symbol: string, trigger = "manual"): Promise<ReadableStreamDefaultReader<Uint8Array>> => {
    if (!(await isTauri())) {
      throw new Error("stream analysis requires Tauri desktop app");
    }
    const { listen } = await import("@tauri-apps/api/event");

    let deltaUnlisten: (() => void) | undefined;
    let doneUnlisten: (() => void) | undefined;
    let errorUnlisten: (() => void) | undefined;

    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        const encoder = new TextEncoder();
        const enqueue = (obj: object) => {
          controller.enqueue(encoder.encode(`data: ${JSON.stringify(obj)}\n\n`));
        };

        listen<{ text: string }>("analysis-delta", (e) => {
          enqueue({ text: e.payload.text });
        }).then((fn) => {
          deltaUnlisten = fn;
        });

        listen<{ status: string; dimension_summary?: Record<string, string[]> }>("analysis-done", (e) => {
          enqueue({
            status: e.payload.status,
            dimension_summary: e.payload.dimension_summary,
          });
          controller.close();
          deltaUnlisten?.();
          doneUnlisten?.();
          errorUnlisten?.();
        }).then((fn) => {
          doneUnlisten = fn;
        });

        listen<string>("analysis-error", (e) => {
          controller.error(new Error(e.payload));
          deltaUnlisten?.();
          doneUnlisten?.();
          errorUnlisten?.();
        }).then((fn) => {
          errorUnlisten = fn;
        });

        invoke("stream_analysis", { symbol, trigger }).catch((err) => {
          controller.error(err);
        });
      },
    });

    return stream.getReader();
  },

  listSimAccounts: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").SimAccount[]>>("list_sim_accounts")),

  createSimAccount: async (payload: { name: string; initial_balance: number }) =>
    unwrap(await invoke<ApiResponse<import("@/types").SimAccount>>("create_sim_account", payload)),

  resetSimAccount: async (accountId: string) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimAccount>>("reset_sim_account", {
        account_id: accountId,
      })
    ),

  getSimAccountSnapshot: async (accountId?: string) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimAccountSnapshot>>("get_sim_account_snapshot", {
        account_id: accountId ?? null,
      })
    ),

  listSimPositions: async (accountId?: string) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimPosition[]>>("list_sim_positions", {
        account_id: accountId ?? null,
      })
    ),

  listSimOrders: async (params?: { account_id?: string; status?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimOrder[]>>("list_sim_orders", {
        account_id: params?.account_id ?? null,
        status: params?.status ?? null,
        limit: params?.limit ?? null,
      })
    ),

  listSimTrades: async (params?: { account_id?: string; symbol?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimTrade[]>>("list_sim_trades", {
        account_id: params?.account_id ?? null,
        symbol: params?.symbol ?? null,
        limit: params?.limit ?? null,
      })
    ),

  listSimEquityCurve: async (params?: { account_id?: string; days?: number }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimEquitySnapshot[]>>("list_sim_equity_curve", {
        account_id: params?.account_id ?? null,
        days: params?.days ?? null,
      })
    ),

  getSimPerformance: async (params?: { account_id?: string }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimPerformance>>("get_sim_performance", {
        account_id: params?.account_id ?? null,
      })
    ),

  placeSimOrder: async (payload: import("@/types").PlaceSimOrderRequest) =>
    unwrap(await invoke<ApiResponse<import("@/types").SimOrder>>("place_sim_order", payload as unknown as Record<string, unknown>)),

  cancelSimOrder: async (orderId: string) =>
    unwrap(await invoke<ApiResponse<import("@/types").SimOrder>>("cancel_sim_order", { order_id: orderId })),

  estimateSimOrder: async (payload: Omit<import("@/types").PlaceSimOrderRequest, "account_id"> & { account_id?: string }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimOrderEstimate>>("estimate_sim_order", {
        ...payload,
        account_id: payload.account_id ?? null,
      })
    ),

  listSimContractRules: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").SimContractRule[]>>("list_sim_contract_rules")),

  saveSimContractRule: async (payload: import("@/types").SimContractRule) =>
    unwrap(await invoke<ApiResponse<import("@/types").SimContractRule>>("save_sim_contract_rule", payload as unknown as Record<string, unknown>)),

  deleteSimContractRule: async (symbol: string) =>
    unwrap(await invoke<ApiResponse<string>>("delete_sim_contract_rule", { symbol })),

  listSimRiskRules: async (params?: { account_id?: string }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimRiskRule[]>>("list_sim_risk_rules", {
        account_id: params?.account_id ?? null,
      })
    ),

  saveSimRiskRule: async (payload: import("@/types").SimRiskRule) =>
    unwrap(await invoke<ApiResponse<import("@/types").SimRiskRule>>("save_sim_risk_rule", payload as unknown as Record<string, unknown>)),

  deleteSimRiskRule: async (id: string) =>
    unwrap(await invoke<ApiResponse<string>>("delete_sim_risk_rule", { id })),

  forceLiquidate: async (payload: { account_id: string; symbol?: string | null }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimOrder[]>>("force_liquidate", {
        account_id: payload.account_id,
        symbol: payload.symbol ?? null,
      })
    ),

  saveSimJournalEntry: async (payload: Partial<import("@/types").SimJournalEntry>) =>
    unwrap(await invoke<ApiResponse<import("@/types").SimJournalEntry>>("save_sim_journal_entry", payload)),

  listSimJournalEntries: async (params?: { account_id?: string; symbol?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").SimJournalEntry[]>>("list_sim_journal_entries", {
        account_id: params?.account_id ?? null,
        symbol: params?.symbol ?? null,
        limit: params?.limit ?? null,
      })
    ),

  startMarketReplay: async (payload: { symbol: string; date: string; account_id?: string; speed?: number }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").ReplayState>>("start_market_replay", {
        ...payload,
        account_id: payload.account_id ?? null,
        speed: payload.speed ?? 1,
      })
    ),

  stopMarketReplay: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").ReplayState>>("stop_market_replay")),

  stepMarketReplay: async (steps?: number) =>
    unwrap(await invoke<ApiResponse<import("@/types").ReplayState>>("step_market_replay", { steps: steps ?? 1 })),

  getReplayState: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").ReplayState>>("get_replay_state")),

  getReplayKlines: async () =>
    unwrap(
      await invoke<ApiResponse<import("@/types").ReplayKlinePayload>>("get_replay_klines")
    ),

  getDatabaseSummary: async () =>
    unwrap(await invoke<ApiResponse<import("@/types").DatabaseSummary>>("get_database_summary")),

  backupDatabase: async () =>
    unwrap(await invoke<ApiResponse<string>>("backup_database")),

  prepareDatabaseRestore: async (backupPath: string) =>
    unwrap(await invoke<ApiResponse<string>>("prepare_database_restore", { backup_path: backupPath })),

  // A 股
  listStockSymbols: async (params?: { query?: string; industry?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<StockSymbol[]>>("list_stock_symbols", {
        query: params?.query ?? null,
        industry: params?.industry ?? null,
        limit: params?.limit ?? null,
      })
    ),

  getAStockDashboard: async () =>
    unwrap(await invoke<ApiResponse<AStockDashboardView>>("get_a_stock_dashboard")),

  getStockKlines: async (params: { ts_code: string; adjustment?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<StockBar[]>>("get_stock_klines", {
        ts_code: params.ts_code,
        adjustment: params.adjustment ?? null,
        limit: params.limit ?? null,
      })
    ),

  getStockDetail: async (ts_code: string) =>
    unwrap(await invoke<ApiResponse<StockDetailView>>("get_stock_detail", { ts_code })),

  listStockIndustries: async (boardType?: string) =>
    unwrap(
      await invoke<ApiResponse<StockBoardView[]>>("list_stock_industries", {
        board_type: boardType ?? null,
      })
    ),

  getStockIndustryDetail: async (params: { board_code: string; trade_date?: string }) =>
    unwrap(
      await invoke<ApiResponse<StockBoardDetailView>>("get_stock_industry_detail", {
        board_code: params.board_code,
        trade_date: params.trade_date ?? null,
      })
    ),

  runStockScreener: async (params: { criteria_json: string; name?: string }) =>
    unwrap(
      await invoke<ApiResponse<StockScreenerResultView>>("run_stock_screener", {
        criteria_json: params.criteria_json,
        name: params.name ?? null,
      })
    ),

  saveStockScreen: async (params: { criteria_json: string; name?: string }) =>
    unwrap(
      await invoke<ApiResponse<StockScreenTemplate>>("save_stock_screen", {
        criteria_json: params.criteria_json,
        name: params.name ?? null,
      })
    ),

  listStockScreenTemplates: async () =>
    unwrap(await invoke<ApiResponse<StockScreenTemplate[]>>("list_stock_screen_templates")),

  deleteStockScreenTemplate: async (id: string) =>
    unwrap(await invoke<ApiResponse<null>>("delete_stock_screen_template", { id })),

  summarizeStockScreen: async (params: { criteria_json: string; result_summary: string }) =>
    unwrap(
      await invoke<ApiResponse<AnalysisReport>>("summarize_stock_screen", {
        criteria_json: params.criteria_json,
        result_summary: params.result_summary,
      })
    ),

  listStockFinancials: async (ts_code: string) =>
    unwrap(await invoke<ApiResponse<StockFinancialMetric[]>>("list_stock_financials", { ts_code })),

  listStockWatchlists: async () =>
    unwrap(await invoke<ApiResponse<StockWatchlist[]>>("list_stock_watchlists")),

  saveStockWatchlist: async (payload: { id?: string; name: string; symbols: string[] }) =>
    unwrap(
      await invoke<ApiResponse<StockWatchlist>>("save_stock_watchlist", {
        id: payload.id ?? null,
        name: payload.name,
        symbols: payload.symbols,
      })
    ),

  deleteStockWatchlist: async (id: string) =>
    unwrap(await invoke<ApiResponse<null>>("delete_stock_watchlist", { id })),

  triggerStockDataSync: async (params: { scope: string; symbols?: string[] }) =>
    unwrap(
      await invoke<ApiResponse<StockDataSyncStatus>>("trigger_stock_data_sync", {
        scope: params.scope,
        symbols: params.symbols ?? null,
      })
    ),

  // A 股模拟组合
  listStockPaperAccounts: async () =>
    unwrap(await invoke<ApiResponse<StockPaperAccount[]>>("list_stock_paper_accounts")),

  createStockPaperAccount: async (payload: CreateStockPaperAccountRequest) =>
    unwrap(
      await invoke<ApiResponse<StockPaperAccount>>("create_stock_paper_account", {
        name: payload.name,
        initial_balance: payload.initial_balance,
      })
    ),

  getStockPaperPortfolio: async (account_id: string) =>
    unwrap(
      await invoke<ApiResponse<StockPaperPortfolioView>>("get_stock_paper_portfolio", {
        account_id,
      })
    ),

  placeStockPaperOrder: async (payload: PlaceStockPaperOrderRequest) =>
    unwrap(
      await invoke<ApiResponse<StockPaperOrder>>("place_stock_paper_order", {
        account_id: payload.account_id,
        ts_code: payload.ts_code,
        side: payload.side,
        order_type: payload.order_type,
        price: payload.price ?? null,
        quantity: payload.quantity,
      })
    ),

  cancelStockPaperOrder: async (payload: CancelStockPaperOrderRequest) =>
    unwrap(
      await invoke<ApiResponse<StockPaperOrder>>("cancel_stock_paper_order", {
        account_id: payload.account_id,
        order_id: payload.order_id,
      })
    ),

  estimateStockPaperOrder: async (params: { price: number; quantity: number; side: string }) =>
    unwrap(
      await invoke<ApiResponse<StockPaperOrderEstimate>>("estimate_stock_paper_order", {
        price: params.price,
        quantity: params.quantity,
        side: params.side,
      })
    ),

  generateStockSummary: async (ts_code: string) =>
    unwrap(await invoke<ApiResponse<AnalysisReport>>("generate_stock_summary", { ts_code })),

  generateStockPortfolioReview: async (account_id: string) =>
    unwrap(
      await invoke<ApiResponse<AnalysisReport>>("generate_stock_portfolio_review", { account_id })
    ),

  generateTradeReview: async (payload: { account_id?: string; days?: number }) =>
    unwrap(
      await invoke<ApiResponse<import("@/types").AnalysisReport>>("generate_trade_review", {
        account_id: payload.account_id ?? null,
        days: payload.days ?? 30,
      })
    ),

  streamFollowup: async (
    reportId: string,
    question: string
  ): Promise<ReadableStreamDefaultReader<Uint8Array>> => {
    if (!(await isTauri())) {
      throw new Error("followup requires Tauri desktop app");
    }
    const { listen } = await import("@tauri-apps/api/event");

    let deltaUnlisten: (() => void) | undefined;
    let doneUnlisten: (() => void) | undefined;
    let errorUnlisten: (() => void) | undefined;

    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        const encoder = new TextEncoder();
        const enqueue = (obj: object) => {
          controller.enqueue(encoder.encode(`data: ${JSON.stringify(obj)}\n\n`));
        };

        listen<{ text: string }>("followup-delta", (e) => {
          enqueue({ text: e.payload.text });
        }).then((fn) => {
          deltaUnlisten = fn;
        });

        listen<{ status: string }>("followup-done", (e) => {
          enqueue({ status: e.payload.status });
          controller.close();
          deltaUnlisten?.();
          doneUnlisten?.();
          errorUnlisten?.();
        }).then((fn) => {
          doneUnlisten = fn;
        });

        listen<string>("followup-error", (e) => {
          controller.error(new Error(e.payload));
          deltaUnlisten?.();
          doneUnlisten?.();
          errorUnlisten?.();
        }).then((fn) => {
          errorUnlisten = fn;
        });

        invoke("analysis_followup", { report_id: reportId, question }).catch((err) => {
          controller.error(err);
        });
      },
    });

    return stream.getReader();
  },

  // CMC 重构：统一市场 API
  getMarketOverview: async () =>
    unwrap(await invoke<ApiResponse<MarketOverview>>("get_market_overview")),

  listMarketAssets: async (params: MarketFilters & { sort_by?: string; sort_desc?: boolean; limit?: number; offset?: number }) =>
    unwrap(
      await invoke<ApiResponse<MarketAssetSearchResult>>("list_market_assets", {
        market: params.market ?? null,
        sector: params.sector ?? null,
        industry: params.industry ?? null,
        quality: params.quality ?? null,
        watched: params.watched ?? null,
        min_turnover: params.min_turnover ?? null,
        query: params.query ?? null,
        sort_by: params.sort_by ?? null,
        sort_desc: params.sort_desc ?? null,
        limit: params.limit ?? null,
        offset: params.offset ?? null,
      })
    ),

  getMarketLeaderboard: async (params: { category: string; market?: string; limit?: number }) =>
    unwrap(
      await invoke<ApiResponse<MarketLeaderboard>>("get_market_leaderboard", {
        category: params.category,
        market: params.market ?? null,
        limit: params.limit ?? null,
      })
    ),

  getAssetSparkline: async (params: { symbol: string; market: string; points?: number }) =>
    unwrap(
      await invoke<ApiResponse<number[]>>("get_asset_sparkline", {
        symbol: params.symbol,
        market: params.market,
        points: params.points ?? null,
      })
    ),

  searchAssets: async (query: string, limit?: number) =>
    unwrap(
      await invoke<ApiResponse<MarketAssetSearchResult>>("search_assets", {
        query,
        limit: limit ?? null,
      })
    ),

  // 统一自选 API
  listWatchlistGroups: async () =>
    unwrap(await invoke<ApiResponse<WatchlistGroup[]>>("list_watchlist_groups")),

  createWatchlistGroup: async (payload: { name: string; sort_order?: number }) =>
    unwrap(
      await invoke<ApiResponse<WatchlistGroup>>("save_watchlist_group", {
        id: null,
        name: payload.name,
        sort_order: payload.sort_order ?? null,
      })
    ),

  updateWatchlistGroup: async (payload: { id: string; name: string; sort_order?: number }) =>
    unwrap(
      await invoke<ApiResponse<WatchlistGroup>>("save_watchlist_group", {
        id: payload.id,
        name: payload.name,
        sort_order: payload.sort_order ?? null,
      })
    ),

  deleteWatchlistGroup: async (id: string) =>
    unwrap(await invoke<ApiResponse<void>>("delete_watchlist_group", { id })),

  listWatchlistItems: async (groupId?: string) =>
    unwrap(
      await invoke<ApiResponse<WatchlistItem[]>>("list_watchlist_items", {
        group_id: groupId ?? null,
      })
    ),

  addWatchlistItem: async (payload: {
    group_id: string;
    asset_type: "futures" | "stock";
    symbol: string;
    name: string;
    notes?: string;
    alert_price?: number;
    alert_pct?: number;
  }) =>
    unwrap(
      await invoke<ApiResponse<WatchlistItem>>("save_watchlist_item", {
        id: null,
        group_id: payload.group_id,
        asset_type: payload.asset_type,
        symbol: payload.symbol,
        name: payload.name,
        notes: payload.notes ?? null,
        alert_price: payload.alert_price ?? null,
        alert_pct: payload.alert_pct ?? null,
        sort_order: null,
      })
    ),

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
  }) =>
    unwrap(
      await invoke<ApiResponse<WatchlistItem>>("save_watchlist_item", {
        id: payload.id,
        group_id: payload.group_id,
        asset_type: payload.asset_type,
        symbol: payload.symbol,
        name: payload.name,
        notes: payload.notes ?? null,
        alert_price: payload.alert_price ?? null,
        alert_pct: payload.alert_pct ?? null,
        sort_order: payload.sort_order ?? null,
      })
    ),

  removeWatchlistItem: async (id: string) =>
    unwrap(await invoke<ApiResponse<void>>("delete_watchlist_item", { id })),

  getWatchlistSummary: async () =>
    unwrap(await invoke<ApiResponse<WatchlistSummary>>("get_watchlist_summary")),

  getWatchlistEvents: async () =>
    unwrap(await invoke<ApiResponse<MarketEvent[]>>("get_watchlist_events")),

  // CMC 重构：P1 事件资讯中心
  listMarketEvents: async (params: MarketEventQuery) =>
    unwrap(
      await invoke<ApiResponse<MarketEventListResult>>("list_market_events", {
        source: params.source ?? null,
        symbol: params.symbol ?? null,
        sector: params.sector ?? null,
        importance: params.importance ?? null,
        event_type: params.event_type ?? null,
        start: params.start ?? null,
        end: params.end ?? null,
        limit: params.limit ?? null,
      })
    ),

  // CMC 重构：P1 数据库资产中心
  getDatabaseDomainSummary: async () =>
    unwrap(await invoke<ApiResponse<DatabaseDomainSummary>>("get_database_domain_summary")),

  syncDataDomain: async (domain: DataDomainCode) =>
    unwrap(
      await invoke<ApiResponse<DataDomainActionResult>>("sync_data_domain", { domain })
    ),

  exportDataDomain: async (domain: DataDomainCode) =>
    unwrap(
      await invoke<ApiResponse<DataDomainActionResult>>("export_data_domain", { domain })
    ),

  cleanupDataDomain: async (domain: DataDomainCode) =>
    unwrap(
      await invoke<ApiResponse<DataDomainActionResult>>("cleanup_data_domain", { domain })
    ),

  // CMC 重构：P1 引用式 AI
  generateAiSummary: async (payload: AiSummaryRequest) =>
    unwrap(
      await invoke<ApiResponse<AiReportSummary>>("generate_ai_summary", {
        task_type: payload.task_type,
        target_symbol: payload.target_symbol ?? null,
        target_assets: payload.target_assets ?? null,
        prompt: payload.prompt ?? null,
        provider: payload.provider ?? null,
      })
    ),

  listAiTasks: async () =>
    unwrap(await invoke<ApiResponse<AiTaskListResult>>("list_ai_tasks")),
};

export const api = E2E_MOCK ? e2eMockApi : liveApi;
