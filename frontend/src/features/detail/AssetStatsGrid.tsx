import { formatAmount, formatVolume } from "@/features/markets/market-utils";
import type React from "react";
import type { MarketAsset, MarketType, StockDetailView } from "@/types";

interface AssetStatsGridProps {
  asset: MarketAsset;
  market: MarketType;
  stockDetail?: StockDetailView | null;
  className?: string;
}

interface StatItemProps {
  label: string;
  value: React.ReactNode;
}

function StatItem({ label, value }: StatItemProps) {
  return (
    <div className="flex flex-col gap-1 rounded-xl border border-border bg-card p-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-sm font-medium tabular-nums text-foreground">{value}</span>
    </div>
  );
}

function maybe(value: number | string | null | undefined, formatter?: (v: number) => string): React.ReactNode {
  if (value === null || value === undefined || value === "") return "--";
  if (typeof value === "number" && Number.isNaN(value)) return "--";
  if (formatter && typeof value === "number") return formatter(value);
  return String(value);
}

function formatPercentValue(value: number) {
  return `${value.toFixed(2)}%`;
}

function formatReportPeriod(period?: string | null) {
  if (!period) return "--";
  if (period.length === 8) return `${period.slice(0, 4)}-${period.slice(4, 6)}-${period.slice(6, 8)}`;
  return period;
}

export function AssetStatsGrid({ asset, market, stockDetail, className }: AssetStatsGridProps) {
  if (market === "futures") {
    return (
      <div className={`grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 ${className ?? ""}`}>
        <StatItem label="成交量" value={maybe(asset.volume, formatVolume)} />
        <StatItem label="持仓量" value={maybe(asset.position_qty ?? null, formatVolume)} />
        <StatItem label="保证金率" value={maybe(null)} />
        <StatItem label="合约乘数" value={maybe(null)} />
        <StatItem label="最小变动价位" value={maybe(null)} />
        <StatItem label="手续费估算" value={maybe(null)} />
        <StatItem label="交易所" value={maybe(asset.exchange)} />
        <StatItem label="相关品种" value={maybe(asset.category ?? asset.sector)} />
      </div>
    );
  }

  const latestBar = stockDetail?.latest_bar;
  const valuation = stockDetail?.latest_valuation;
  const financial = stockDetail?.latest_financial;

  return (
    <div className={`grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 ${className ?? ""}`}>
      <StatItem label="成交额" value={maybe(asset.turnover, formatAmount)} />
      <StatItem label="换手率" value={maybe(latestBar?.turnover_rate, formatPercentValue)} />
      <StatItem label="市值" value={maybe(valuation?.market_cap, formatAmount)} />
      <StatItem label="PE" value={maybe(valuation?.pe_ttm, (v) => v.toFixed(2))} />
      <StatItem label="PB" value={maybe(valuation?.pb, (v) => v.toFixed(2))} />
      <StatItem label="ROE" value={maybe(financial?.roe, formatPercentValue)} />
      <StatItem label="营收同比" value={maybe(financial?.revenue_yoy, formatPercentValue)} />
      <StatItem label="净利同比" value={maybe(financial?.net_profit_yoy, formatPercentValue)} />
      <StatItem label="最新报告期" value={formatReportPeriod(financial?.report_period)} />
      <StatItem label="数据来源" value={financial?.source ?? valuation?.source ?? asset.source ?? "--"} />
    </div>
  );
}
