import { useQuery } from "@tanstack/react-query";
import { Calendar } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/api/client";
import type { MarketEvent } from "@/types";
import { formatTimeAgo } from "@/features/markets/market-utils";

export function WatchlistEventPanel() {
  const { data: events = [], isLoading } = useQuery({
    queryKey: ["watchlist-events"],
    queryFn: () => api.getWatchlistEvents(),
  });

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">相关事件</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-2">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full rounded-md" />
            ))}
          </div>
        ) : events.length === 0 ? (
          <div className="flex h-24 items-center justify-center text-xs text-muted-foreground">
            暂无相关事件
          </div>
        ) : (
          <div className="space-y-2">
            {events.map((event) => (
              <EventRow key={event.id} event={event} />
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function EventRow({ event }: { event: MarketEvent }) {
  return (
    <div className="rounded-lg border border-border/50 bg-muted/20 p-2.5 transition-colors hover:bg-muted/40">
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <div className="flex items-center gap-1.5">
            <Calendar className="h-3 w-3 shrink-0 text-muted-foreground" />
            <span className="truncate text-xs font-medium text-foreground">
              {event.title}
            </span>
          </div>
          <div className="mt-1 line-clamp-2 text-[11px] text-muted-foreground">
            {event.summary || `涉及: ${event.affected_symbols.join(", ") || "—"}`}
          </div>
        </div>
        <EventImportanceBadge importance={event.importance} />
      </div>
      <div className="mt-1.5 flex items-center gap-2 text-[10px] text-muted-foreground">
        <span>{formatTimeAgo(event.display_time)}</span>
        {event.direction && (
          <span
            className={cn(
              "rounded px-1 py-0.5 text-[10px]",
              event.direction === "bullish" && "bg-[var(--color-up-bg)] text-[var(--color-up)]",
              event.direction === "bearish" && "bg-[var(--color-down-bg)] text-[var(--color-down)]",
              event.direction === "neutral" && "bg-muted text-muted-foreground"
            )}
          >
            {event.direction === "bullish" ? "偏多" : event.direction === "bearish" ? "偏空" : "中性"}
          </span>
        )}
      </div>
    </div>
  );
}

function EventImportanceBadge({ importance }: { importance: MarketEvent["importance"] }) {
  const config = {
    high: { label: "高", variant: "up" as const },
    medium: { label: "中", variant: "outline" as const },
    low: { label: "低", variant: "secondary" as const },
  };
  const { label, variant } = config[importance] ?? config.medium;
  return <Badge variant={variant}>{label}</Badge>;
}
