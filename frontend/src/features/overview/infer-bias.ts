import type { AnalysisReport } from "@/types";
import { stripThinkBlocks } from "@/features/analysis/report-text";

export type MarketBias = "long" | "short" | "neutral";

const BULL = /偏多|看多|上涨|反弹|多头|走强|利多|抬升|突破|升水|去库|供不应求/;
const BEAR = /偏空|看空|下跌|回落|空头|走弱|利空|承压|跌破|贴水|累库|过剩|偏弱/;

function scoreText(text: string): number {
  let s = 0;
  if (BULL.test(text)) s += 1;
  if (BEAR.test(text)) s -= 1;
  return s;
}

/** 从 LLM 报告维度摘要与正文推断多空倾向。 */
export function inferReportBias(report: AnalysisReport): MarketBias {
  let score = scoreText(stripThinkBlocks(report.content));

  const ds = report.dimension_summary;
  if (ds && typeof ds === "object") {
    for (const [code, points] of Object.entries(ds)) {
      const joined = Array.isArray(points)
        ? points.map((p) => stripThinkBlocks(String(p))).join(" ")
        : stripThinkBlocks(String(points));
      const weight = code === "technical" || code === "flow" ? 1.5 : 1;
      score += scoreText(joined) * weight;
    }
  }

  score += scoreText(report.context_summary) * 0.5;

  if (score >= 1) return "long";
  if (score <= -1) return "short";
  return "neutral";
}

export function biasLabel(bias: MarketBias): string {
  if (bias === "long") return "偏多";
  if (bias === "short") return "偏空";
  return "震荡";
}

export function biasVariant(bias: MarketBias): "up" | "down" | "secondary" {
  if (bias === "long") return "up";
  if (bias === "short") return "down";
  return "secondary";
}
