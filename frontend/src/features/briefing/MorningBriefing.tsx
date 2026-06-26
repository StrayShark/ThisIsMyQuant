import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Sunrise } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { Badge } from "@/components/ui/badge";
import { getFuturesProduct } from "@/data/futures";
import { isWithinHours, triggerLabel } from "@/data/calendar";

import type { CalendarEvent } from "@/types";

function normalizeWatchSymbol(sym: string): string {
  const s = sym.trim().toUpperCase();
  if (/^[A-Z]{1,3}\d+$/.test(s)) {
    return `${s.replace(/\d+$/, "")}0`;
  }
  return s;
}

function isRecent(iso: string, hours = 36): boolean {
  const ms = Date.parse(iso);
  if (Number.isNaN(ms)) return false;
  return Date.now() - ms <= hours * 3600 * 1000;
}

function triggerPriority(trigger: string): number {
  if (trigger === "tomorrow") return 0;
  if (trigger === "short_term") return 1;
  if (trigger === "scheduled") return 2;
  return 3;
}

function reportPreview(content: string, max = 48): string {
  const text = content.replace(/\s+/g, " ").trim();
  if (text.length <= max) return text;
  return `${text.slice(0, max)}…`;
}

export function MorningBriefing() {
  const watchlist = useAppStore((s) => s.watchlist);

  const { data: reports } = useQuery({
    queryKey: ["reports-briefing"],
    queryFn: () => api.listReports({ limit: 80 }),
    staleTime: 60_000,
  });

  const { data: calendar } = useQuery<CalendarEvent[]>({
    queryKey: ["calendar", "briefing-48h"],
    queryFn: () => api.listCalendarEvents({ min_star: 4 }),
    staleTime: 300_000,
  });

  const watchReports = useMemo(() => {
    if (!reports?.length) return [];
    const bases = new Set(watchlist.map(normalizeWatchSymbol));
    return reports
      .filter((r) => bases.has(normalizeWatchSymbol(r.symbol)))
      .filter((r) => isRecent(r.created_at))
      .sort(
        (a, b) =>
          triggerPriority(a.trigger) - triggerPriority(b.trigger) ||
          Date.parse(b.created_at) - Date.parse(a.created_at)
      )
      .slice(0, 4);
  }, [reports, watchlist]);

  const upcomingMacro = useMemo(
    () => (calendar ?? []).filter((e) => isWithinHours(e.pub_time, 48)).slice(0, 5),
    [calendar]
  );

  if (watchReports.length === 0 && upcomingMacro.length === 0) {
    return null;
  }

  return (
    <div className="shrink-0 border-b border-border bg-muted/20 px-4 py-2.5">
      <div className="flex flex-wrap items-start gap-x-6 gap-y-2">
        <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
          <Sunrise className="h-3.5 w-3.5 text-primary" />
          今日待关注
        </div>

        {watchReports.length > 0 && (
          <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
            <span className="text-[11px] text-muted-foreground">关注品种报告</span>
            {watchReports.map((r) => (
              <Link key={r.id} to={`/reports/${r.id}`} title={reportPreview(r.content, 120)}>
                <Badge variant="secondary" className="max-w-[240px] cursor-pointer truncate font-normal hover:bg-muted">
                  {getFuturesProduct(r.symbol)?.name ?? r.symbol}
                  <span className="ml-1 opacity-70">{triggerLabel(r.trigger)}</span>
                </Badge>
              </Link>
            ))}
          </div>
        )}

        {upcomingMacro.length > 0 && (
          <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
            <span className="text-[11px] text-muted-foreground">48h 宏观</span>
            {upcomingMacro.map((e) => (
              <Badge key={e.id} variant="outline" className="max-w-[220px] truncate font-normal">
                {e.pub_time.slice(5, 16)} · {e.country} {e.name.slice(0, 12)}
                {e.name.length > 12 ? "…" : ""}
              </Badge>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
