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
  NewsItemView,
} from "@/types";
import { e2eMockApi } from "@/api/e2e-mock";

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

  getSettings: async () => unwrap(await invoke<ApiResponse<AppSettings>>("get_settings")),

  marketSubscribe: async (symbols: string[]) =>
    unwrap(
      await invoke<ApiResponse<{ subscribed: string[] }>>("market_subscribe", { symbols })
    ),

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
};

export const api = E2E_MOCK ? e2eMockApi : liveApi;
