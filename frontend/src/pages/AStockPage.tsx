import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AStockDashboard } from "@/features/stocks/AStockDashboard";
import { StockIndustryView } from "@/features/stocks/StockIndustryView";
import { StockDetailWorkspace } from "@/features/stocks/StockDetailWorkspace";
import { StockScreener } from "@/features/stocks/StockScreener";
import { StockPaperPortfolio } from "@/features/stocks/StockPaperPortfolio";
import { StockFinancialCenter } from "@/features/stocks/StockFinancialCenter";
import { StockWatchlistView } from "@/features/stocks/StockWatchlistView";
import type { StockBoardView, StockSymbolSnapshot } from "@/types";

type TabValue = "overview" | "industry" | "stock" | "screener" | "financials" | "portfolio" | "watchlist";

export function AStockPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const tab = (searchParams.get("tab") as TabValue) || "overview";
  const selectedBoard = searchParams.get("board") || "";
  const selectedStock = searchParams.get("symbol") || "";
  const [syncStatus, setSyncStatus] = useState<{ status: string; message?: string }>({
    status: "idle",
    message: "每日 15:35 自动同步 A 股数据",
  });

  useEffect(() => {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;
    let unlistenProgress: (() => void) | undefined;
    let unlistenDone: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<{ status: string; message?: string }>("stock-sync-progress", (e) => {
        setSyncStatus({ status: e.payload.status, message: e.payload.message });
      }).then((fn) => (unlistenProgress = fn));
      listen<{ status: string; total?: number }>("stock-sync-done", (e) => {
        setSyncStatus({
          status: e.payload.status,
          message: `同步完成，共 ${e.payload.total ?? 0} 条记录`,
        });
      }).then((fn) => (unlistenDone = fn));
    });
    return () => {
      unlistenProgress?.();
      unlistenDone?.();
    };
  }, []);

  const setTab = (value: TabValue, params?: Record<string, string>) => {
    const next = new URLSearchParams();
    next.set("tab", value);
    if (params) {
      Object.entries(params).forEach(([k, v]) => {
        if (v) next.set(k, v);
      });
    }
    setSearchParams(next, { replace: true });
  };

  const handleSelectBoard = (board: StockBoardView) => {
    setTab("industry", { board: board.board_code });
  };

  const handleBackToOverview = () => {
    setTab("overview");
  };

  const handleSelectStock = (row: StockSymbolSnapshot) => {
    setTab("stock", { symbol: row.ts_code });
  };

  const handleSelectStockByCode = (tsCode: string) => {
    setTab("stock", { symbol: tsCode });
  };

  const handleBackFromStock = () => {
    setTab("screener");
  };

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border bg-background px-4 py-2">
        {syncStatus.status !== "idle" && (
          <div
            className={`mb-2 rounded-md px-2 py-1 text-[10px] ${
              syncStatus.status === "running"
                ? "bg-amber-500/10 text-amber-500"
                : syncStatus.status === "done"
                  ? "bg-emerald-500/10 text-emerald-500"
                  : "bg-muted text-muted-foreground"
            }`}
          >
            {syncStatus.status === "running" ? "同步中" : syncStatus.status === "done" ? "同步完成" : "同步状态"}：
            {syncStatus.message}
          </div>
        )}
        <Tabs value={tab} onValueChange={(v) => setTab(v as TabValue)}>
          <TabsList className="h-8">
            <TabsTrigger value="overview" className="text-xs">
              总览
            </TabsTrigger>
            <TabsTrigger value="industry" className="text-xs">
              行业/概念
            </TabsTrigger>
            <TabsTrigger value="stock" className="text-xs">
              个股
            </TabsTrigger>
            <TabsTrigger value="screener" className="text-xs">
              筛选器
            </TabsTrigger>
            <TabsTrigger value="financials" className="text-xs">
              财报
            </TabsTrigger>
            <TabsTrigger value="portfolio" className="text-xs">
              组合
            </TabsTrigger>
            <TabsTrigger value="watchlist" className="text-xs">
              自选
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      <div className="min-h-0 flex-1">
        {tab === "overview" && <AStockDashboard onSelectBoard={handleSelectBoard} />}
        {tab === "industry" && selectedBoard && (
          <StockIndustryView boardCode={selectedBoard} onBack={handleBackToOverview} />
        )}
        {tab === "industry" && !selectedBoard && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            请在总览页选择板块
          </div>
        )}
        {tab === "stock" && selectedStock && (
          <StockDetailWorkspace tsCode={selectedStock} onBack={handleBackFromStock} />
        )}
        {tab === "stock" && !selectedStock && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            请在筛选器中选择个股
          </div>
        )}
        {tab === "screener" && <StockScreener onSelectStock={handleSelectStock} />}
        {tab === "financials" && selectedStock && (
          <StockFinancialCenter tsCode={selectedStock} />
        )}
        {tab === "financials" && !selectedStock && (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            请在个股或筛选器中选择股票
          </div>
        )}
        {tab === "portfolio" && <StockPaperPortfolio />}
        {tab === "watchlist" && <StockWatchlistView onSelectStock={handleSelectStockByCode} />}
      </div>
    </div>
  );
}
