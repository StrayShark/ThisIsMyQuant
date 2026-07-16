import type { DataDomainCode } from "@/types";

export const DOMAIN_LABELS: Record<DataDomainCode, string> = {
  quotes: "行情报价",
  klines: "K线数据",
  news: "资讯",
  calendar: "财经日历",
  reports: "研报",
  simulation: "模拟交易",
  watchlist: "自选",
  stocks: "A股",
  settings: "设置",
};

export function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function formatDateTime(value?: string | null) {
  if (!value) return "—";
  return new Date(value).toLocaleString();
}

export function formatNumber(value: number) {
  return value.toLocaleString();
}
