import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { IndexQuoteCard } from "./components/IndexQuoteCard";
import { MarketBreadthStrip } from "./components/MarketBreadthStrip";
import { IndustryHeatmap } from "./components/IndustryHeatmap";
import type { StockBoardView } from "@/types";

interface AStockDashboardProps {
  onSelectBoard: (board: StockBoardView) => void;
}

export function AStockDashboard({ onSelectBoard }: AStockDashboardProps) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["a-stock-dashboard"],
    queryFn: () => api.getAStockDashboard(),
    staleTime: 30_000,
  });

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        加载中…
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        加载失败：{error?.message ?? "未知错误"}
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        {data.indices.map((quote) => (
          <IndexQuoteCard key={quote.index_code} quote={quote} />
        ))}
      </div>

      <MarketBreadthStrip breadth={data.breadth} />

      <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
        <div className="mb-3 flex items-center justify-between">
          <h3 className="text-sm font-medium">行业/概念热力</h3>
          <span className="text-xs text-muted-foreground">点击下钻</span>
        </div>
        <IndustryHeatmap boards={data.boards} onSelect={onSelectBoard} />
      </div>

      <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
        <div className="mb-2 text-sm font-medium">数据质量</div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span
            className={`inline-block h-2 w-2 rounded-full ${
              data.quality.status === "available" ? "bg-emerald-500" : "bg-amber-500"
            }`}
          />
          <span>
            {data.quality.status === "available" ? "数据可用" : "待更新"}
            {data.quality.last_success_at && ` · 最新 ${data.quality.last_success_at}`}
          </span>
        </div>
      </div>
    </div>
  );
}
