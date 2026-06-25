/** 主工作台：品种列表 + K 线 + AI / 资讯面板。 */
import { ChartPanel } from "@/features/chart/ChartPanel";
import { SymbolList } from "@/features/market/SymbolList";
import { AiPanel } from "@/features/analysis/AiPanel";
import { NewsPanel } from "@/features/news/NewsPanel";

export function DashboardPage() {
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
          <NewsPanel />
        </div>
      </div>
    </div>
  );
}
