/** 图表常见技术指标开关（localStorage 持久化）。 */

export type OverlayIndicatorId = "ma5" | "ma20" | "ma60" | "ema12" | "ema26" | "boll" | "sar";
export type PaneIndicatorId = "macd" | "rsi" | "kdj";

export interface IndicatorToggles {
  ma5: boolean;
  ma20: boolean;
  ma60: boolean;
  ema12: boolean;
  ema26: boolean;
  boll: boolean;
  sar: boolean;
  macd: boolean;
  rsi: boolean;
  kdj: boolean;
}

export interface IndicatorSettings {
  bollPeriod: number;
  bollMult: number;
  kdjPeriod: number;
}

export const DEFAULT_INDICATOR_TOGGLES: IndicatorToggles = {
  ma5: true,
  ma20: true,
  ma60: false,
  ema12: false,
  ema26: false,
  boll: false,
  sar: false,
  macd: false,
  rsi: false,
  kdj: false,
};

export const DEFAULT_INDICATOR_SETTINGS: IndicatorSettings = {
  bollPeriod: 20,
  bollMult: 2,
  kdjPeriod: 9,
};

export const INDICATOR_STORAGE_KEY = "thisismyquant-chart-indicators";
export const INDICATOR_SETTINGS_KEY = "thisismyquant-indicator-settings";

export function loadIndicatorToggles(): IndicatorToggles {
  if (typeof localStorage === "undefined") return { ...DEFAULT_INDICATOR_TOGGLES };
  try {
    const raw = localStorage.getItem(INDICATOR_STORAGE_KEY);
    if (!raw) return { ...DEFAULT_INDICATOR_TOGGLES };
    return { ...DEFAULT_INDICATOR_TOGGLES, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_INDICATOR_TOGGLES };
  }
}

export function saveIndicatorToggles(toggles: IndicatorToggles): void {
  try {
    localStorage.setItem(INDICATOR_STORAGE_KEY, JSON.stringify(toggles));
  } catch {
    /* ignore */
  }
}

export function loadIndicatorSettings(): IndicatorSettings {
  if (typeof localStorage === "undefined") return { ...DEFAULT_INDICATOR_SETTINGS };
  try {
    const raw = localStorage.getItem(INDICATOR_SETTINGS_KEY);
    if (!raw) return { ...DEFAULT_INDICATOR_SETTINGS };
    return { ...DEFAULT_INDICATOR_SETTINGS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_INDICATOR_SETTINGS };
  }
}

export function saveIndicatorSettings(settings: IndicatorSettings): void {
  try {
    localStorage.setItem(INDICATOR_SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    /* ignore */
  }
}

export interface IndicatorMeta {
  id: keyof IndicatorToggles;
  label: string;
  kind: "overlay" | "pane";
  color?: string;
}

export const INDICATOR_REGISTRY: IndicatorMeta[] = [
  { id: "ma5", label: "MA5", kind: "overlay", color: "#0070f3" },
  { id: "ma20", label: "MA20", kind: "overlay", color: "#f5a623" },
  { id: "ma60", label: "MA60", kind: "overlay", color: "#7928ca" },
  { id: "ema12", label: "EMA12", kind: "overlay", color: "#00bcd4" },
  { id: "ema26", label: "EMA26", kind: "overlay", color: "#009688" },
  { id: "boll", label: "BOLL", kind: "overlay", color: "#888888" },
  { id: "sar", label: "SAR", kind: "overlay", color: "#e91e63" },
  { id: "macd", label: "MACD", kind: "pane" },
  { id: "rsi", label: "RSI", kind: "pane" },
  { id: "kdj", label: "KDJ", kind: "pane" },
];
