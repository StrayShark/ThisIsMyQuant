import { cn } from "@/lib/utils";
import type { StockIndexQuote } from "@/types";

interface IndexQuoteCardProps {
  quote: StockIndexQuote;
}

function formatAmount(amount?: number | null): string {
  if (amount === undefined || amount === null) return "--";
  if (amount >= 1e12) return `${(amount / 1e12).toFixed(2)}万亿`;
  if (amount >= 1e8) return `${(amount / 1e8).toFixed(2)}亿`;
  if (amount >= 1e4) return `${(amount / 1e4).toFixed(2)}万`;
  return amount.toFixed(0);
}

export function IndexQuoteCard({ quote }: IndexQuoteCardProps) {
  const pct = quote.pct_chg ?? 0;
  const up = pct >= 0;
  return (
    <div className="rounded-lg border border-border bg-card p-3 shadow-sm">
      <div className="text-xs text-muted-foreground">{quote.name}</div>
      <div className={cn("mt-1 text-xl font-semibold tabular-nums", up ? "text-emerald-500" : "text-rose-500")}>
        {quote.close?.toFixed(2) ?? "--"}
      </div>
      <div className={cn("text-xs tabular-nums", up ? "text-emerald-500" : "text-rose-500")}>
        {pct >= 0 ? "+" : ""}
        {pct.toFixed(2)}%
      </div>
      <div className="mt-1 text-[11px] text-muted-foreground">
        成交额 {formatAmount(quote.amount)}
      </div>
    </div>
  );
}
