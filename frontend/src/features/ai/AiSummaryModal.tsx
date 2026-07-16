import { useEffect, useRef } from "react";
import { X, Sparkles, AlertTriangle, Calendar } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
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

interface AiSummaryModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  report?: AiReportSummary | null;
  isLoading?: boolean;
  error?: string | null;
}

export function AiSummaryModal({
  isOpen,
  onClose,
  title,
  report,
  isLoading,
  error,
}: AiSummaryModalProps) {
  const overlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const displayTitle =
    title ??
    (report ? (TASK_LABELS[report.task_type] ?? report.task_type) : "AI 摘要");

  return (
    <div
      ref={overlayRef}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
      onClick={(e) => {
        if (e.target === overlayRef.current) onClose();
      }}
    >
      <Card className="flex max-h-[85vh] w-full max-w-2xl flex-col">
        <CardHeader className="flex-row items-center justify-between border-b pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <Sparkles className="h-4 w-4 text-primary" />
            {displayTitle}
            {report?.target_symbol && (
              <span className="font-mono text-xs text-muted-foreground">
                ({report.target_symbol})
              </span>
            )}
          </CardTitle>
          <Button type="button" variant="ghost" size="icon" className="h-8 w-8" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </CardHeader>

        <ScrollArea className="flex-1 overflow-hidden">
          <CardContent className="space-y-4 py-4">
            {isLoading && (
              <div className="space-y-3">
                <Skeleton className="h-4 w-3/4" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-4 w-1/2" />
              </div>
            )}

            {!isLoading && error && (
              <div className="rounded-xl border border-destructive/30 bg-destructive/10 p-4 text-sm text-destructive">
                {error}
              </div>
            )}

            {!isLoading && report && (
              <>
                <div className="whitespace-pre-wrap text-sm leading-relaxed text-foreground">
                  {report.content}
                </div>

                {report.sources && report.sources.length > 0 && (
                  <AiSourceList sources={report.sources} />
                )}

                {report.data_date && (
                  <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                    <Calendar className="h-3 w-3" />
                    数据日期：{report.data_date}
                  </div>
                )}
              </>
            )}

            {!isLoading && !error && !report && (
              <p className="text-sm text-muted-foreground">正在准备 AI 分析...</p>
            )}

            <div
              className={cn(
                "rounded-lg border border-amber-500/20 bg-amber-500/10 p-3",
                (isLoading || (!report && !error)) && "opacity-70"
              )}
            >
              <div className="flex items-start gap-2">
                <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
                <p className="text-[11px] leading-relaxed text-amber-600">
                  {report?.disclaimer ?? "仅供研究与复盘，不构成投资建议"}
                </p>
              </div>
            </div>
          </CardContent>
        </ScrollArea>
      </Card>
    </div>
  );
}
