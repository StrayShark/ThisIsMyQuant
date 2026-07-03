/** 从 CSS 变量读取 K 线主题色；涨跌色固定为 --market-*，不随主题变化。 */

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

export interface ChartTheme {
  background: string;
  textColor: string;
  fontFamily: string;
  gridColor: string;
  borderColor: string;
  upColor: string;
  downColor: string;
}

export function getChartTheme(): ChartTheme {
  return {
    background: cssVar("--color-canvas-soft"),
    textColor: cssVar("--color-muted"),
    fontFamily: cssVar("--font-mono"),
    gridColor: cssVar("--color-hairline-soft"),
    borderColor: cssVar("--color-hairline"),
    upColor: cssVar("--market-up"),
    downColor: cssVar("--market-down"),
  };
}

export function volumeColor(close: number, open: number, theme: ChartTheme): string {
  return close >= open ? theme.upColor : theme.downColor;
}
