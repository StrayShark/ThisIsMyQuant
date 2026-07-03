import type { AnalysisReport } from "@/types";

const PREFERRED_DIMENSIONS = [
  "technical",
  "flow",
  "demand",
  "inventory",
  "macro",
  "domestic_supply",
  "overseas_finance",
  "overseas_supply",
] as const;

const THINK_OPEN = new RegExp("<" + "think" + ">", "i");
const THINK_CLOSE = new RegExp("</" + "think" + ">", "i");

/** 移除 DeepSeek 等模型的 think 推理块（含未闭合尾部）。 */
export function stripThinkBlocks(text: string, collapseWhitespace = false): string {
  let s = text;
  let guard = 0;
  while (guard++ < 32) {
    const openMatch = THINK_OPEN.exec(s);
    if (!openMatch || openMatch.index < 0) break;
    const open = openMatch.index;
    const afterOpen = open + openMatch[0].length;
    const tail = s.slice(afterOpen);
    const closeMatch = THINK_CLOSE.exec(tail);
    if (!closeMatch || closeMatch.index < 0) {
      s = s.slice(0, open).trim();
      break;
    }
    const close = afterOpen + closeMatch.index + closeMatch[0].length;
    s = `${s.slice(0, open).trim()}\n\n${s.slice(close).trim()}`.trim();
  }
  if (collapseWhitespace) {
    return s.replace(/\s+/g, " ").trim();
  }
  return s.trim();
}

function firstDimensionPoint(summary: Record<string, string[]> | null | undefined): string {
  if (!summary || typeof summary !== "object") return "";
  for (const code of PREFERRED_DIMENSIONS) {
    const pts = summary[code];
    if (Array.isArray(pts)) {
      for (const p of pts) {
        const cleaned = stripThinkBlocks(String(p), true);
        if (cleaned.length >= 4) return cleaned;
      }
    }
  }
  for (const pts of Object.values(summary)) {
    if (!Array.isArray(pts)) continue;
    for (const p of pts) {
      const cleaned = stripThinkBlocks(String(p), true);
      if (cleaned.length >= 4) return cleaned;
    }
  }
  return "";
}

/** 用于列表/卡片展示的报告摘要（优先 dimension_summary，过滤推理噪声）。 */
export function reportDisplaySnippet(report: AnalysisReport, maxLen = 120): string {
  const fromDims = firstDimensionPoint(report.dimension_summary ?? undefined);
  if (fromDims) return fromDims.length <= maxLen ? fromDims : `${fromDims.slice(0, maxLen)}…`;

  const fromContent = stripThinkBlocks(report.content, true);
  if (!fromContent || /^[`<]|^let me analyze/i.test(fromContent)) return "";
  return fromContent.length <= maxLen ? fromContent : `${fromContent.slice(0, maxLen)}…`;
}

/** 报告是否已有可展示的最终结论（非纯推理中间态）。 */
export function isReportDisplayReady(report: AnalysisReport): boolean {
  return reportDisplaySnippet(report, 200).length >= 6;
}

/** 报告详情页正文：去掉推理块后的 Markdown。 */
export function reportDisplayContent(content: string): string {
  const cleaned = stripThinkBlocks(content, false);
  return cleaned || "（报告正文为空或仅含模型推理过程，请重新生成分析）";
}
