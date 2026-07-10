import { cn } from "@/lib/utils";
import type { StockSymbolSnapshot } from "@/types";

interface ScreenResultTableProps {
  rows: StockSymbolSnapshot[];
  onSelect?: (row: StockSymbolSnapshot) => void;
}

function formatAmount(amount?: number | null): string {
  if (amount === undefined || amount === null) return "--";
  if (amount >= 1e12) return `${(amount / 1e12).toFixed(2)}万亿`;
  if (amount >= 1e8) return `${(amount / 1e8).toFixed(2)}亿`;
  if (amount >= 1e4) return `${(amount / 1e4).toFixed(2)}万`;
  return amount.toFixed(0);
}

export function ScreenResultTable({ rows, onSelect }: ScreenResultTableProps) {
  if (rows.length === 0) {
    return (
      <div className="flex h-40 items-center justify-center text-xs text-muted-foreground">
        暂无筛选结果
      </div>
    );
  }

  return (
    <div className="overflow-auto">
      <table className="w-full text-xs">
        <thead>
          <tr className="border-b border-border text-muted-foreground">
            <th className="py-2 text-left font-medium">代码</th>
            <th className="py-2 text-left font-medium">名称</th>
            <th className="py-2 text-left font-medium">行业</th>
            <th className="py-2 text-right font-medium">最新价</th>
            <th className="py-2 text-right font-medium">涨跌幅</th>
            <th className="py-2 text-right font-medium">成交额</th>
            <th className="py-2 text-right font-medium">市值</th>
            <th className="py-2 text-right font-medium">PE</th>
            <th className="py-2 text-right font-medium">PB</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const pct = row.pct_chg ?? 0;
            return (
              <tr
                key={row.ts_code}
                className={cn(
                  "border-b border-border/50 hover:bg-accent/30",
                  onSelect && "cursor-pointer"
                )}
                onClick={() => onSelect?.(row)}
              >
                <td className="py-2 tabular-nums">{row.ts_code}</td>
                <td className="py-2">{row.name}</td>
                <td className="py-2 text-muted-foreground">{row.industry ?? "--"}</td>
                <td className="py-2 text-right tabular-nums">{row.close?.toFixed(2) ?? "--"}</td>
                <td
                  className={cn(
                    "py-2 text-right tabular-nums",
                    pct >= 0 ? "text-emerald-500" : "text-rose-500"
                  )}
                >
                  {pct >= 0 ? "+" : ""}
                  {pct.toFixed(2)}%
                </td>
                <td className="py-2 text-right tabular-nums">{formatAmount(row.amount)}</td>
                <td className="py-2 text-right tabular-nums">{formatAmount(row.market_cap)}</td>
                <td className="py-2 text-right tabular-nums">{row.pe_ttm?.toFixed(2) ?? "--"}</td>
                <td className="py-2 text-right tabular-nums">{row.pb?.toFixed(2) ?? "--"}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
