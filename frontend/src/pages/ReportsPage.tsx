import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "@/api/client";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { DimensionSummary } from "@/features/analysis/DimensionSummary";
import { reportDisplaySnippet } from "@/features/analysis/report-text";
import { triggerLabel } from "@/data/calendar";
import { getFuturesProduct } from "@/data/futures";
import type { ReportTrigger } from "@/types";
import { FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { FilterPill } from "@/components/ui/filter-pill";

const TRIGGERS: Array<{ value: ReportTrigger | "all"; label: string }> = [
  { value: "all", label: "全部" },
  { value: "tomorrow", label: "明日展望" },
  { value: "short_term", label: "短期研判" },
  { value: "scheduled", label: "定时" },
  { value: "manual", label: "手动" },
  { value: "anomaly", label: "异动" },
];

export function ReportsPage() {
  const [symbolFilter, setSymbolFilter] = useState("");
  const [triggerFilter, setTriggerFilter] = useState<ReportTrigger | "all">("all");
  const [dateFilter, setDateFilter] = useState("");

  const { data: reports, isLoading } = useQuery({
    queryKey: ["reports-all", triggerFilter],
    queryFn: () =>
      api.listReports({
        trigger: triggerFilter === "all" ? undefined : triggerFilter,
        limit: 100,
      }),
  });

  const filtered = useMemo(() => {
    if (!reports) return [];
    const sym = symbolFilter.trim().toLowerCase();
    return reports.filter((r) => {
      if (sym) {
        const name = getFuturesProduct(r.symbol)?.name?.toLowerCase() ?? "";
        if (!r.symbol.toLowerCase().includes(sym) && !name.includes(sym)) {
          return false;
        }
      }
      if (dateFilter) {
        const day = r.created_at.slice(0, 10);
        if (day !== dateFilter) return false;
      }
      return true;
    });
  }, [reports, symbolFilter, dateFilter]);

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <div className="mb-4 flex flex-wrap items-end justify-between gap-3">
          <div className="flex flex-wrap items-end gap-3">
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">品种</label>
              <Input
                value={symbolFilter}
                onChange={(e) => setSymbolFilter(e.target.value)}
                placeholder="rb / 螺纹钢"
                className="w-[140px]"
              />
            </div>
            <div className="space-y-1">
              <label className="text-xs text-muted-foreground">日期</label>
              <Input
                type="date"
                value={dateFilter}
                onChange={(e) => setDateFilter(e.target.value)}
                className="w-[150px]"
              />
            </div>
            <div className="flex flex-wrap gap-1.5 pb-0.5">
              {TRIGGERS.map((t) => (
                <FilterPill
                  key={t.value}
                  active={triggerFilter === t.value}
                  onClick={() => setTriggerFilter(t.value)}
                >
                  {t.label}
                </FilterPill>
              ))}
            </div>
          </div>
          <Button variant="outline" size="sm" asChild>
            <Link to="/reports/compare">对比报告</Link>
          </Button>
        </div>

        {isLoading ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-52 rounded-lg" />
            ))}
          </div>
        ) : filtered.length > 0 ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {filtered.map((r) => (
              <Link key={r.id} to={`/reports/${r.id}`}>
                <Card className="h-full transition-colors hover:border-hairline-strong hover:bg-muted/10">
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between gap-2">
                      <CardTitle className="font-mono text-base">
                        {getFuturesProduct(r.symbol)?.name || r.symbol}
                      </CardTitle>
                      <Badge variant="secondary">{triggerLabel(r.trigger)}</Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    {r.dimension_summary && (
                      <DimensionSummary summary={r.dimension_summary} compact />
                    )}
                    <p className="line-clamp-3 text-sm leading-relaxed text-muted-foreground">
                      {reportDisplaySnippet(r, 200) || "（暂无有效摘要）"}
                    </p>
                    <div className="flex justify-between border-t border-border pt-3 text-xs text-muted-foreground">
                      <span>{new Date(r.created_at).toLocaleString("zh-CN")}</span>
                      <span className="font-mono">
                        {r.provider}
                        {r.news_ids && r.news_ids.length > 0
                          ? ` · ${r.news_ids.length} 条资讯`
                          : ""}
                      </span>
                    </div>
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>
        ) : (
          <EmptyState icon={FileText} title="暂无报告" />
        )}
      </div>
    </div>
  );
}
