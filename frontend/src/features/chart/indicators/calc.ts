/**
 * 常见技术指标计算（纯 TS，无第三方依赖）。
 * 算法与 src-tauri/src/engine/indicator.rs 保持一致，便于前后端对照。
 * 若需 400+ Pine 指标，可迁移至 lightweight-charts-indicators 包。
 */
import type { Time } from "lightweight-charts";
import type { KLine } from "@/types";

export type LinePoint = { time: Time; value: number };
export type HistPoint = { time: Time; value: number; color?: string };

export function extractCloses(klines: KLine[]): { times: Time[]; closes: number[] } {
  const times = klines.map(
    (k) => Math.floor(new Date(k.start_time).getTime() / 1000) as Time
  );
  const closes = klines.map((k) => k.close);
  return { times, closes };
}

export function sma(values: number[], period: number, times: Time[]): LinePoint[] {
  if (values.length < period) return [];
  const out: LinePoint[] = [];
  for (let i = period - 1; i < values.length; i++) {
    let sum = 0;
    for (let j = i - period + 1; j <= i; j++) sum += values[j];
    out.push({ time: times[i], value: sum / period });
  }
  return out;
}

export function ema(values: number[], period: number): number[] {
  if (values.length === 0) return [];
  const alpha = 2 / (period + 1);
  const out = new Array<number>(values.length);
  out[0] = values[0];
  for (let i = 1; i < values.length; i++) {
    out[i] = alpha * values[i] + (1 - alpha) * out[i - 1];
  }
  return out;
}

export function emaPoints(values: number[], period: number, times: Time[]): LinePoint[] {
  const series = ema(values, period);
  return series.map((value, i) => ({ time: times[i], value }));
}

export function macd(
  closes: number[],
  times: Time[],
  upColor: string,
  downColor: string
): { dif: LinePoint[]; dea: LinePoint[]; hist: HistPoint[] } {
  if (closes.length === 0) return { dif: [], dea: [], hist: [] };
  const emaFast = ema(closes, 12);
  const emaSlow = ema(closes, 26);
  const difValues = emaFast.map((f, i) => f - emaSlow[i]);
  const deaValues = ema(difValues, 9);
  const dif: LinePoint[] = [];
  const dea: LinePoint[] = [];
  const hist: HistPoint[] = [];
  for (let i = 0; i < closes.length; i++) {
    const d = difValues[i];
    const e = deaValues[i];
    const h = 2 * (d - e);
    dif.push({ time: times[i], value: d });
    dea.push({ time: times[i], value: e });
    hist.push({
      time: times[i],
      value: h,
      color: h >= 0 ? `${upColor}99` : `${downColor}99`,
    });
  }
  return { dif, dea, hist };
}

/** Wilder RSI */
export function rsi(closes: number[], times: Time[], period = 14): LinePoint[] {
  if (closes.length <= period) return [];
  const out: LinePoint[] = [];
  let avgGain = 0;
  let avgLoss = 0;
  for (let i = 1; i <= period; i++) {
    const change = closes[i] - closes[i - 1];
    if (change >= 0) avgGain += change;
    else avgLoss -= change;
  }
  avgGain /= period;
  avgLoss /= period;

  const rs0 = avgLoss === 0 ? 100 : avgGain / avgLoss;
  out.push({ time: times[period], value: 100 - 100 / (1 + rs0) });

  for (let i = period + 1; i < closes.length; i++) {
    const change = closes[i] - closes[i - 1];
    const gain = change > 0 ? change : 0;
    const loss = change < 0 ? -change : 0;
    avgGain = (avgGain * (period - 1) + gain) / period;
    avgLoss = (avgLoss * (period - 1) + loss) / period;
    const rs = avgLoss === 0 ? 100 : avgGain / avgLoss;
    out.push({ time: times[i], value: 100 - 100 / (1 + rs) });
  }
  return out;
}

