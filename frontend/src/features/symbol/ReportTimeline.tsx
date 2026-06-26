import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "@/api/client";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PanelSkeleton } from "@/components/ui/panel-skeleton";
import { triggerLabel } from "@/data/calendar";

interface ReportTimelineProps {
  symbol: string;
}

export function ReportTimeline({ symbol }: ReportTimelineProps) {
  const { data: reports, isLoading } = useQuery({
    queryKey: ["reports-timeline", symbol],
    queryFn: () => api.listReports({ symbol, limit: 30 }),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>近 30 条分析报告</CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <PanelSkeleton rows={4} />
        ) : reports && reports.length > 0 ? (
          <ul className="space-y-2">
            {reports.map((r) => (
              <li key={r.id}>
                <Link
                  to={`/reports/${r.id}`}
                  className="flex flex-wrap items-center gap-2 rounded-md border border-border px-3 py-2 text-sm transition-colors hover:bg-muted/30"
                >
                  <Badge variant="secondary" className="text-[10px]">
                    {triggerLabel(r.trigger)}
                  </Badge>
                  <span className="text-xs text-muted-foreground">
                    {new Date(r.created_at).toLocaleString("zh-CN")}
                  </span>
                  {r.anomaly_reason && (
                    <span className="text-xs text-amber-600 dark:text-amber-400">
                      {r.anomaly_reason}
                    </span>
                  )}
                  <span className="line-clamp-1 flex-1 text-muted-foreground">
                    {r.context_summary}
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">暂无报告</p>
        )}
      </CardContent>
    </Card>
  );
}
