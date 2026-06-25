/** 图表常见技术指标开关（localStorage 持久化）。 */

export type OverlayIndicatorId = "ma5" | "ma20" | "ma60" | "boll";
export type PaneIndicatorId = "macd" | "rsi";

export interface IndicatorToggles {
  ma5: boolean;
  ma20: boolean;
  ma60: boolean;
  boll: boolean;
  macd: boolean;
  rsi: boolean;
}

export const DEFAULT_INDICATOR_TOGGLES: IndicatorToggles = {
  ma5: true,
  ma20: true,
  ma60: false,
  boll: false,
  macd: false,
  rsi: false,
};

export const INDICATOR_STORAGE_KEY = "thisismyquant-chart-indicators";

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
  { id: "boll", label: "BOLL", kind: "overlay", color: "#888888" },
  { id: "macd", label: "MACD", kind: "pane" },
  { id: "rsi", label: "RSI", kind: "pane" },
];
