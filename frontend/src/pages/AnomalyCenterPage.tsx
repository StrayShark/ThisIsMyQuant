import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { BellRing, Zap } from "lucide-react";
import { api } from "@/api/client";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

function severityVariant(severity: string): "up" | "down" | "secondary" | "outline" {
  if (severity === "high") return "down";
  if (severity === "medium") return "up";
  if (severity === "watch") return "outline";
  return "secondary";
}

function severityText(severity: string) {
  const map: Record<string, string> = {
    high: "高",
    medium: "中",
    watch: "观察",
  };
  return map[severity] ?? severity;
}

export function AnomalyCenterPage() {
  const queryClient = useQueryClient();
  const { data, isLoading } = useQuery({
    queryKey: ["anomaly-center"],
    queryFn: () => api.getProfessionalDashboard(),
    refetchInterval: 30_000,
  });
  const { data: settings } = useQuery({
    queryKey: ["settings", "anomaly-center"],
    queryFn: () => api.getSettings(),
  });

  const trigger = useMutation({
    mutationFn: (symbol: string) => api.triggerAnalysis({ symbol, trigger: "anomaly" }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["reports-all"] });
      void queryClient.invalidateQueries({ queryKey: ["professional-dashboard"] });
    },
  });

  const alerts = data?.alerts ?? [];

  return (
    <div className="page-scroll">
      <div className="page-inner space-y-4">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-normal">异动预警中心</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              价格、成交、新闻密度与外盘传导统一进入雷达，触发后可生成异动点评。
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant={settings?.anomaly_enabled ? "up" : "secondary"}>
              {settings?.anomaly_enabled ? "检测开启" : "检测关闭"}
            </Badge>
            <Badge variant="secondary">
              阈值 {settings?.anomaly_price_pct ?? 1.5}% / {settings?.anomaly_window_secs ?? 300}s
            </Badge>
          </div>
        </div>

        {isLoading ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-40 rounded-lg" />
            ))}
          </div>
        ) : alerts.length > 0 ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {alerts.map((alert) => (
              <Card key={`${alert.symbol}-${alert.timestamp}`}>
                <CardHeader className="pb-2">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <CardTitle className="text-base">
                        {alert.product_name}
                        <span className="ml-2 font-mono text-sm text-muted-foreground">
                          {alert.symbol}
                        </span>
                      </CardTitle>
                      <p className="mt-1 text-xs text-muted-foreground">{alert.sector}</p>
                    </div>
                    <Badge variant={severityVariant(alert.severity)}>
                      {severityText(alert.severity)}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  <p className="text-sm leading-relaxed">{alert.reason}</p>
                  <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
                    <p>涨跌：{alert.change_pct.toFixed(2)}%</p>
                    <p>{new Date(alert.timestamp).toLocaleString("zh-CN")}</p>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={trigger.isPending}
                    onClick={() => trigger.mutate(alert.symbol)}
                  >
                    <Zap className="h-3.5 w-3.5" />
                    生成异动报告
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={BellRing}
            title="暂无异动事件"
            description="行情轮询运行后，系统会按价格窗口、新闻密度与外盘联动生成异动观察。"
          />
        )}

        <Card>
          <CardHeader>
            <CardTitle>雷达信号</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-3 text-sm text-muted-foreground md:grid-cols-4">
            <p>价格：窗口涨跌幅、突破、回撤。</p>
            <p>成交：主力连续成交活跃度。</p>
            <p>资讯：金十新闻密度和利多利空归因。</p>
            <p>外盘：WTI、COMEX、CBOT 对内盘传导。</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
