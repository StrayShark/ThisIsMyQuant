import { useQuery } from "@tanstack/react-query";
import { CalendarDays } from "lucide-react";
import { api } from "@/api/client";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
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
  const { data: events, isLoading } = useQuery({
    queryKey: ["calendar", "macro"],
    queryFn: () => api.listCalendarEvents({ min_star: 3 }),
    refetchInterval: 300_000,
  });

  return (
    <Card>
      <CardHeader className="flex-row items-center gap-2 space-y-0 pb-2 pt-4">
        <CalendarDays className="h-4 w-4 text-primary" />
        <CardTitle className="text-sm font-semibold">宏观数据日程</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-muted-foreground">
          金十财经日历 · 未来两周 · ★3 及以上
        </p>
        <ScrollArea className="h-[200px]">
          {isLoading ? (
            <p className="text-sm text-muted-foreground">加载中…</p>
          ) : events && events.length > 0 ? (
            <div className="space-y-3 pr-3">
              {events.map((event) => (
                <article
                  key={event.id}
                  className="border-b border-border pb-3 last:border-0"
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
              ))}
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