export function bollinger(
  closes: number[],
  times: Time[],
  period = 20,
  mult = 2
): { upper: LinePoint[]; middle: LinePoint[]; lower: LinePoint[] } {
  const upper: LinePoint[] = [];
  const middle: LinePoint[] = [];
  const lower: LinePoint[] = [];
  if (closes.length < period) return { upper, middle, lower };

  for (let i = period - 1; i < closes.length; i++) {
    const slice = closes.slice(i - period + 1, i + 1);
    const mean = slice.reduce((a, b) => a + b, 0) / period;
    const variance = slice.reduce((a, b) => a + (b - mean) ** 2, 0) / period;
    const std = Math.sqrt(variance);
    const t = times[i];
    middle.push({ time: t, value: mean });
    upper.push({ time: t, value: mean + mult * std });
    lower.push({ time: t, value: mean - mult * std });
  }
  return { upper, middle, lower };
}

export function extractOhlc(klines: KLine[]): {
  times: Time[];
  highs: number[];
  lows: number[];
  closes: number[];
} {
  const times = klines.map(
    (k) => Math.floor(new Date(k.start_time).getTime() / 1000) as Time
  );
  return {
    times,
    highs: klines.map((k) => k.high),
    lows: klines.map((k) => k.low),
    closes: klines.map((k) => k.close),
  };
}

/** KDJ (9,3,3) — 与常见行情软件一致 */
export function kdj(
  highs: number[],
  lows: number[],
  closes: number[],
  times: Time[],
  period = 9
): { k: LinePoint[]; d: LinePoint[]; j: LinePoint[] } {
  const kLine: LinePoint[] = [];
  const dLine: LinePoint[] = [];
  const jLine: LinePoint[] = [];
  if (closes.length < period) return { k: kLine, d: dLine, j: jLine };

  let k = 50;
  let d = 50;
  for (let i = period - 1; i < closes.length; i++) {
    const sliceH = highs.slice(i - period + 1, i + 1);
    const sliceL = lows.slice(i - period + 1, i + 1);
    const high = Math.max(...sliceH);
    const low = Math.min(...sliceL);
    const close = closes[i];
    const rsv = high === low ? 50 : ((close - low) / (high - low)) * 100;
    k = (2 / 3) * k + (1 / 3) * rsv;
    d = (2 / 3) * d + (1 / 3) * k;
    const j = 3 * k - 2 * d;
    const t = times[i];
    kLine.push({ time: t, value: k });
    dLine.push({ time: t, value: d });
    jLine.push({ time: t, value: j });
  }
  return { k: kLine, d: dLine, j: jLine };
}

/** Parabolic SAR (step 0.02, max 0.2) */
export function sar(
  highs: number[],
  lows: number[],
  times: Time[],
  step = 0.02,
  maxStep = 0.2
): LinePoint[] {
  const out: LinePoint[] = [];
  if (highs.length < 2) return out;

  let isUp = true;
  let af = step;
  let ep = highs[0];
  let sarVal = lows[0];

  for (let i = 1; i < highs.length; i++) {
    const prevSar = sarVal;
    sarVal = prevSar + af * (ep - prevSar);

    if (isUp) {
      sarVal = Math.min(sarVal, lows[i - 1], i >= 2 ? lows[i - 2] : lows[i - 1]);
      if (lows[i] < sarVal) {
        isUp = false;
        sarVal = ep;
        ep = lows[i];
        af = step;
      } else if (highs[i] > ep) {
        ep = highs[i];
        af = Math.min(af + step, maxStep);
      }
    } else {
      sarVal = Math.max(sarVal, highs[i - 1], i >= 2 ? highs[i - 2] : highs[i - 1]);
      if (highs[i] > sarVal) {
        isUp = true;
        sarVal = ep;
        ep = highs[i];
        af = step;
      } else if (lows[i] < ep) {
        ep = lows[i];
        af = Math.min(af + step, maxStep);
      }
    }
    out.push({ time: times[i], value: sarVal });
  }
  return out;
}
