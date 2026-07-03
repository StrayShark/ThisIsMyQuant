/** 主工作台：品种列表 + K 线 + AI / 资讯面板。 */
import { useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import { useAppStore } from "@/app/store";
import { ChartPanel } from "@/features/chart/ChartPanel";
import { SymbolList } from "@/features/market/SymbolList";
import { AiPanel } from "@/features/analysis/AiPanel";
import { CalendarPanel } from "@/features/calendar/CalendarPanel";
import { NewsPanel } from "@/features/news/NewsPanel";

export function DashboardPage() {
  const [searchParams] = useSearchParams();
  const setCurrentSymbol = useAppStore((s) => s.setCurrentSymbol);

  useEffect(() => {
    const sym = searchParams.get("symbol");
    if (sym) setCurrentSymbol(sym.toUpperCase());
  }, [searchParams, setCurrentSymbol]);

  return (
    <div className="flex h-full overflow-hidden bg-canvas-soft">
      <div className="w-[220px] shrink-0 border-r border-border">
        <SymbolList />
      </div>
      <div className="min-w-0 flex-1 p-3">
        <ChartPanel />
      </div>
      <div className="w-[360px] shrink-0 overflow-y-auto border-l border-border bg-background">
        <div className="flex flex-col gap-3 p-3">
          <AiPanel />
          <CalendarPanel />
          <NewsPanel />
        </div>
      </div>
    </div>
  );
}
