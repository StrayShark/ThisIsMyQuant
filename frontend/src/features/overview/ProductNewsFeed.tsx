import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Newspaper } from "lucide-react";
import { api } from "@/api/client";
import { getFuturesProduct } from "@/data/futures";
import { dimensionLabel } from "@/data/dimensions";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

export function ProductNewsFeed() {
  const { data: news, isLoading } = useQuery({
    queryKey: ["overview-news"],
    queryFn: () => api.listNews({ limit: 24 }),
    staleTime: 120_000,
  });

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm font-semibold">
          <Newspaper className="h-4 w-4 text-primary" />
          资讯 · 宏观摘要
        </CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-2">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-16 rounded-md" />
            ))}
          </div>
        ) : !news?.length ? (
          <p className="text-sm text-muted-foreground">暂无资讯</p>
        ) : (
          <ul className="max-h-[360px] space-y-2 overflow-y-auto pr-1">
            {news.map((item) => (
              <li
                key={item.id}
                className="rounded-md border border-border bg-muted/10 px-3 py-2"
              >
                <p className="text-sm font-medium leading-snug">{item.title}</p>
                {item.summary && (
                  <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{item.summary}</p>
                )}
                <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
                  <span className="text-[10px] text-muted-foreground">
                    {new Date(item.display_time).toLocaleString("zh-CN")}
                  </span>
                  {item.classifications.slice(0, 3).map((c, i) => (
                    <Link
                      key={`${item.id}-${i}`}
                      to={`/workspace?symbol=${c.symbol}`}
                      className="inline-flex items-center gap-1"
                    >
                      <Badge variant="secondary" className="text-[10px] font-normal">
                        {getFuturesProduct(c.symbol)?.name ?? c.symbol}
                        · {dimensionLabel(c.dimension_code)}
                      </Badge>
                    </Link>
                  ))}
                </div>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
