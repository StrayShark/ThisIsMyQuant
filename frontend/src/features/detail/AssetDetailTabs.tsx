import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Newspaper, Calendar, Link2, Briefcase, Sparkles, BarChart3, LayoutDashboard } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/api/client";
import { KlinePanel } from "./KlinePanel";
import { AssetStatsGrid } from "./AssetStatsGrid";
import { formatAmount } from "@/features/markets/market-utils";
import type { MarketAsset, MarketType, StockDetailView } from "@/types";

interface AssetDetailTabsProps {
  symbol: string;
  market: MarketType;
  asset: MarketAsset;
  className?: string;
}

function NewsList({ symbol }: { symbol: string }) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["asset-news", symbol],
    queryFn: () => api.listNews({ symbol, limit: 20 }),
    staleTime: 60_000,
  });

  if (isLoading) return <Skeleton className="h-40 w-full" />;
  if (error) return <EmptyState text="资讯加载失败" />;
  if (!data || data.length === 0) return <EmptyState text="暂无相关资讯" />;

  return (
    <div className="flex flex-col gap-3">
      {data.map((item) => (
        <div key={item.id} className="rounded-xl border border-border bg-card p-4">
          <div className="flex items-start justify-between gap-3">
            <h4 className="text-sm font-medium text-foreground">{item.title}</h4>
            <span className="shrink-0 text-xs text-muted-foreground">
              {new Date(item.display_time).toLocaleString("zh-CN")}
            </span>
          </div>
          <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{item.summary}</p>
          <div className="mt-2 flex flex-wrap gap-2">
            {item.classifications.slice(0, 3).map((c) => (
              <span
                key={`${c.symbol}-${c.dimension_code}`}
                className="rounded-full bg-muted px-2 py-0.5 text-[10px] text-muted-foreground"
              >
                {c.dimension_label}
              </span>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function EventsList({ symbol }: { symbol: string }) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["asset-events", symbol],
    queryFn: () => api.listCalendarEvents({ keyword: symbol }),
    staleTime: 60_000,
  });

  if (isLoading) return <Skeleton className="h-40 w-full" />;
  if (error) return <EmptyState text="事件加载失败" />;
  if (!data || data.length === 0) return <EmptyState text="暂无相关事件" />;

  return (
    <div className="flex flex-col gap-3">
      {data.map((event) => (
        <div key={event.id} className="rounded-xl border border-border bg-card p-4">
          <div className="flex items-start justify-between gap-3">
            <h4 className="text-sm font-medium text-foreground">{event.name}</h4>
            <span className="shrink-0 text-xs text-muted-foreground">{event.pub_time}</span>
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {event.country} · 重要性{" "}
            {"★".repeat(event.star)}
            {"☆".repeat(Math.max(0, 5 - event.star))}
          </p>
          {(event.previous || event.consensus || event.actual) && (
            <div className="mt-2 flex flex-wrap gap-3 text-xs">
              {event.previous && (
                <span className="text-muted-foreground">前值: {event.previous}</span>
              )}
              {event.consensus && (
                <span className="text-muted-foreground">预期: {event.consensus}</span>
              )}
              {event.actual && <span className="text-foreground">公布: {event.actual}</span>}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function PositionsList() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["asset-positions"],
    queryFn: () => api.listSimPositions(),
    staleTime: 30_000,
  });

  if (isLoading) return <Skeleton className="h-40 w-full" />;
  if (error) return <EmptyState text="持仓加载失败" />;
  if (!data || data.length === 0) return <EmptyState text="暂无模拟持仓" />;

  return (
    <div className="flex flex-col gap-3">
      {data.map((pos) => (
        <div key={`${pos.account_id}-${pos.symbol}`} className="rounded-xl border border-border bg-card p-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-foreground">
              {pos.name} ({pos.symbol})
            </span>
            <span
              className={`rounded-full px-2 py-0.5 text-xs ${
                pos.position_side === "long"
                  ? "bg-[var(--color-up-bg)] text-[var(--color-up)]"
                  : "bg-[var(--color-down-bg)] text-[var(--color-down)]"
              }`}
            >
              {pos.position_side === "long" ? "多" : "空"}
            </span>
          </div>
          <div className="mt-2 grid grid-cols-2 gap-2 text-xs text-muted-foreground">
            <span>持仓: {pos.total_qty}</span>
            <span>均价: {pos.avg_price.toFixed(2)}</span>
            <span>保证金: {pos.margin.toFixed(2)}</span>
            <span>浮盈: {pos.unrealized_pnl.toFixed(2)}</span>
          </div>
        </div>
      ))}
    </div>
  );
}

function EmptyState({ text }: { text: string }) {
  return (
    <div className="rounded-xl border border-dashed border-border bg-card p-8 text-center text-sm text-muted-foreground">
      {text}
    </div>
  );
}

function AiSummaryPlaceholder({ disclaimer = "本摘要由 AI 生成，仅供参考，不构成投资建议。" }: { disclaimer?: string }) {
  return (
    <div className="rounded-xl border border-border bg-card p-5">
      <div className="flex items-center gap-2">
        <Sparkles className="h-4 w-4 text-primary" />
        <h4 className="text-sm font-medium text-foreground">AI 摘要</h4>
      </div>
      <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
        暂无针对该标的的 AI 摘要。后续可通过「触发分析」生成引用式研报，汇总行情、资讯、持仓与事件等多维信息。
      </p>
      <p className="mt-3 text-xs text-muted-foreground/70">{disclaimer}</p>
    </div>
  );
}

function formatPercent(value?: number | null) {
  if (value === null || value === undefined || Number.isNaN(value)) return "--";
  return `${value.toFixed(2)}%`;
}

function formatNumber(value?: number | null) {
  if (value === null || value === undefined || Number.isNaN(value)) return "--";
  return value.toFixed(2);
}

function formatReportPeriod(period?: string | null) {
  if (!period) return "--";
  if (period.length === 8) return `${period.slice(0, 4)}-${period.slice(4, 6)}-${period.slice(6, 8)}`;
  return period;
}

function StockResearchPanel({ detail, isLoading }: { detail?: StockDetailView | null; isLoading: boolean }) {
  if (isLoading) return <Skeleton className="h-36 w-full" />;
  const fin = detail?.latest_financial;
  const val = detail?.latest_valuation;
  if (!fin && !val) return <EmptyState text="暂无财务估值数据" />;

  const items = [
    ["报告期", formatReportPeriod(fin?.report_period)],
    ["营收", fin?.revenue ? formatAmount(fin.revenue) : "--"],
    ["营收同比", formatPercent(fin?.revenue_yoy)],
    ["净利润", fin?.net_profit ? formatAmount(fin.net_profit) : "--"],
    ["净利同比", formatPercent(fin?.net_profit_yoy)],
    ["ROE", formatPercent(fin?.roe)],
    ["PE TTM", formatNumber(val?.pe_ttm)],
    ["PB", formatNumber(val?.pb)],
    ["总市值", val?.market_cap ? formatAmount(val.market_cap) : "--"],
    ["估值日期", val?.trade_date ?? "--"],
    ["财务来源", fin?.source ?? "--"],
    ["估值来源", val?.source ?? "--"],
  ];

  return (
    <div className="rounded-xl border border-border bg-card p-4">
      <div className="mb-3 flex items-center justify-between gap-2">
        <h4 className="text-sm font-medium text-foreground">财务与估值</h4>
        <span className="text-xs text-muted-foreground">
          {detail?.symbol.name ?? detail?.symbol.ts_code ?? ""}
        </span>
      </div>
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        {items.map(([label, value]) => (
          <div key={label} className="rounded-lg bg-muted/30 p-3">
            <div className="text-[11px] text-muted-foreground">{label}</div>
            <div className="mt-1 truncate text-sm font-medium tabular-nums text-foreground">{value}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

export function AssetDetailTabs({ symbol, market, asset, className }: AssetDetailTabsProps) {
  const [activeTab, setActiveTab] = useState("overview");
  const isStock = market === "stock";
  const stockDetailQuery = useQuery({
    queryKey: ["stock-detail", symbol],
    queryFn: () => api.getStockDetail(symbol),
    enabled: isStock,
    staleTime: 60_000,
  });

  const tabs = [
    { value: "overview", label: "概览", icon: LayoutDashboard },
    { value: "quotes", label: "行情", icon: BarChart3 },
    { value: "news", label: "资讯", icon: Newspaper },
    { value: "events", label: "事件", icon: Calendar },
    { value: "related", label: "相关标的", icon: Link2 },
    { value: "positions", label: "模拟持仓", icon: Briefcase },
    { value: "ai", label: "AI 摘要", icon: Sparkles },
  ];

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab} className={className}>
      <TabsList className="h-10 w-full justify-start overflow-x-auto bg-muted/40 p-1">
        {tabs.map((tab) => (
          <TabsTrigger
            key={tab.value}
            value={tab.value}
            className="flex items-center gap-1.5 text-xs"
          >
            <tab.icon className="h-3.5 w-3.5" />
            {tab.label}
          </TabsTrigger>
        ))}
      </TabsList>

      <TabsContent value="overview" className="mt-4 space-y-4">
        <AssetStatsGrid
          asset={asset}
          market={market}
          stockDetail={stockDetailQuery.data ?? null}
        />
        {isStock && (
          <StockResearchPanel
            detail={stockDetailQuery.data}
            isLoading={stockDetailQuery.isLoading}
          />
        )}
        <AiSummaryPlaceholder />
      </TabsContent>

      <TabsContent value="quotes" className="mt-4">
        <KlinePanel symbol={symbol} market={market} />
      </TabsContent>

      <TabsContent value="news" className="mt-4">
        <NewsList symbol={symbol} />
      </TabsContent>

      <TabsContent value="events" className="mt-4">
        <EventsList symbol={symbol} />
      </TabsContent>

      <TabsContent value="related" className="mt-4">
        <EmptyState text="相关标的待补充" />
      </TabsContent>

      <TabsContent value="positions" className="mt-4">
        <PositionsList />
      </TabsContent>

      <TabsContent value="ai" className="mt-4">
        <AiSummaryPlaceholder />
      </TabsContent>
    </Tabs>
  );
}
