import { Sparkles, AlertTriangle, Calendar } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { AiSourceList } from "./AiSourceList";
import type { AiReportSummary } from "@/types";

const TASK_LABELS: Record<string, string> = {
  market_summary: "市场摘要",
  leaderboard_explain: "排行榜解释",
  asset_brief: "标的速览",
  watchlist_summary: "自选摘要",
  position_risk: "持仓风险",
  event_impact: "事件影响",
  custom: "自定义分析",
};

interface AiReportCardProps {
  report?: AiReportSummary | null;
  isLoading?: boolean;
  className?: string;
}

export function AiReportCard({ report, isLoading, className }: AiReportCardProps) {
  if (isLoading) {
    return (
      <Card className={className}>
        <CardHeader className="pb-2">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Sparkles className="h-4 w-4 text-primary" />
            生成中...
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Skeleton className="h-4 w-3/4" />
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-5/6" />
          <Skeleton className="h-4 w-1/2" />
        </CardContent>
      </Card>
    );
  }

  if (!report) {
    return (
      <Card className={className}>
        <CardHeader className="pb-2">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Sparkles className="h-4 w-4 text-primary" />
            报告展示
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            在左侧选择任务并点击生成，或从列表中选择历史任务查看引用式 AI 摘要。
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className={className}>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm">
          <Sparkles className="h-4 w-4 text-primary" />
          {TASK_LABELS[report.task_type] ?? report.task_type}
          {report.target_symbol && (
            <span className="font-mono text-xs text-muted-foreground">({report.target_symbol})</span>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="text-sm leading-relaxed whitespace-pre-wrap text-foreground">
          {report.content}
        </div>

        {report.sources && report.sources.length > 0 && <AiSourceList sources={report.sources} />}

        {report.data_date && (
          <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
            <Calendar className="h-3 w-3" />
            数据日期：{report.data_date}
          </div>
        )}

        <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 p-3">
          <div className="flex items-start gap-2">
            <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
            <p className="text-[11px] leading-relaxed text-amber-600">
              {report.disclaimer || "仅供研究与复盘，不构成投资建议"}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
