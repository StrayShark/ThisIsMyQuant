import { MARKET_CSS, marketUpDown } from "@/lib/market-colors";

/** Finviz 风格涨跌色：固定绿涨红跌，不受主题与外观 quote 设置影响。 */
export function treemapHeatStyle(changePct: number | null): {
  backgroundColor: string;
  color: string;
} {
  if (changePct === null) {
    return {
      backgroundColor: MARKET_CSS.neutral,
      color: "var(--color-muted)",
    };
  }

  const { fg, bg } = marketUpDown(changePct);
  const cap = 4;
  const t = Math.min(Math.abs(changePct) / cap, 1);
  const bgMix = `${Math.round(55 + t * 45)}%`;

  return {
    backgroundColor: `color-mix(in srgb, ${bg} ${bgMix}, ${MARKET_CSS.neutral})`,
    color: fg,
  };
}


export function productWeight(
  klines: import("@/types").KLine[] | undefined,
  fallback = 1
): number {
  if (!klines?.length) return fallback;
  const last = klines[klines.length - 1];
  const turnover = last.turnover > 0 ? last.turnover : last.volume * last.close;
  if (Number.isFinite(turnover) && turnover > 0) return turnover;
  return fallback;
}
