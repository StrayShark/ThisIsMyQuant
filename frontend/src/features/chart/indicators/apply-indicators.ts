import {
  HistogramSeries,
  LineSeries,
  type IChartApi,
  type ISeriesApi,
} from "lightweight-charts";
import type { KLine } from "@/types";
import type { ChartUserConfig } from "../chart-config";
import { bollinger, extractCloses, macd, rsi, sma } from "./calc";
import type { IndicatorToggles } from "./types";

/** pane 索引：0=K线 1=成交量 2=MACD 3=RSI */
export const PANE = { MAIN: 0, VOLUME: 1, MACD: 2, RSI: 3 } as const;

export interface IndicatorSeriesBundle {
  overlay: Partial<Record<"ma5" | "ma20" | "ma60" | "bollUpper" | "bollMid" | "bollLower", ISeriesApi<"Line">>>;
  macd: {
    dif: ISeriesApi<"Line"> | null;
    dea: ISeriesApi<"Line"> | null;
    hist: ISeriesApi<"Histogram"> | null;
  };
  rsi: ISeriesApi<"Line"> | null;
}

export function createIndicatorSeries(_chart: IChartApi): IndicatorSeriesBundle {
  return {
    overlay: {},
    macd: { dif: null, dea: null, hist: null },
    rsi: null,
  };
}

const OVERLAY_STYLE = {
  priceLineVisible: false,
  lastValueVisible: false,
  crosshairMarkerVisible: true,
};

export function applyIndicatorPaneLayout(
  chart: IChartApi,
  chartConfig: ChartUserConfig,
  toggles: IndicatorToggles
) {
  const panes = chart.panes();
  panes[PANE.MAIN]?.setStretchFactor(chartConfig.mainPaneStretch);
  panes[PANE.VOLUME]?.setStretchFactor(
    chartConfig.showVolume ? chartConfig.volumePaneStretch : 0
  );
  panes[PANE.MACD]?.setStretchFactor(toggles.macd ? 1.1 : 0);
  panes[PANE.RSI]?.setStretchFactor(toggles.rsi ? 0.9 : 0);
}

function ensureOverlayLine(
  chart: IChartApi,
  bundle: IndicatorSeriesBundle,
  key: keyof IndicatorSeriesBundle["overlay"],
  color: string
): ISeriesApi<"Line"> {
  if (bundle.overlay[key]) return bundle.overlay[key]!;
  const s = chart.addSeries(
    LineSeries,
    { ...OVERLAY_STYLE, color, lineWidth: key.startsWith("boll") ? 1 : 2 },
    PANE.MAIN
  );
  bundle.overlay[key] = s;
  return s;
}

function ensureMacdSeries(chart: IChartApi, bundle: IndicatorSeriesBundle) {
  if (!bundle.macd.dif) {
    bundle.macd.dif = chart.addSeries(
      LineSeries,
      { ...OVERLAY_STYLE, color: "#2962FF", lineWidth: 1 },
      PANE.MACD
    );
    bundle.macd.dea = chart.addSeries(
      LineSeries,
      { ...OVERLAY_STYLE, color: "#FF6D00", lineWidth: 1 },
      PANE.MACD
    );
    bundle.macd.hist = chart.addSeries(
      HistogramSeries,
      { priceFormat: { type: "volume" }, ...OVERLAY_STYLE },
      PANE.MACD
    );
  }
}

function ensureRsiSeries(chart: IChartApi, bundle: IndicatorSeriesBundle) {
  if (!bundle.rsi) {
    bundle.rsi = chart.addSeries(
      LineSeries,
      { ...OVERLAY_STYLE, color: "#7E57C2", lineWidth: 2 },
      PANE.RSI
    );
  }
}

export function syncIndicatorData(
  chart: IChartApi,
  bundle: IndicatorSeriesBundle,
  klines: KLine[],
  toggles: IndicatorToggles,
  chartConfig: ChartUserConfig
) {
  applyIndicatorPaneLayout(chart, chartConfig, toggles);
  const { times, closes } = extractCloses(klines);

  const setOverlay = (
    key: keyof IndicatorSeriesBundle["overlay"],
    enabled: boolean,
    color: string,
    data: { time: import("lightweight-charts").Time; value: number }[]
  ) => {
    const s = bundle.overlay[key];
    if (!enabled) {
      s?.setData([]);
      return;
    }
    ensureOverlayLine(chart, bundle, key, color).setData(data);
  };

  setOverlay("ma5", toggles.ma5, "#0070f3", sma(closes, 5, times));
  setOverlay("ma20", toggles.ma20, "#f5a623", sma(closes, 20, times));
  setOverlay("ma60", toggles.ma60, "#7928ca", sma(closes, 60, times));

  if (toggles.boll) {
    const bb = bollinger(closes, times);
    setOverlay("bollUpper", true, "#666666", bb.upper);
    setOverlay("bollMid", true, "#888888", bb.middle);
    setOverlay("bollLower", true, "#666666", bb.lower);
  } else {
    setOverlay("bollUpper", false, "#666666", []);
    setOverlay("bollMid", false, "#888888", []);
    setOverlay("bollLower", false, "#666666", []);
  }

  if (toggles.macd && klines.length > 0) {
    ensureMacdSeries(chart, bundle);
    const m = macd(closes, times, chartConfig.upColor, chartConfig.downColor);
    bundle.macd.dif?.setData(m.dif);
    bundle.macd.dea?.setData(m.dea);
    bundle.macd.hist?.setData(m.hist);
  } else {
    bundle.macd.dif?.setData([]);
    bundle.macd.dea?.setData([]);
    bundle.macd.hist?.setData([]);
  }

  if (toggles.rsi && klines.length > 0) {
    ensureRsiSeries(chart, bundle);
    bundle.rsi?.setData(rsi(closes, times));
  } else {
    bundle.rsi?.setData([]);
  }
}
