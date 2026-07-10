import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { cn } from "@/lib/utils";
import { ArrowLeft } from "lucide-react";

interface StockIndustryViewProps {
  boardCode: string;
  onBack: () => void;
}

function formatAmount(amount?: number | null): string {
  if (amount === undefined || amount === null) return "--";
  if (amount >= 1e12) return `${(amount / 1e12).toFixed(2)}万亿`;
  if (amount >= 1e8) return `${(amount / 1e8).toFixed(2)}亿`;
  if (amount >= 1e4) return `${(amount / 1e4).toFixed(2)}万`;
  return amount.toFixed(0);
}

export function StockIndustryView({ boardCode, onBack }: StockIndustryViewProps) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["stock-industry-detail", boardCode],
    queryFn: () => api.getStockIndustryDetail({ board_code: boardCode }),
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

  const { board, snapshot, members } = data;
  const pct = snapshot?.pct_chg ?? 0;

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onBack}
          className="inline-flex items-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent"
        >
          <ArrowLeft className="h-3.5 w-3.5" />
          返回
        </button>
        <h2 className="text-base font-semibold">{board.board_name}</h2>
        <span
          className={cn(
            "text-sm tabular-nums",
            pct >= 0 ? "text-emerald-500" : "text-rose-500"
          )}
        >
          {pct >= 0 ? "+" : ""}
          {pct.toFixed(2)}%
        </span>
      </div>

      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <div className="rounded-lg border border-border bg-card p-3">
          <div className="text-xs text-muted-foreground">上涨家数</div>
          <div className="text-lg font-semibold text-emerald-500">{snapshot?.up_count ?? "--"}</div>
        </div>
        <div className="rounded-lg border border-border bg-card p-3">
          <div className="text-xs text-muted-foreground">下跌家数</div>
          <div className="text-lg font-semibold text-rose-500">{snapshot?.down_count ?? "--"}</div>
        </div>
        <div className="rounded-lg border border-border bg-card p-3">
          <div className="text-xs text-muted-foreground">成交额</div>
          <div className="text-lg font-semibold tabular-nums">{formatAmount(snapshot?.amount)}</div>
        </div>
        <div className="rounded-lg border border-border bg-card p-3">
          <div className="text-xs text-muted-foreground">净流入</div>
          <div className={cn("text-lg font-semibold tabular-nums", (snapshot?.net_flow ?? 0) >= 0 ? "text-emerald-500" : "text-rose-500")}>
            {formatAmount(snapshot?.net_flow)}
          </div>
        </div>
      </div>

      <div className="rounded-lg border border-border bg-card p-4">
        <h3 className="mb-3 text-sm font-medium">成分股</h3>
        <div className="overflow-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-border text-muted-foreground">
                <th className="py-2 text-left font-medium">代码</th>
                <th className="py-2 text-left font-medium">名称</th>
                <th className="py-2 text-right font-medium">最新价</th>
                <th className="py-2 text-right font-medium">涨跌幅</th>
                <th className="py-2 text-right font-medium">成交额</th>
                <th className="py-2 text-right font-medium">PE(TTM)</th>
                <th className="py-2 text-right font-medium">PB</th>
              </tr>
            </thead>
            <tbody>
              {members.map((stock) => {
                const stockPct = stock.pct_chg ?? 0;
                return (
                  <tr key={stock.ts_code} className="border-b border-border/50 hover:bg-accent/30">
                    <td className="py-2 tabular-nums">{stock.ts_code}</td>
                    <td className="py-2">{stock.name}</td>
                    <td className="py-2 text-right tabular-nums">{stock.close?.toFixed(2) ?? "--"}</td>
                    <td
                      className={cn(
                        "py-2 text-right tabular-nums",
                        stockPct >= 0 ? "text-emerald-500" : "text-rose-500"
                      )}
                    >
                      {stockPct >= 0 ? "+" : ""}
                      {stockPct.toFixed(2)}%
                    </td>
                    <td className="py-2 text-right tabular-nums">{formatAmount(stock.amount)}</td>
                    <td className="py-2 text-right tabular-nums">{stock.pe_ttm?.toFixed(2) ?? "--"}</td>
                    <td className="py-2 text-right tabular-nums">{stock.pb?.toFixed(2) ?? "--"}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
