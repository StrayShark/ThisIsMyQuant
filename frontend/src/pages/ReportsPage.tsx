import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "@/api/client";
import { PageHeader } from "@/components/PageHeader";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { FileText } from "lucide-react";
import { getFuturesProduct } from "@/data/futures";

export function ReportsPage() {
  const { data: reports, isLoading } = useQuery({
    queryKey: ["reports-all"],
    queryFn: () => api.listReports({ limit: 50 }),
  });

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <PageHeader
          title="分析报告"
          description="大模型生成的期货走势分析，每日定时与实时触发。"
        />

        {isLoading ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-44 rounded-lg" />
            ))}
          </div>
        ) : reports && reports.length > 0 ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {reports.map((r) => (
              <Link key={r.id} to={`/reports/${r.id}`}>
                <Card className="h-full transition-colors hover:border-hairline-strong hover:bg-muted/10">
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between gap-2">
                      <CardTitle className="font-mono text-base">
                        {getFuturesProduct(r.symbol)?.name || r.symbol}
                      </CardTitle>
                      <Badge variant="secondary">{r.trigger}</Badge>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="line-clamp-4 text-sm leading-relaxed text-muted-foreground">
                      {r.content}
                    </p>
                    <div className="mt-4 flex justify-between border-t border-border pt-3 text-xs text-muted-foreground">
                      <span>{new Date(r.created_at).toLocaleString("zh-CN")}</span>
                      <span className="font-mono">{r.provider}</span>
                    </div>
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={FileText}
            title="暂无报告"
            description="在行情页选择合约并点击「立即分析」，或等待每日定时报告生成。"
          />
        )}
      </div>
    </div>
  );
}
