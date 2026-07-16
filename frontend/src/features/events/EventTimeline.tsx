import { useMemo } from "react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { PanelSkeleton } from "@/components/ui/panel-skeleton";
import { EventImpactTags } from "./EventImpactTags";
import type { EventImportance, EventSource, MarketEvent } from "@/types";

interface EventTimelineProps {
  events: MarketEvent[];
  isLoading?: boolean;
  selectedId?: string | null;
  onSelect: (event: MarketEvent) => void;
  onSymbolClick?: (symbol: string) => void;
  onSectorClick?: (sector: string) => void;
  className?: string;
}

const SOURCE_LABELS: Record<EventSource, string> = {
  jin10: "金十",
  calendar: "日历",
  announcement: "公告",
  earnings: "财报",
  industry: "产业",
};

const IMPORTANCE_DOT: Record<EventImportance, string> = {
  high: "bg-red-500",
  medium: "bg-amber-500",
  low: "bg-slate-400",
};

const DIRECTION_BADGE: Record<
  Exclude<MarketEvent["direction"], null | undefined>,
  { label: string; variant: "up" | "down" | "secondary" }
> = {
  bullish: { label: "偏多", variant: "up" },
  bearish: { label: "偏空", variant: "down" },
  neutral: { label: "中性", variant: "secondary" },
};

function formatTime(iso: string) {
  return new Date(iso).toLocaleString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatGroupLabel(iso: string) {
  const d = new Date(iso);
  const today = new Date();
  const isToday =
    d.getFullYear() === today.getFullYear() &&
    d.getMonth() === today.getMonth() &&
    d.getDate() === today.getDate();

  if (isToday) return "今天";

  return d.toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
    weekday: "short",
  });
}

export function EventTimeline({
  events,
  isLoading,
  selectedId,
  onSelect,
  onSymbolClick,
  onSectorClick,
  className,
}: EventTimelineProps) {
  const grouped = useMemo(() => {
    const map = new Map<string, MarketEvent[]>();
    for (const event of events) {
      const key = formatGroupLabel(event.display_time);
      const list = map.get(key) ?? [];
      list.push(event);
      map.set(key, list);
    }
    return Array.from(map.entries());
  }, [events]);

  if (isLoading) {
    return (
      <div className={cn("rounded-2xl border border-border bg-card p-4", className)}>
        <PanelSkeleton rows={6} />
      </div>
    );
  }

  if (events.length === 0) {
    return (
      <div
        className={cn(
          "rounded-2xl border border-border bg-card p-8 text-center text-sm text-muted-foreground",
          className
        )}
      >
        暂无符合条件的事件
      </div>
    );
  }

  return (
    <div className={cn("space-y-4", className)}>
      {grouped.map(([dateLabel, items]) => (
        <div key={dateLabel}>
          <div className="sticky top-0 z-10 mb-2 flex items-center gap-2 bg-background/95 py-1 backdrop-blur">
            <span className="text-xs font-semibold text-foreground">{dateLabel}</span>
            <span className="text-xs text-muted-foreground">({items.length})</span>
          </div>
          <div className="space-y-2">
            {items.map((event) => {
              const selected = selectedId === event.id;
              const direction = event.direction
                ? DIRECTION_BADGE[event.direction]
                : null;

              return (
                <button
                  key={event.id}
                  type="button"
                  onClick={() => onSelect(event)}
                  className={cn(
                    "w-full rounded-xl border p-3 text-left transition-colors",
                    selected
                      ? "border-primary bg-primary/5"
                      : "border-border bg-card hover:border-muted-foreground/30"
                  )}
                >
                  <div className="mb-2 flex items-start justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <span
                        className={cn(
                          "h-2 w-2 shrink-0 rounded-full",
                          IMPORTANCE_DOT[event.importance]
                        )}
                      />
                      <span className="text-xs text-muted-foreground">
                        {formatTime(event.display_time)}
                      </span>
                      <Badge variant="secondary" className="text-[10px]">
                        {SOURCE_LABELS[event.source]}
                      </Badge>
                      {direction && (
                        <Badge variant={direction.variant} className="text-[10px]">
                          {direction.label}
                        </Badge>
                      )}
                    </div>
                  </div>

                  <h3
                    className={cn(
                      "mb-2 text-sm font-medium leading-snug",
                      selected ? "text-primary" : "text-foreground"
                    )}
                  >
                    {event.title}
                  </h3>

                  <EventImpactTags
                    event={event}
                    onSymbolClick={onSymbolClick}
                    onSectorClick={onSectorClick}
                  />
                </button>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
