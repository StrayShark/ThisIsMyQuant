import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { CalendarDays, Sparkles, AlertTriangle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { AiSummaryModal } from "@/features/ai/AiSummaryModal";
import { useAiSummary } from "@/features/ai/useAiSummary";
import { api } from "@/api/client";

export function WatchlistAiSummary() {
  const aiSummary = useAiSummary();
  const { data: items = [] } = useQuery({
    queryKey: ["watchlist-items", "all", "daily-brief"],
    queryFn: () => api.listWatchlistItems(),
    staleTime: 30_000,
  });
  const { data: events = [] } = useQuery({
    queryKey: ["watchlist-events", "daily-brief"],
    queryFn: () => api.getWatchlistEvents(),
    staleTime: 60_000,
  });
  const targetAssets = useMemo(
    () => Array.from(new Set(items.map((item) => item.symbol))).slice(0, 12),
    [items]
  );

  const generateDailyBrief = () => {
    const today = new Date().toLocaleDateString("zh-CN");
    const assetLine = items
      .slice(0, 12)
      .map((item) => `${item.name}(${item.symbol})`)
      .join("、");
    aiSummary.generate({
      task_type: "watchlist_summary",
      target_assets: targetAssets,
      prompt:
        `请生成 ${today} 的自选日报，覆盖：1）今日最需要盯的标的；` +
        "2）价格/事件驱动；3）明日观察清单；4）需要减仓或暂缓交易的风险点。" +
        `自选标的：${assetLine || "暂无"}。` +
        `相关事件数量：${events.length}。请使用简洁小标题，明确数据来源，并保留免责声明。`,
    });
  };

  return (
    <>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="flex items-center gap-2 text-sm font-semibold">
            <CalendarDays className="h-4 w-4 text-primary" />
            AI 自选日报
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-xs leading-relaxed text-muted-foreground">
            汇总自选标的、相关事件和行情变化，生成每日观察清单与风险提示。
          </p>
          <div className="grid grid-cols-3 gap-2 text-center text-xs">
            <div className="rounded-lg border border-border bg-muted/30 p-2">
              <div className="text-muted-foreground">自选</div>
              <div className="mt-1 font-mono text-sm font-medium">{items.length}</div>
            </div>
            <div className="rounded-lg border border-border bg-muted/30 p-2">
              <div className="text-muted-foreground">事件</div>
              <div className="mt-1 font-mono text-sm font-medium">{events.length}</div>
            </div>
            <div className="rounded-lg border border-border bg-muted/30 p-2">
              <div className="text-muted-foreground">引用</div>
              <div className="mt-1 font-mono text-sm font-medium">{targetAssets.length}</div>
            </div>
          </div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="w-full gap-1.5 text-xs"
            onClick={generateDailyBrief}
            disabled={aiSummary.isLoading || targetAssets.length === 0}
          >
            {aiSummary.isLoading ? (
              <>生成中...</>
            ) : (
              <>
                <Sparkles className="h-3.5 w-3.5" />
                生成今日自选日报
              </>
            )}
          </Button>
          <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 p-2.5">
            <div className="flex items-start gap-2">
              <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
              <p className="text-[11px] leading-relaxed text-amber-600">
                免责声明：本摘要由 AI 生成，仅供研究与复盘，不构成投资建议。
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <AiSummaryModal
        isOpen={aiSummary.isOpen}
        onClose={aiSummary.close}
        title="AI 自选日报"
        report={aiSummary.report}
        isLoading={aiSummary.isLoading}
        error={aiSummary.error}
      />
    </>
  );
}
