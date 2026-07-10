import { cn } from "@/lib/utils";
import type { StockMarketBreadth } from "@/types";

interface MarketBreadthStripProps {
  breadth: StockMarketBreadth;
}

function formatAmount(amount?: number | null): string {
  if (amount === undefined || amount === null) return "--";
  if (amount >= 1e12) return `${(amount / 1e12).toFixed(2)}万亿`;
  if (amount >= 1e8) return `${(amount / 1e8).toFixed(2)}亿`;
  if (amount >= 1e4) return `${(amount / 1e4).toFixed(2)}万`;
  return amount.toFixed(0);
}

export function MarketBreadthStrip({ breadth }: MarketBreadthStripProps) {
  const total = breadth.up_count + breadth.down_count + breadth.flat_count;
  const upPct = total > 0 ? (breadth.up_count / total) * 100 : 0;
  const downPct = total > 0 ? (breadth.down_count / total) * 100 : 0;
  const amountChange = breadth.amount_change_pct ?? 0;

  return (
    <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="text-sm font-medium">市场宽度</div>
        <div className="text-xs text-muted-foreground">
          {breadth.trade_date ? `${breadth.trade_date.slice(0, 4)}-${breadth.trade_date.slice(4, 6)}-${breadth.trade_date.slice(6, 8)}` : "--"}
        </div>
      </div>
      <div className="mt-3 flex items-center gap-6 text-sm">
        <div>
          <span className="text-emerald-500 font-semibold">{breadth.up_count}</span>
          <span className="ml-1 text-xs text-muted-foreground">上涨</span>
        </div>
        <div>
          <span className="text-rose-500 font-semibold">{breadth.down_count}</span>
          <span className="ml-1 text-xs text-muted-foreground">下跌</span>
        </div>
        <div>
          <span className="text-muted-foreground font-semibold">{breadth.flat_count}</span>
          <span className="ml-1 text-xs text-muted-foreground">平盘</span>
        </div>
        <div>
          <span className="text-amber-500 font-semibold">{breadth.limit_up_count}</span>
          <span className="ml-1 text-xs text-muted-foreground">涨停</span>
        </div>
        <div>
          <span className="text-slate-500 font-semibold">{breadth.limit_down_count}</span>
          <span className="ml-1 text-xs text-muted-foreground">跌停</span>
        </div>
        <div>
          <span className="font-semibold tabular-nums">{formatAmount(breadth.total_amount)}</span>
          <span className="ml-1 text-xs text-muted-foreground">成交额</span>
          {breadth.amount_change_pct !== undefined && breadth.amount_change_pct !== null && (
            <span className={cn("ml-1 text-xs tabular-nums", amountChange >= 0 ? "text-emerald-500" : "text-rose-500")}>
              {amountChange >= 0 ? "+" : ""}
              {amountChange.toFixed(2)}%
            </span>
          )}
        </div>
      </div>
      <div className="mt-3 h-2 w-full overflow-hidden rounded-full bg-muted">
        <div
          className="flex h-full"
          style={{ width: `${total > 0 ? 100 : 0}%` }}
        >
          <div className="h-full bg-emerald-500" style={{ width: `${upPct}%` }} />
          <div className="h-full bg-rose-500" style={{ width: `${downPct}%` }} />
          <div className="h-full bg-muted-foreground/30" style={{ width: `${100 - upPct - downPct}%` }} />
        </div>
      </div>
    </div>
  );
}
