import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { CalendarClock } from "lucide-react";
import { api } from "@/api/client";
import { useFuturesCatalog } from "@/hooks/useFuturesCatalog";
import { getFuturesProduct } from "@/data/futures";
import { isWithinHours } from "@/data/calendar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { mapCalendarToProducts } from "./calendar-products";

export function MacroTimeline() {
  const { data: sectors = [] } = useFuturesCatalog("core");

  const { data: calendar, isLoading } = useQuery({
    queryKey: ["overview-calendar"],
    queryFn: () => api.listCalendarEvents({ min_star: 3 }),
    staleTime: 300_000,
  });

  const items = useMemo(() => {
    if (!calendar?.length) return [];
    return calendar
      .filter((e) => isWithinHours(e.pub_time, 24 * 7))
      .sort((a, b) => a.pub_time.localeCompare(b.pub_time))
      .slice(0, 12)
      .map((event) => mapCalendarToProducts(event, sectors));
  }, [calendar, sectors]);

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm font-semibold">
          <CalendarClock className="h-4 w-4 text-primary" />
          重要数据时间窗
        </CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-2">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-14 rounded-md" />
            ))}
          </div>
        ) : items.length === 0 ? (
          <p className="text-sm text-muted-foreground">暂无高星级日历事件</p>
        ) : (
          <ul className="space-y-2">
            {items.map(({ event, productSymbols }) => (
              <li
                key={event.id}
                className="rounded-md border border-border bg-muted/10 px-3 py-2"
              >
                <div className="flex flex-wrap items-center gap-2">
                  <span className="font-mono text-[11px] text-muted-foreground">
                    {event.pub_time.slice(5, 16)}
                  </span>
                  <Badge variant="outline" className="text-[10px]">
                    {event.country}
                  </Badge>
                  <span className="text-xs font-medium">{event.name}</span>
                  {event.star >= 4 && (
                    <Badge variant="secondary" className="text-[10px]">
                      ★{event.star}
                    </Badge>
                  )}
                </div>
                {productSymbols.length > 0 && (
                  <div className="mt-1.5 flex flex-wrap gap-1">
                    {productSymbols.map((sym) => (
                      <Link
                        key={sym}
                        to={`/workspace?symbol=${sym}`}
                        className="rounded bg-background px-1.5 py-0.5 text-[10px] text-primary hover:underline"
                      >
                        {getFuturesProduct(sym)?.name ?? sym}
                      </Link>
                    ))}
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
