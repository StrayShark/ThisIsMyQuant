import { test, expect } from "@playwright/test";

const E2E_HTTP = "http://127.0.0.1:17845";

interface E2eModuleResult {
  module: string;
  ok: boolean;
  message: string;
  duration_ms: number;
}

interface E2eSuiteReport {
  ok: boolean;
  symbol: string;
  symbol_checks: Array<{
    symbol: string;
    sector: string;
    bars: number;
    context_bars: number;
    ok: boolean;
    message: string;
  }>;
  modules: E2eModuleResult[];
  analyses: Array<{
    trigger: string;
    report_id: string;
    symbol: string;
    content_len: number;
    has_disclaimer: boolean;
  }>;
}

test.describe("客户端 Live E2E", () => {
  test("Tauri 客户端 HTTP 健康检查", async () => {
    const res = await fetch(`${E2E_HTTP}/health`);
    expect(res.ok).toBeTruthy();
    const body = (await res.json()) as { status: string };
    expect(body.status).toBe("ok");
  });

  test("各业务模块 + LLM 明日/短期分析", async () => {
    test.setTimeout(600_000);

    const res = await fetch(`${E2E_HTTP}/e2e/run`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ symbol: "rb0", symbols: ["rb0", "au0", "m0", "sc0", "ec0"] }),
    });
    expect(res.ok).toBeTruthy();

    const report = (await res.json()) as E2eSuiteReport;
    console.log("[e2e-client] suite report:", JSON.stringify(report, null, 2));

    expect(report.symbol).toBe("rb0");
    expect(report.symbol_checks).toHaveLength(5);
    expect(report.symbol_checks.map((s) => s.sector)).toEqual([
      "黑色建材",
      "有色贵金属",
      "农产品软商品",
      "能源化工",
      "航运运价",
    ]);
    for (const check of report.symbol_checks) {
      expect(check.ok, `${check.symbol}: ${check.message}`).toBeTruthy();
      expect(check.bars, `${check.symbol} bars`).toBeGreaterThan(0);
      expect(check.context_bars, `${check.symbol} context`).toBeGreaterThan(0);
    }
    expect(report.modules.length).toBeGreaterThanOrEqual(8);

    const required = [
      "llm",
      "akshare_klines",
      "dimensions",
      "sectors",
      "analysis_context",
      "fundamentals",
      "overseas_symbols",
      "professional_dashboard",
      "reports_db",
      "analysis_tomorrow",
      "analysis_short_term",
    ];
    for (const name of required) {
      const mod = report.modules.find((m) => m.module === name);
      expect(mod, `missing module ${name}`).toBeTruthy();
      expect(mod!.ok, `${name}: ${mod!.message}`).toBeTruthy();
    }

    expect(report.analyses).toHaveLength(2);
    const triggers = report.analyses.map((a) => a.trigger).sort();
    expect(triggers).toEqual(["short_term", "tomorrow"]);
    for (const a of report.analyses) {
      expect(a.content_len).toBeGreaterThan(80);
      expect(a.has_disclaimer).toBeTruthy();
    }
    expect(report.ok).toBeTruthy();
  });
});
