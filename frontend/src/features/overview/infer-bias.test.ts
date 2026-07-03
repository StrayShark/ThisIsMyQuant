import { describe, expect, it } from "vitest";
import type { AnalysisReport } from "@/types";
import { biasLabel, biasVariant, inferReportBias, type MarketBias } from "./infer-bias";

function report(overrides: Partial<AnalysisReport> = {}): AnalysisReport {
  return {
    id: "r1",
    symbol: "rb2505",
    trigger: "manual",
    provider: "doubao",
    prompt_version: "v1",
    context_summary: "横盘整理",
    content: "震荡",
    created_at: "2026-01-01T00:00:00Z",
    tags: [],
    ...overrides,
  };
}

describe("inferReportBias", () => {
  it("infers long when content contains bullish keywords", () => {
    const r = report({ content: "多头走强，突破压力位" });
    expect(inferReportBias(r)).toBe("long");
  });

  it("infers short when content contains bearish keywords", () => {
    const r = report({ content: "空头施压，跌破支撑位" });
    expect(inferReportBias(r)).toBe("short");
  });

  it("weights technical dimension more heavily", () => {
    const r = report({
      content: "中性",
      dimension_summary: { technical: ["均线多头排列", "放量突破"] },
    });
    expect(inferReportBias(r)).toBe("long");
  });

  it("falls back to neutral when scores cancel out", () => {
    const r = report({
      content: "上涨动能与回落风险并存",
      dimension_summary: { technical: ["偏多"], fundamental: ["偏空"] },
    });
    expect(inferReportBias(r)).toBe("neutral");
  });

  it("ignores <think> blocks when scoring", () => {
    const r = report({
      content: "<think>偏空</think> 多头走强，突破压力位",
    });
    expect(inferReportBias(r)).toBe("long");
  });
});

describe("biasLabel", () => {
  it.each<[MarketBias, string]>([
    ["long", "偏多"],
    ["short", "偏空"],
    ["neutral", "震荡"],
  ])("maps %s to %s", (bias, label) => {
    expect(biasLabel(bias)).toBe(label);
  });
});

describe("biasVariant", () => {
  it.each<[MarketBias, "up" | "down" | "secondary"]>([
    ["long", "up"],
    ["short", "down"],
    ["neutral", "secondary"],
  ])("maps %s to %s", (bias, variant) => {
    expect(biasVariant(bias)).toBe(variant);
  });
});
