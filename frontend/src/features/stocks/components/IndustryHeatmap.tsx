import { cn } from "@/lib/utils";
import type { StockBoardView } from "@/types";

interface IndustryHeatmapProps {
  boards: StockBoardView[];
  onSelect: (board: StockBoardView) => void;
  sortBy?: "pct_chg" | "amount" | "net_flow";
}

export function IndustryHeatmap({ boards, onSelect, sortBy = "pct_chg" }: IndustryHeatmapProps) {
  const sorted = [...boards].sort((a, b) => {
    const va = a[sortBy] ?? 0;
    const vb = b[sortBy] ?? 0;
    return vb - va;
  });

  return (
    <div className="grid grid-cols-4 gap-1 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-8">
      {sorted.map((board) => {
        const pct = board.pct_chg ?? 0;
        const intensity = Math.min(Math.abs(pct) / 3, 1);
        const up = pct >= 0;
        return (
          <button
            key={board.board_code}
            type="button"
            onClick={() => onSelect(board)}
            className={cn(
              "flex flex-col items-center justify-center rounded-md px-1 py-2 text-center transition-colors hover:ring-1 hover:ring-inset hover:ring-primary/50",
              up ? "bg-emerald-500/10 hover:bg-emerald-500/20" : "bg-rose-500/10 hover:bg-rose-500/20"
            )}
            style={{
              backgroundColor: up
                ? `rgba(16, 185, 129, ${0.08 + intensity * 0.32})`
                : `rgba(244, 63, 94, ${0.08 + intensity * 0.32})`,
            }}
          >
            <span className="truncate text-[11px] leading-tight">{board.board_name}</span>
            <span
              className={cn(
                "mt-0.5 text-[11px] font-medium tabular-nums",
                up ? "text-emerald-400" : "text-rose-400"
              )}
            >
              {pct >= 0 ? "+" : ""}
              {pct.toFixed(2)}%
            </span>
          </button>
        );
      })}
    </div>
  );
}
