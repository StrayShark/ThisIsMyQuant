import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Calendar, Sparkles, Loader2 } from "lucide-react";
import { PageShell } from "@/components/layout/PageShell";
import { PageHeader } from "@/components/layout/PageHeader";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/api/client";
import { cn } from "@/lib/utils";
import { formatTimeAgo } from "@/features/markets/market-utils";
import { EventImpactTags } from "@/features/events/EventImpactTags";
import { AiSummaryModal } from "@/features/ai/AiSummaryModal";
import { useAiSummary } from "@/features/ai/useAiSummary";
import type { MarketEvent, EventImportance, EventSource } from "@/types";

const IMPORTANCE_FILTERS: { value: EventImportance | "all"; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "high", label: "高" },
  { value: "medium", label: "中" },
  { value: "low", label: "低" },
];

const SOURCE_FILTERS: { value: EventSource | "all"; label: string }[] = [
  { value: "all", label: "全部来源" },
  { value: "jin10", label: "金十" },
  { value: "calendar", label: "日历" },
  { value: "announcement", label: "公告" },
  { value: "earnings", label: "财报" },
  { value: "industry", label: "行业" },
];

const SOURCE_LABELS: Record<EventSource, string> = {
  jin10: "金十",
  calendar: "日历",
  announcement: "公告",
  earnings: "财报",
  industry: "行业",
};

function EventImportanceBadge({ importance }: { importance: MarketEvent["importance"] }) {
  const config = {
    high: { label: "高", className: "bg-[var(--color-up-bg)] text-[var(--color-up)]" },
    medium: { label: "中", className: "bg-muted text-muted-foreground" },
    low: { label: "低", className: "bg-secondary text-secondary-foreground" },
  };
  const { label, className } = config[importance] ?? config.medium;
  return (
    <span className={`rounded-full px-2 py-0.5 text-[10px] font-medium ${className}`}>{label}</span>
  );
}

export function EventsPage() {
  const navigate = useNavigate();
  const [importance, setImportance] = useState<EventImportance | "all">("all");
  const [source, setSource] = useState<EventSource | "all">("all");
  const aiSummary = useAiSummary();
  const [activeEvent, setActiveEvent] = useState<MarketEvent | null>(null);

  const { data: result, isLoading } = useQuery({
    queryKey: ["market-events", source, importance],
    queryFn: () =>
      api.listMarketEvents({
        source,
        importance,
        limit: 50,
      }),
  });

  const events = result?.events ?? [];

  const handleAnalyze = (event: MarketEvent) => {
    setActiveEvent(event);
    aiSummary.generate({
      task_type: "event_impact",
      target_symbol: event.affected_symbols[0] ?? undefined,
      prompt: `分析事件「${event.title}」对相关标的影响`,
    });
  };

  const handleSymbolClick = (symbol: string) => {
    const isStock = /^\d{6}(\.SH|\.SZ)?$/i.test(symbol);
    const path = isStock
      ? `/markets/stocks/${encodeURIComponent(symbol)}`
      : `/markets/futures/${encodeURIComponent(symbol)}`;
    navigate(path);
  };

  return (
    <PageShell>
      <PageHeader
        title="事件资讯"
        description="聚合财经日历、公告与市场事件"
      />

      <div className="mb-4 flex flex-wrap items-center gap-2">
        {SOURCE_FILTERS.map((filter) => (
          <Button
            key={filter.value}
            type="button"
            variant={source === filter.value ? "default" : "outline"}
            size="sm"
            className="h-7 rounded-full text-xs"
            onClick={() => setSource(filter.value)}
          >
            {filter.label}
          </Button>
        ))}
      </div>

      <div className="mb-4 flex flex-wrap items-center gap-2">
        {IMPORTANCE_FILTERS.map((filter) => (
          <Button
            key={filter.value}
            type="button"
            variant={importance === filter.value ? "default" : "outline"}
            size="sm"
            className="h-7 rounded-full text-xs"
            onClick={() => setImportance(filter.value)}
          >
            {filter.label}
          </Button>
        ))}
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-24 w-full rounded-xl" />
          ))}
        </div>
      ) : events.length === 0 ? (
        <Card>
          <CardContent className="flex h-64 flex-col items-center justify-center gap-4 p-8 text-center">
            <Calendar className="h-8 w-8 text-muted-foreground" />
            <p className="text-sm text-muted-foreground">暂无事件</p>
          </CardContent>
        </Card>
      ) : (
        <div className="relative space-y-4 pl-4">
          <div className="absolute left-[7px] top-2 bottom-2 w-px bg-border" />
          {events.map((event) => (
            <div key={event.id} className="relative pl-6">
              <div className="absolute left-0 top-2 h-3.5 w-3.5 rounded-full border-2 border-background bg-primary" />
              <Card>
                <CardContent className="p-4">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className="text-sm font-medium text-foreground">{event.title}</h3>
                        <EventImportanceBadge importance={event.importance} />
                      </div>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {event.summary || `类型：${event.event_type}`}
                      </p>
                      <div className="mt-2 flex flex-wrap items-center gap-3 text-[11px] text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {formatTimeAgo(event.display_time)}
                        </span>
                        <Badge variant="outline" className="text-[10px]">
                          来源：{SOURCE_LABELS[event.source] ?? event.source}
                        </Badge>
                        {event.direction && (
                          <Badge
                            variant={
                              event.direction === "bullish"
                                ? "up"
                                : event.direction === "bearish"
                                  ? "down"
                                  : "secondary"
                            }
                            className="text-[10px]"
                          >
                            {event.direction === "bullish"
                              ? "偏多"
                              : event.direction === "bearish"
                                ? "偏空"
                                : "中性"}
                          </Badge>
                        )}
                      </div>
                      <EventImpactTags
                        event={event}
                        className="mt-2"
                        onSymbolClick={handleSymbolClick}
                      />
                    </div>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className={cn(
                        "h-8 shrink-0 gap-1.5 rounded-full text-xs",
                        activeEvent?.id === event.id &&
                          aiSummary.isLoading &&
                          "opacity-70"
                      )}
                      onClick={() => handleAnalyze(event)}
                      disabled={activeEvent?.id === event.id && aiSummary.isLoading}
                    >
                      {activeEvent?.id === event.id && aiSummary.isLoading ? (
                        <Loader2 className="h-3.5 w-3.5 animate-spin" />
                      ) : (
                        <Sparkles className="h-3.5 w-3.5" />
                      )}
                      AI 影响分析
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          ))}
        </div>
      )}

      <AiSummaryModal
        isOpen={aiSummary.isOpen}
        onClose={aiSummary.close}
        title={activeEvent ? `AI 影响分析：${activeEvent.title}` : "AI 事件影响分析"}
        report={aiSummary.report}
        isLoading={aiSummary.isLoading}
        error={aiSummary.error}
      />
    </PageShell>
  );
}
