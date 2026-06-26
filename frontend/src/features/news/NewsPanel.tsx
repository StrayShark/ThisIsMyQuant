import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Newspaper, X } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { FilterPill } from "@/components/ui/filter-pill";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PanelSkeleton } from "@/components/ui/panel-skeleton";
import { ScrollArea } from "@/components/ui/scroll-area";
import { dimensionLabel } from "@/data/dimensions";
import { getFuturesProduct } from "@/data/futures";

export function NewsPanel() {
  const currentSymbol = useAppStore((s) => s.currentSymbol);
  const newsFocus = useAppStore((s) => s.newsFocus);
  const setNewsFocus = useAppStore((s) => s.setNewsFocus);
  const product = getFuturesProduct(currentSymbol);

  const dimension = newsFocus?.dimension ?? null;

  const { data: dimensions } = useQuery({
    queryKey: ["dimensions", currentSymbol],
    queryFn: () => api.listDimensions(currentSymbol),
  });

  const { data: news, isLoading } = useQuery({
    queryKey: ["news", currentSymbol, dimension],
    queryFn: () =>
      api.listNews({
        symbol: currentSymbol,
        dimension: dimension ?? undefined,
        limit: 20,
      }),
    refetchInterval: 60_000,
  });

  const filteredNews = useMemo(() => {
    if (!news) return [];
    if (!newsFocus?.keyword) return news;
    const kw = newsFocus.keyword.toLowerCase();
    return news.filter(
      (item) =>
        item.title.toLowerCase().includes(kw) ||
        item.summary.toLowerCase().includes(kw)
    );
  }, [news, newsFocus?.keyword]);

  const displayList = newsFocus?.keyword ? filteredNews : news;

  return (
    <Card>
      <CardHeader className="flex-row items-center gap-2 space-y-0">
        <Newspaper className="h-4 w-4 text-primary" />
        <CardTitle>分维度资讯</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-muted-foreground">
          {product?.name ?? currentSymbol} · 已分类入库资讯
        </p>
        {newsFocus && (
          <div className="flex items-start gap-2 rounded-md border border-primary/30 bg-primary/5 px-2.5 py-2">
            <div className="min-w-0 flex-1 text-[11px] leading-relaxed text-foreground">
              日历联动：
              {newsFocus.eventName && (
                <span className="ml-1 font-medium">{newsFocus.eventName}</span>
              )}
              {newsFocus.dimension && (
                <Badge variant="secondary" className="ml-1.5 text-[10px]">
                  {dimensionLabel(newsFocus.dimension)}
                </Badge>
              )}
              {newsFocus.keyword && (
                <span className="ml-1 text-muted-foreground">· 关键词 {newsFocus.keyword}</span>
              )}
            </div>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 shrink-0 p-0"
              onClick={() => setNewsFocus(null)}
              aria-label="清除日历联动"
            >
              <X className="h-3.5 w-3.5" />
            </Button>
          </div>
        )}
        <div className="flex flex-wrap gap-1.5">
          <FilterPill active={dimension === null} onClick={() => setNewsFocus(null)}>
            全部
          </FilterPill>
          {(dimensions ?? []).map((d) => (
            <FilterPill
              key={d.code}
              active={dimension === d.code}
              onClick={() =>
                setNewsFocus({
                  dimension: d.code,
                  keyword: null,
                  eventId: null,
                  eventName: null,
                })
              }
            >
              {d.label}
            </FilterPill>
          ))}
        </div>
        <ScrollArea className="h-[220px]">
          {isLoading ? (
            <PanelSkeleton rows={5} />
          ) : displayList && displayList.length > 0 ? (
            <div className="space-y-3 pr-3">
              {displayList.map((item) => {
                const primary = item.classifications[0];
                return (
                  <article
                    key={item.id || `${item.display_time}-${item.title}`}
                    className="border-b border-border pb-3 last:border-0"
                  >
                    <div className="mb-1 flex flex-wrap items-center gap-1.5">
                      {primary && (
                        <Badge variant="secondary" className="text-[10px]">
                          {primary.dimension_label ||
                            dimensionLabel(primary.dimension_code)}
                        </Badge>
                      )}
                      <span className="text-[10px] text-muted-foreground">
                        {item.display_time}
                      </span>
                    </div>
                    <p className="text-sm font-medium leading-snug text-foreground">
                      {item.title}
                    </p>
                    {item.summary && (
                      <p className="mt-1 line-clamp-2 text-xs leading-relaxed text-muted-foreground">
                        {item.summary}
                      </p>
                    )}
                  </article>
                );
              })}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              {dimension
                ? `暂无「${dimensionLabel(dimension)}」维度资讯${
                    newsFocus?.keyword ? `（关键词 ${newsFocus.keyword}）` : ""
                  }`
                : "暂无已分类资讯，请确认金十轮询已开启。"}
            </p>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
