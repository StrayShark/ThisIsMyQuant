import {
  CrosshairMode,
  PriceScaleMode,
  ColorType,
  type ChartOptions,
  type DeepPartial,
} from "lightweight-charts";
import type { ChartUserConfig } from "./chart-config";

export function buildChartOptions(config: ChartUserConfig): DeepPartial<ChartOptions> {
  return {
    autoSize: true,
    layout: {
      background: { type: ColorType.Solid, color: config.background },
      textColor: config.textColor,
      fontFamily: "Geist Mono Variable, ui-monospace, monospace",
      fontSize: 11,
      panes: {
        enableResize: true,
        separatorColor: config.borderColor,
        separatorHoverColor: "var(--chart-separator-hover)",
      },
    },
    grid: {
      vertLines: {
        visible: config.gridVertVisible,
        color: config.gridColor,
      },
      horzLines: {
        visible: config.gridHorzVisible,
        color: config.gridColor,
      },
    },
    rightPriceScale: {
      borderColor: config.borderColor,
      scaleMargins: { top: 0.08, bottom: 0.02 },
      mode:
        config.priceScaleMode === "logarithmic"
          ? PriceScaleMode.Logarithmic
          : PriceScaleMode.Normal,
      invertScale: config.invertScale,
    },
    leftPriceScale: { visible: false },
    timeScale: {
      borderColor: config.borderColor,
      timeVisible: config.timeVisible,
      secondsVisible: config.secondsVisible,
      barSpacing: config.barSpacing,
      rightOffset: config.rightOffset,
      fixRightEdge: config.fixRightEdge,
      fixLeftEdge: false,
    },
    crosshair: {
      mode: config.crosshairMode === "magnet" ? CrosshairMode.Magnet : CrosshairMode.Normal,
      vertLine: {
        visible: config.crosshairVertVisible,
        labelVisible: true,
        color: "var(--chart-crosshair)",
        labelBackgroundColor: "var(--chart-crosshair-label-bg)",
      },
      horzLine: {
        visible: config.crosshairHorzVisible,
        labelVisible: true,
        color: "var(--chart-crosshair)",
        labelBackgroundColor: "var(--chart-crosshair-label-bg)",
      },
    },
    handleScroll: config.scrollEnabled
      ? {
          mouseWheel: true,
          pressedMouseMove: true,
          horzTouchDrag: true,
          vertTouchDrag: false,
        }
      : false,
    handleScale: config.scaleEnabled
      ? {
          mouseWheel: true,
          pinch: true,
          axisPressedMouseMove: { time: true, price: true },
        }
      : false,
    kineticScroll: config.kineticScroll
      ? { touch: true, mouse: true }
      : { touch: false, mouse: false },
    localization: {
      locale: "zh-CN",
      dateFormat: "yyyy-MM-dd",
    },
  };
}

export function buildCandleOptions(config: ChartUserConfig) {
  return {
    upColor: config.upColor,
    downColor: config.downColor,
    borderUpColor: config.upColor,
    borderDownColor: config.downColor,
    wickUpColor: config.wickVisible ? config.upColor : "transparent",
    wickDownColor: config.wickVisible ? config.downColor : "transparent",
    lastValueVisible: config.lastValueVisible,
    priceLineVisible: config.priceLineVisible,
    priceLineWidth: 1 as const,
    priceLineColor: config.upColor,
  };
}

export function buildVolumeOptions(_config: ChartUserConfig) {
  return {
    priceFormat: { type: "volume" as const },
    lastValueVisible: false,
    priceLineVisible: false,
  };
}

export function volumeBarColor(close: number, open: number, config: ChartUserConfig): string {
  const base = close >= open ? config.upColor : config.downColor;
  return `${base}99`;
}
