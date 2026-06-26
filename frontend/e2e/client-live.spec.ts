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
      body: JSON.stringify({ symbol: "rb0" }),
    });
    expect(res.ok).toBeTruthy();

    const report = (await res.json()) as E2eSuiteReport;
    console.log("[e2e-client] suite report:", JSON.stringify(report, null, 2));

    expect(report.symbol).toBe("rb0");
    expect(report.modules.length).toBeGreaterThanOrEqual(8);

    const required = [
      "llm",
      "akshare_klines",
      "dimensions",
      "sectors",
      "analysis_context",
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
