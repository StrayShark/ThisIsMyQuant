import { useQuery } from "@tanstack/react-query";
import { Cloud, LayoutGrid, Eye, Wallet, Newspaper, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";
import { api } from "@/api/client";
import { DataQualityBadge } from "./DataQualityBadge";

function formatMoney(n: number) {
  return `¥${n.toLocaleString("zh-CN", { maximumFractionDigits: 0 })}`;
}

interface BarItemProps {
  icon: React.ElementType;
  label: string;
  value: React.ReactNode;
  className?: string;
}

function BarItem({ icon: Icon, label, value, className }: BarItemProps) {
  return (
    <div className={cn("flex items-center gap-2 text-sm", className)}>
      <Icon className="h-4 w-4 text-muted-foreground" strokeWidth={1.8} />
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium text-foreground">{value}</span>
    </div>
  );
}

export function GlobalMarketBar({ className }: { className?: string }) {
  const { data: overview } = useQuery({
    queryKey: ["market-overview"],
    queryFn: () => api.getMarketOverview(),
    staleTime: 30_000,
  });

  const { data: watchlistSummary } = useQuery({
    queryKey: ["watchlist-summary"],
    queryFn: () => api.getWatchlistSummary(),
    staleTime: 30_000,
  });

  const { data: events } = useQuery({
    queryKey: ["watchlist-events"],
    queryFn: () => api.getWatchlistEvents(),
    staleTime: 60_000,
  });

  const { data: snapshot } = useQuery({
    queryKey: ["sim-snapshot"],
    queryFn: () => api.getSimAccountSnapshot(),
    staleTime: 30_000,
  });

  const futuresSectorCount = overview?.futures_sectors.length ?? 5;
  const stockIndexCount = overview?.a_stock_indices.length ?? 3;
  const watchlistCount = watchlistSummary?.total_count ?? 0;
  const eventCount = events?.length ?? 0;
  const reportCount = 0; // TODO: wire to AI report summary API when available
  const equity = snapshot?.account.equity ?? 1_000_000;

  return (
    <div
      className={cn(
        "flex flex-wrap items-center gap-x-6 gap-y-2 border-b border-border bg-background px-8 py-2.5",
        className
      )}
    >
      <div className="flex items-center gap-2">
        <Cloud className="h-4 w-4 text-green-500" strokeWidth={1.8} />
        <DataQualityBadge status="live" label="本地数据在线" />
      </div>

      <BarItem icon={LayoutGrid} label="期货板块" value={futuresSectorCount} />
      <BarItem icon={Eye} label="A股观察池" value={stockIndexCount} />
      <BarItem icon={Wallet} label="模拟权益" value={formatMoney(equity)} />
      <BarItem icon={Newspaper} label="资讯" value={eventCount} />
      <BarItem icon={Sparkles} label="AI 报告" value={reportCount} />

      {watchlistCount > 0 && (
        <BarItem icon={Eye} label="自选" value={watchlistCount} />
      )}
    </div>
  );
}
