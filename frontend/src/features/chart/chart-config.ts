/** K 线用户配置（TradingView 风格默认值 + localStorage 持久化）。 */

import { withMarketCandleColors } from "@/lib/market-colors";

export type CrosshairModeSetting = "magnet" | "normal";
export type PriceScaleModeSetting = "normal" | "logarithmic";

export interface ChartUserConfig {
  /** 主图 / 成交量 pane 高度比（TradingView 约 3:1） */
  mainPaneStretch: number;
  volumePaneStretch: number;
  showVolume: boolean;

  background: string;
  textColor: string;
  gridVertVisible: boolean;
  gridHorzVisible: boolean;
  gridColor: string;
  borderColor: string;

  upColor: string;
  downColor: string;
  wickVisible: boolean;

  crosshairMode: CrosshairModeSetting;
  crosshairVertVisible: boolean;
  crosshairHorzVisible: boolean;

  barSpacing: number;
  rightOffset: number;
  fixRightEdge: boolean;
  timeVisible: boolean;
  secondsVisible: boolean;

  priceScaleMode: PriceScaleModeSetting;
  invertScale: boolean;

  lastValueVisible: boolean;
  priceLineVisible: boolean;

  scrollEnabled: boolean;
  scaleEnabled: boolean;
  kineticScroll: boolean;
}

const STORAGE_KEY = "thisismyquant-chart-config";

/** TradingView 深色默认 + 固定 market 涨跌色 */
export function defaultChartConfigFromTheme(theme: {
  background: string;
  textColor: string;
  gridColor: string;
  borderColor: string;
  upColor: string;
  downColor: string;
}): ChartUserConfig {
  return withMarketCandleColors({
    mainPaneStretch: 3,
    volumePaneStretch: 1,
    showVolume: true,

    background: theme.background,
    textColor: theme.textColor,
    gridVertVisible: true,
    gridHorzVisible: true,
    gridColor: theme.gridColor,
    borderColor: theme.borderColor,

    upColor: theme.upColor,
    downColor: theme.downColor,
    wickVisible: true,

    crosshairMode: "magnet",
    crosshairVertVisible: true,
    crosshairHorzVisible: true,

    barSpacing: 6,
    rightOffset: 12,
    fixRightEdge: false,
    timeVisible: true,
    secondsVisible: false,

    priceScaleMode: "normal",
    invertScale: false,

    lastValueVisible: true,
    priceLineVisible: true,

    scrollEnabled: true,
    scaleEnabled: true,
    kineticScroll: true,
  });
}

export function loadChartConfig(fallback: ChartUserConfig): ChartUserConfig {
  if (typeof localStorage === "undefined") return withMarketCandleColors(fallback);
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return withMarketCandleColors(fallback);
    return withMarketCandleColors({ ...fallback, ...JSON.parse(raw) });
  } catch {
    return withMarketCandleColors(fallback);
  }
}

export function saveChartConfig(config: ChartUserConfig): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(withMarketCandleColors(config)));
  } catch {
    /* ignore quota */
  }
}
