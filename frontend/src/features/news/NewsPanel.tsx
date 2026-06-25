import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Newspaper } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { dimensionLabel } from "@/data/dimensions";
import { getFuturesProduct } from "@/data/futures";

export function NewsPanel() {
  const currentSymbol = useAppStore((s) => s.currentSymbol);
  const product = getFuturesProduct(currentSymbol);
  const [dimension, setDimension] = useState<string | null>(null);

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
        limit: 15,
      }),
    refetchInterval: 60_000,
  });

  return (
    <Card>
      <CardHeader className="flex-row items-center gap-2 space-y-0 pb-2 pt-4">
        <Newspaper className="h-4 w-4 text-primary" />
        <CardTitle className="text-sm font-semibold">分维度资讯</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-muted-foreground">
          {product?.name ?? currentSymbol} · 已分类入库资讯
        </p>
        <div className="flex flex-wrap gap-1.5">
          <button
            type="button"
            onClick={() => setDimension(null)}
            className={`rounded-full px-2.5 py-0.5 text-[11px] transition-colors ${
              dimension === null
                ? "bg-primary text-primary-foreground"
                : "bg-muted text-muted-foreground hover:bg-muted/80"
            }`}
          >
            全部
          </button>
          {(dimensions ?? []).map((d) => (
            <button
              key={d.code}
              type="button"
              onClick={() => setDimension(d.code)}
              className={`rounded-full px-2.5 py-0.5 text-[11px] transition-colors ${
                dimension === d.code
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted text-muted-foreground hover:bg-muted/80"
              }`}
            >
              {d.label}
            </button>
          ))}
        </div>
        <ScrollArea className="h-[220px]">
          {isLoading ? (
            <p className="text-sm text-muted-foreground">加载中…</p>
          ) : news && news.length > 0 ? (
            <div className="space-y-3 pr-3">
              {news.map((item) => {
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
                ? `暂无「${dimensionLabel(dimension)}」维度资讯`
                : "暂无已分类资讯，请确认金十轮询已开启。"}
            </p>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
