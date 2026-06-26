import { useMemo, useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { CalendarDays } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { Badge } from "@/components/ui/badge";
import { FilterPill } from "@/components/ui/filter-pill";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { PanelSkeleton } from "@/components/ui/panel-skeleton";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  calendarKeywordFromEvent,
  dimensionForCalendarCountry,
  uniqueCountries,
} from "@/data/calendar";
import type { CalendarEvent } from "@/types";

function starLabel(star: number) {
  return "★".repeat(Math.min(star, 5));
}

function formatValues(event: CalendarEvent) {
  const parts: string[] = [];
  if (event.previous) parts.push(`前值 ${event.previous}`);
  if (event.consensus) parts.push(`预期 ${event.consensus}`);
  if (event.actual) parts.push(`公布 ${event.actual}`);
  if (event.unit && !parts.some((p) => p.includes(event.unit!))) {
    parts.push(event.unit);
  }
  return parts.join(" · ");
}

export function CalendarPanel() {
  const newsFocus = useAppStore((s) => s.newsFocus);
  const setNewsFocus = useAppStore((s) => s.setNewsFocus);
  const [country, setCountry] = useState<string | null>(null);
  const [minStar, setMinStar] = useState(3);
  const [keyword, setKeyword] = useState("");

  const { data: events, isLoading, isError, error } = useQuery<CalendarEvent[]>({
    queryKey: ["calendar", "macro", minStar, country, keyword],
    queryFn: () =>
      api.listCalendarEvents({
        min_star: minStar,
        country: country ?? undefined,
        keyword: keyword.trim() || undefined,
      }),
    refetchInterval: 300_000,
  });

  const showToast = useAppStore((s) => s.showToast);

  useEffect(() => {
    if (isError && error instanceof Error) {
      showToast(error.message);
    }
  }, [isError, error, showToast]);

  const countries = useMemo(() => uniqueCountries(events ?? []), [events]);

  function handleEventClick(event: CalendarEvent) {
    const dimension = dimensionForCalendarCountry(event.country);
    setNewsFocus({
      dimension,
      keyword: calendarKeywordFromEvent(event),
      eventId: event.id,
      eventName: event.name,
    });
  }

  return (
    <Card>
      <CardHeader className="flex-row items-center gap-2 space-y-0">
        <CalendarDays className="h-4 w-4 text-primary" />
        <CardTitle>宏观数据日程</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-muted-foreground">
          金十财经日历 · 未来两周 · 点击事件联动下方资讯
        </p>
        <div className="flex flex-wrap gap-1.5">
          {[3, 4, 5].map((star) => (
            <FilterPill key={star} active={minStar === star} onClick={() => setMinStar(star)}>
              ★{star}+
            </FilterPill>
          ))}
        </div>
        <div className="flex flex-wrap gap-1.5">
          <FilterPill active={country === null} onClick={() => setCountry(null)}>
            全部国家
          </FilterPill>
          {countries.map((c) => (
            <FilterPill key={c} active={country === c} onClick={() => setCountry(c)}>
              {c}
            </FilterPill>
          ))}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Input
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            placeholder="关键词：非农、CPI…"
            className="max-w-[160px] text-xs"
          />
          {["非农", "CPI", "PMI", "美联储"].map((kw) => (
            <FilterPill
              key={kw}
              active={keyword === kw}
              onClick={() => setKeyword((prev) => (prev === kw ? "" : kw))}
            >
              {kw}
            </FilterPill>
          ))}
        </div>
        <ScrollArea className="h-[200px]">
          {isLoading ? (
            <PanelSkeleton rows={5} />
          ) : events && events.length > 0 ? (
            <div className="space-y-3 pr-3">
              {events.map((event) => {
                const active = newsFocus?.eventId === event.id;
                return (
                  <article
                    key={event.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => handleEventClick(event)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") handleEventClick(event);
                    }}
                    className={`cursor-pointer rounded-md border-b border-border pb-3 transition-colors last:border-0 ${
                      active ? "border-l-2 border-l-primary bg-muted/30 pl-2" : "hover:bg-muted/20"
                    }`}
                  >
                    <div className="mb-1 flex flex-wrap items-center gap-1.5">
                      <Badge variant="outline" className="text-[10px]">
                        {event.country}
                      </Badge>
                      <span className="text-[10px] text-amber-600 dark:text-amber-400">
                        {starLabel(event.star)}
                      </span>
                      {event.status === "released" && (
                        <Badge variant="secondary" className="text-[10px]">
                          已公布
                        </Badge>
                      )}
                      <span className="text-[10px] text-muted-foreground">
                        {event.pub_time}
                      </span>
                    </div>
                    <p className="text-sm font-medium leading-snug text-foreground">
                      {event.name}
                    </p>
                    {formatValues(event) && (
                      <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
                        {formatValues(event)}
                      </p>
                    )}
                  </article>
                );
              })}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              暂无日程数据。请确认 JINSHI_ENABLED=true，且已配置 JIN10_MCP_TOKEN。
            </p>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
