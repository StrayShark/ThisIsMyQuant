/**
 * K 线、热力图等行情可视化的固定涨跌色（绿涨红跌）。
 * 不受 html[data-theme] 与 data-quote-scheme 影响；UI 文案/徽章仍用 --color-up/down。
 */
export const MARKET_CSS = {
  up: "var(--market-up)",
  down: "var(--market-down)",
  upBg: "var(--market-up-bg)",
  downBg: "var(--market-down-bg)",
  neutral: "var(--heat-neutral)",
} as const;

export function marketUpDown(changePct: number | null): { fg: string; bg: string } {
  if (changePct === null) {
    return { fg: "var(--color-muted)", bg: MARKET_CSS.neutral };
  }
  return changePct >= 0
    ? { fg: MARKET_CSS.up, bg: MARKET_CSS.upBg }
    : { fg: MARKET_CSS.down, bg: MARKET_CSS.downBg };
}

/** 强制 K 线配置使用固定涨跌色（忽略 localStorage 中的自定义色）。 */
export function withMarketCandleColors<T extends { upColor: string; downColor: string }>(cfg: T): T {
  return {
    ...cfg,
    upColor: getComputedStyle(document.documentElement).getPropertyValue("--market-up").trim(),
    downColor: getComputedStyle(document.documentElement).getPropertyValue("--market-down").trim(),
  };
}
