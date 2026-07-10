import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Card, CardContent } from "@/components/ui/card";
import { SectorHeatmap } from "@/features/overview/SectorHeatmap";
import { LlmBiasColumn } from "@/features/overview/LlmBiasColumn";
import { MacroTimeline } from "@/features/overview/MacroTimeline";
import { ProductNewsFeed } from "@/features/overview/ProductNewsFeed";
import { ProfessionalWorkbench } from "@/features/overview/ProfessionalWorkbench";

function formatMoney(n: number) {
  return `¥${n.toLocaleString("zh-CN", { maximumFractionDigits: 0 })}`;
}

export function OverviewPage() {
  const { data: snapshot } = useQuery({
    queryKey: ["sim-snapshot"],
    queryFn: () => api.getSimAccountSnapshot(),
  });

  return (
    <div className="page-scroll h-full">
      <div className="page-inner space-y-4 pb-6">
        {snapshot && (
          <div className="grid grid-cols-2 gap-3 md:grid-cols-5">
            <Card className="border-l-4 border-l-primary">
              <CardContent className="px-3 py-2">
                <div className="text-xs text-muted-foreground">模拟账户</div>
                <div className="truncate text-sm font-semibold">{snapshot.account.name}</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="px-3 py-2">
                <div className="text-xs text-muted-foreground">权益</div>
                <div className="text-sm font-semibold tabular-nums">{formatMoney(snapshot.account.equity)}</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="px-3 py-2">
                <div className="text-xs text-muted-foreground">可用资金</div>
                <div className="text-sm font-semibold tabular-nums">{formatMoney(snapshot.account.cash_balance)}</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="px-3 py-2">
                <div className="text-xs text-muted-foreground">今日盈亏</div>
                <div className={`text-sm font-semibold tabular-nums ${snapshot.today_pnl >= 0 ? "text-green-500" : "text-red-500"}`}>
                  {formatMoney(snapshot.today_pnl)}
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="px-3 py-2">
                <div className="text-xs text-muted-foreground">风险度</div>
                <div className="text-sm font-semibold tabular-nums">{(snapshot.risk_ratio * 100).toFixed(1)}%</div>
              </CardContent>
            </Card>
          </div>
        )}
        <ProfessionalWorkbench />
        <SectorHeatmap />
        <LlmBiasColumn />
        <div className="grid gap-4 lg:grid-cols-2">
          <MacroTimeline />
          <ProductNewsFeed />
        </div>
      </div>
    </div>
  );
}
