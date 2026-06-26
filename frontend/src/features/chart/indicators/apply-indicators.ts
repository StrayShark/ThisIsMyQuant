import {
  HistogramSeries,
  LineSeries,
  type IChartApi,
  type ISeriesApi,
} from "lightweight-charts";
import type { KLine } from "@/types";
import type { ChartUserConfig } from "../chart-config";
import { bollinger, emaPoints, extractCloses, extractOhlc, kdj, macd, rsi, sar, sma } from "./calc";
import type { IndicatorSettings, IndicatorToggles } from "./types";

/** pane 索引：0=K线 1=成交量 2=MACD 3=RSI 4=KDJ */
export const PANE = { MAIN: 0, VOLUME: 1, MACD: 2, RSI: 3, KDJ: 4 } as const;

export interface IndicatorSeriesBundle {
  overlay: Partial<
    Record<
      "ma5" | "ma20" | "ma60" | "ema12" | "ema26" | "bollUpper" | "bollMid" | "bollLower" | "sar",
      ISeriesApi<"Line">
    >
  >;
  macd: {
    dif: ISeriesApi<"Line"> | null;
    dea: ISeriesApi<"Line"> | null;
    hist: ISeriesApi<"Histogram"> | null;
  };
  rsi: ISeriesApi<"Line"> | null;
  kdj: {
    k: ISeriesApi<"Line"> | null;
    d: ISeriesApi<"Line"> | null;
    j: ISeriesApi<"Line"> | null;
  };
}

export function createIndicatorSeries(_chart: IChartApi): IndicatorSeriesBundle {
  return {
    overlay: {},
    macd: { dif: null, dea: null, hist: null },
    rsi: null,
    kdj: { k: null, d: null, j: null },
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
  panes[PANE.KDJ]?.setStretchFactor(toggles.kdj ? 0.9 : 0);
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

function ensureKdjSeries(chart: IChartApi, bundle: IndicatorSeriesBundle) {
  if (!bundle.kdj.k) {
    bundle.kdj.k = chart.addSeries(
      LineSeries,
      { ...OVERLAY_STYLE, color: "#2962FF", lineWidth: 1 },
      PANE.KDJ
    );
    bundle.kdj.d = chart.addSeries(
      LineSeries,
      { ...OVERLAY_STYLE, color: "#FF6D00", lineWidth: 1 },
      PANE.KDJ
    );
    bundle.kdj.j = chart.addSeries(
      LineSeries,
      { ...OVERLAY_STYLE, color: "#AB47BC", lineWidth: 1 },
      PANE.KDJ
    );
  }
}

export function syncIndicatorData(
  chart: IChartApi,
  bundle: IndicatorSeriesBundle,
  klines: KLine[],
  toggles: IndicatorToggles,
  chartConfig: ChartUserConfig,
  indicatorSettings: IndicatorSettings
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
  setOverlay("ema12", toggles.ema12, "#00bcd4", emaPoints(closes, 12, times));
  setOverlay("ema26", toggles.ema26, "#009688", emaPoints(closes, 26, times));

  if (toggles.boll) {
    const bb = bollinger(
      closes,
      times,
      indicatorSettings.bollPeriod,
      indicatorSettings.bollMult
    );
    setOverlay("bollUpper", true, "#666666", bb.upper);
    setOverlay("bollMid", true, "#888888", bb.middle);
    setOverlay("bollLower", true, "#666666", bb.lower);
  } else {
    setOverlay("bollUpper", false, "#666666", []);
    setOverlay("bollMid", false, "#888888", []);
    setOverlay("bollLower", false, "#666666", []);
  }

  if (toggles.sar && klines.length > 0) {
    const ohlc = extractOhlc(klines);
    setOverlay("sar", true, "#e91e63", sar(ohlc.highs, ohlc.lows, ohlc.times));
  } else {
    setOverlay("sar", false, "#e91e63", []);
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

  if (toggles.kdj && klines.length > 0) {
    ensureKdjSeries(chart, bundle);
    const ohlc = extractOhlc(klines);
    const kd = kdj(
      ohlc.highs,
      ohlc.lows,
      ohlc.closes,
      ohlc.times,
      indicatorSettings.kdjPeriod
    );
    bundle.kdj.k?.setData(kd.k);
    bundle.kdj.d?.setData(kd.d);
    bundle.kdj.j?.setData(kd.j);
  } else {
    bundle.kdj.k?.setData([]);
    bundle.kdj.d?.setData([]);
    bundle.kdj.j?.setData([]);
  }
}
