import type { CalendarEvent } from "@/types";

/** 日历事件国家 → 资讯维度（用于联动筛选）。 */
export function dimensionForCalendarCountry(country: string): string {
  if (country.startsWith("中国")) return "macro";
  return "overseas_finance";
}

export function calendarKeywordFromEvent(event: CalendarEvent): string {
  const name = event.name;
  for (const kw of ["CPI", "PPI", "PCE", "非农", "PMI", "美联储", "FOMC", "GDP", "LPR"]) {
    if (name.includes(kw)) return kw;
  }
  return name.slice(0, 8);
}

export function parsePubTime(pubTime: string): number | null {
  if (!pubTime) return null;
  const normalized = pubTime.includes("T") ? pubTime : pubTime.replace(" ", "T");
  const withSec = normalized.length === 16 ? `${normalized}:00` : normalized;
  const ms = Date.parse(withSec);
  return Number.isNaN(ms) ? null : ms;
}

export function isWithinHours(pubTime: string, hours: number): boolean {
  const ms = parsePubTime(pubTime);
  if (ms === null) return false;
  const now = Date.now();
  return ms >= now && ms <= now + hours * 3600 * 1000;
}

export function uniqueCountries(events: CalendarEvent[]): string[] {
  const set = new Set<string>();
  for (const e of events) {
    if (e.country) set.add(e.country);
  }
  return Array.from(set).sort((a, b) => a.localeCompare(b, "zh-CN"));
}

export const TRIGGER_LABELS: Record<string, string> = {
  scheduled: "定时",
  daily: "每日",
  realtime: "实时",
  manual: "手动",
  anomaly: "异动",
  tomorrow: "明日展望",
  short_term: "短期研判",
};

export function triggerLabel(trigger: string): string {
  return TRIGGER_LABELS[trigger] ?? trigger;
}
