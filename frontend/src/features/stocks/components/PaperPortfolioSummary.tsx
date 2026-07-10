import type { StockPaperAccount } from "@/types";

interface PaperPortfolioSummaryProps {
  account: StockPaperAccount;
}

export function PaperPortfolioSummary({ account }: PaperPortfolioSummaryProps) {
  const totalReturn = account.total_equity - account.initial_balance;
  const totalReturnPct = account.initial_balance > 0 ? (totalReturn / account.initial_balance) * 100 : 0;

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
      <div className="rounded-lg border border-border bg-card p-3">
        <div className="text-xs text-muted-foreground">总资产</div>
        <div className="text-lg font-semibold tabular-nums">{account.total_equity.toFixed(2)}</div>
      </div>
      <div className="rounded-lg border border-border bg-card p-3">
        <div className="text-xs text-muted-foreground">可用现金</div>
        <div className="text-lg font-semibold tabular-nums">{account.cash_balance.toFixed(2)}</div>
      </div>
      <div className="rounded-lg border border-border bg-card p-3">
        <div className="text-xs text-muted-foreground">持仓市值</div>
        <div className="text-lg font-semibold tabular-nums">{account.market_value.toFixed(2)}</div>
      </div>
      <div className="rounded-lg border border-border bg-card p-3">
        <div className="text-xs text-muted-foreground">累计盈亏</div>
        <div className={`text-lg font-semibold tabular-nums ${totalReturn >= 0 ? "text-emerald-500" : "text-rose-500"}`}>
          {totalReturn >= 0 ? "+" : ""}
          {totalReturn.toFixed(2)} ({totalReturnPct.toFixed(2)}%)
        </div>
      </div>
    </div>
  );
}
