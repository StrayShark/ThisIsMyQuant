import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Activity, Database } from "lucide-react";
import { api } from "@/api/client";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FilterPill } from "@/components/ui/filter-pill";
import { Skeleton } from "@/components/ui/skeleton";

function signalVariant(signal: string): "up" | "down" | "secondary" | "outline" {
  if (signal === "bullish" || signal === "tracked") return "up";
  if (signal === "bearish") return "down";
  if (signal === "pending") return "outline";
  return "secondary";
}

function qualityText(quality: string) {
  const map: Record<string, string> = {
    "live+history": "实时+历史",
    live: "实时",
    history: "历史",
    pending: "待接入",
  };
  return map[quality] ?? quality;
}

export function FactorCenterPage() {
  const [sector, setSector] = useState("全部");
  const { data, isLoading } = useQuery({
    queryKey: ["factor-center"],
    queryFn: () => api.getProfessionalDashboard(),
    refetchInterval: 120_000,
  });

  const sectors = useMemo(
    () => ["全部", ...Array.from(new Set((data?.factors ?? []).map((f) => f.sector)))],
    [data?.factors]
  );

  const factors = useMemo(() => {
    const list = data?.factors ?? [];
    return sector === "全部" ? list : list.filter((f) => f.sector === sector);
  }, [data?.factors, sector]);

  return (
    <div className="page-scroll">
      <div className="page-inner space-y-4">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-normal">因子中心</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              按五大板块跟踪价格动量、成交活跃度、核心驱动与数据质量。
            </p>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {sectors.map((s) => (
              <FilterPill key={s} active={sector === s} onClick={() => setSector(s)}>
                {s}
              </FilterPill>
            ))}
          </div>
        </div>

        {isLoading ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {[1, 2, 3, 4, 5].map((i) => (
              <Skeleton key={i} className="h-56 rounded-lg" />
            ))}
          </div>
        ) : factors.length > 0 ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {factors.map((factor) => (
              <Card key={factor.symbol} className="h-full">
                <CardHeader className="pb-2">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <CardTitle className="text-base">
                        {factor.product_name}
                        <span className="ml-2 font-mono text-sm text-muted-foreground">
                          {factor.symbol}
                        </span>
                      </CardTitle>
                      <p className="mt-1 text-xs text-muted-foreground">{factor.sector}</p>
                    </div>
                    <Badge variant={factor.quality === "pending" ? "outline" : "secondary"}>
                      {qualityText(factor.quality)}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  {factor.signals.map((signal) => (
                    <div key={signal.label} className="rounded-md border border-border p-3">
                      <div className="flex items-center justify-between gap-3">
                        <p className="text-sm font-medium">{signal.label}</p>
                        <Badge variant={signalVariant(signal.signal)}>{signal.value}</Badge>
                      </div>
                      <p className="mt-2 text-xs leading-relaxed text-muted-foreground">
                        {signal.detail}
                      </p>
                    </div>
                  ))}
                  <p className="border-t border-border pt-3 text-xs text-muted-foreground">
                    更新：{new Date(factor.updated_at).toLocaleString("zh-CN")}
                  </p>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={Database}
            title="暂无因子快照"
            description="等待行情轮询或历史 K 线回填后，系统会生成价格、成交和核心驱动因子。"
          />
        )}

        <Card>
          <CardHeader className="flex-row items-center gap-2 space-y-0">
            <Activity className="h-4 w-4 text-primary" />
            <CardTitle>待接入产业专源</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-3 text-sm text-muted-foreground md:grid-cols-2 xl:grid-cols-4">
            <p>库存 / 仓单：交易所仓单、社会库存、港口库存。</p>
            <p>持仓排名：交易所会员多空持仓、净多净空变化。</p>
            <p>产业利润：钢厂利润、压榨利润、裂解价差、进口利润。</p>
            <p>天气 / 运价：产区天气、红海绕航、舱位与运价指数。</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
