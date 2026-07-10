import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Skeleton } from "@/components/ui/skeleton";
import { DataQualityBadge } from "./components/DataQualityBadge";
import type { StockFinancialMetric } from "@/types";

interface StockFinancialCenterProps {
  tsCode: string;
}

function formatWanYi(value?: number | null): string {
  if (value === undefined || value === null) return "--";
  if (Math.abs(value) >= 1e12) return `${(value / 1e12).toFixed(2)}万亿`;
  if (Math.abs(value) >= 1e8) return `${(value / 1e8).toFixed(2)}亿`;
  if (Math.abs(value) >= 1e4) return `${(value / 1e4).toFixed(2)}万`;
  return value.toFixed(2);
}

function metricCells(m: StockFinancialMetric) {
  return [
    { label: "营业收入", value: formatWanYi(m.revenue), sub: `同比 ${m.revenue_yoy?.toFixed(2) ?? "--"}%` },
    { label: "净利润", value: formatWanYi(m.net_profit), sub: `同比 ${m.net_profit_yoy?.toFixed(2) ?? "--"}%` },
    { label: "ROE", value: m.roe ? `${m.roe.toFixed(2)}%` : "--", sub: "净资产收益率" },
    { label: "毛利率", value: m.gross_margin ? `${m.gross_margin.toFixed(2)}%` : "--", sub: "毛利/营收" },
    { label: "资产负债率", value: m.debt_ratio ? `${m.debt_ratio.toFixed(2)}%` : "--", sub: "总负债/总资产" },
    { label: "经营现金流", value: formatWanYi(m.operating_cash_flow), sub: "元" },
    { label: "EPS", value: m.eps ? m.eps.toFixed(2) : "--", sub: "每股收益" },
  ];
}

export function StockFinancialCenter({ tsCode }: StockFinancialCenterProps) {
  const { data: metrics, isLoading, error } = useQuery({
    queryKey: ["stock-financials", tsCode],
    queryFn: () => api.listStockFinancials(tsCode),
    staleTime: 300_000,
  });

  if (isLoading) {
    return <Skeleton className="h-64 w-full rounded-md" />;
  }

  const quality = error
    ? { status: "error", message: (error as Error).message }
    : metrics && metrics.length > 0
      ? { status: "available", lastSuccessAt: metrics[0].report_period }
      : { status: "pending", message: "暂无财报数据" };

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <DataQualityBadge
        status={quality.status}
        message={quality.message}
        lastSuccessAt={quality.lastSuccessAt}
      />

      {metrics && metrics.length > 0 ? (
        <div className="space-y-4">
          {metrics.map((m) => (
            <div
              key={m.report_period}
              className="rounded-lg border border-border bg-card p-4 shadow-sm"
            >
              <div className="mb-3 text-sm font-medium">报告期 {m.report_period}</div>
              <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
                {metricCells(m).map((item) => (
                  <div key={item.label} className="rounded-md bg-muted/40 p-2.5">
                    <div className="text-xs text-muted-foreground">{item.label}</div>
                    <div className="mt-0.5 text-sm font-semibold tabular-nums">{item.value}</div>
                    <div className="text-[10px] text-muted-foreground">{item.sub}</div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
          暂无财报数据，请在总览页触发数据同步
        </div>
      )}
    </div>
  );
}
