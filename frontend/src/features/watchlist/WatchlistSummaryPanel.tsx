import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/api/client";

export function WatchlistSummaryPanel() {
  const { data: summary, isLoading } = useQuery({
    queryKey: ["watchlist-summary"],
    queryFn: () => api.getWatchlistSummary(),
  });

  const stats = [
    { label: "自选总数", value: summary?.total_count ?? 0 },
    { label: "期货", value: summary?.futures_count ?? 0 },
    { label: "A股", value: summary?.stock_count ?? 0 },
    { label: "异动", value: summary?.move_count ?? 0 },
    { label: "事件", value: summary?.event_count ?? 0 },
  ];

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">自选概览</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="grid grid-cols-2 gap-3">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-16 w-full rounded-xl" />
            ))}
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-3">
            {stats.map((stat) => (
              <div
                key={stat.label}
                className="rounded-xl border border-border bg-muted/30 p-3"
              >
                <div className="text-xs text-muted-foreground">{stat.label}</div>
                <div className="mt-1 text-xl font-semibold tabular-nums text-foreground">
                  {stat.value}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
