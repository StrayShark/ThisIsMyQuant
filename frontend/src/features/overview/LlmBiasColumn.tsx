import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { TrendingDown, TrendingUp, Minus } from "lucide-react";
import { api } from "@/api/client";
import { getFuturesProduct } from "@/data/futures";
import { triggerLabel } from "@/data/calendar";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { biasLabel, biasVariant, inferReportBias, type MarketBias } from "./infer-bias";
import { isReportDisplayReady, reportDisplaySnippet } from "@/features/analysis/report-text";

function BiasIcon({ bias }: { bias: MarketBias }) {
  if (bias === "long") return <TrendingUp className="h-3.5 w-3.5" />;
  if (bias === "short") return <TrendingDown className="h-3.5 w-3.5" />;
  return <Minus className="h-3.5 w-3.5" />;
}

export function LlmBiasColumn() {
  const { data: reports, isLoading } = useQuery({
    queryKey: ["overview-bias-reports"],
    queryFn: () => api.listReports({ limit: 80 }),
    staleTime: 60_000,
  });

  const items = useMemo(() => {
    if (!reports?.length) return [];
    const bySymbol = new Map<string, (typeof reports)[0]>();
    const sorted = [...reports].sort(
      (a, b) => Date.parse(b.created_at) - Date.parse(a.created_at)
    );
    for (const r of sorted) {
      const key = r.symbol.toLowerCase();
      if (!bySymbol.has(key)) bySymbol.set(key, r);
    }
    return [...bySymbol.values()]
      .filter((report) => isReportDisplayReady(report))
      .map((report) => ({
        report,
        bias: inferReportBias(report),
      }))
      .sort((a, b) => Date.parse(b.report.created_at) - Date.parse(a.report.created_at))
      .slice(0, 16);
  }, [reports]);

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">LLM 多空建议</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {Array.from({ length: 8 }).map((_, i) => (
              <Skeleton key={i} className="h-24 rounded-md" />
            ))}
          </div>
        ) : items.length === 0 ? (
          <p className="text-sm text-muted-foreground">暂无分析报告</p>
        ) : (
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {items.map(({ report, bias }) => {
              const product = getFuturesProduct(report.symbol);
              const hint = reportDisplaySnippet(report, 80);
              return (
                <Link
                  key={report.id}
                  to={`/reports/${report.id}`}
                  className="block rounded-md border border-border bg-muted/10 p-3 transition-colors hover:bg-muted/30"
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex min-w-0 items-center gap-2">
                      <Badge variant={biasVariant(bias)} className="gap-1 text-[10px]">
                        <BiasIcon bias={bias} />
                        {biasLabel(bias)}
                      </Badge>
                      <span className="truncate text-sm font-medium">
                        {product?.name ?? report.symbol}
                      </span>
                    </div>
                    <span className="shrink-0 text-[10px] text-muted-foreground">
                      {triggerLabel(report.trigger)}
                    </span>
                  </div>
                  <p className="mt-1.5 line-clamp-2 text-xs leading-relaxed text-muted-foreground">
                    {hint}
                  </p>
                  <p className="mt-1 text-[10px] text-muted-foreground">
                    {new Date(report.created_at).toLocaleString("zh-CN")}
                  </p>
                </Link>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
