import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Plus } from "lucide-react";
import { DataQualityBadge } from "./components/DataQualityBadge";
import { FinancialSnapshot } from "./components/FinancialSnapshot";
import { StockKlineChart } from "./components/StockKlineChart";
import { ArrowLeft, TrendingUp, TrendingDown, Sparkles } from "lucide-react";

interface StockDetailWorkspaceProps {
  tsCode: string;
  onBack: () => void;
}

export function StockDetailWorkspace({ tsCode, onBack }: StockDetailWorkspaceProps) {
  const [summary, setSummary] = useState<string | null>(null);
  const [adjustment, setAdjustment] = useState<"none" | "qfq" | "hfq">("qfq");
  const { data, isLoading, error } = useQuery({
    queryKey: ["stock-detail", tsCode],
    queryFn: () => api.getStockDetail(tsCode),
    staleTime: 30_000,
  });

  const summaryMutation = useMutation({
    mutationFn: () => api.generateStockSummary(tsCode),
    onSuccess: (report) => setSummary(report.content),
  });

  const queryClient = useQueryClient();
  const { data: watchlists } = useQuery({
    queryKey: ["stock-watchlists"],
    queryFn: () => api.listStockWatchlists(),
  });
  const addToWatchlistMutation = useMutation({
    mutationFn: async () => {
      const target = watchlists?.[0];
      if (target) {
        if (target.symbols.includes(tsCode)) return target;
        return api.saveStockWatchlist({
          id: target.id,
          name: target.name,
          symbols: [...target.symbols, tsCode],
        });
      }
      return api.saveStockWatchlist({ name: "我的自选股", symbols: [tsCode] });
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["stock-watchlists"] }),
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

  const { symbol, latest_bar, latest_valuation, latest_financial, factor_scores, quality } = data;
  const pct = latest_bar?.pct_chg ?? 0;
  const up = pct >= 0;

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onBack}
          className="inline-flex items-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent"
        >
          <ArrowLeft className="h-3.5 w-3.5" />
          返回
        </button>
        <div>
          <div className="flex items-center gap-2">
            <span className="text-base font-semibold">{symbol.name}</span>
            <span className="text-xs text-muted-foreground">{symbol.ts_code}</span>
          </div>
          <div className="text-xs text-muted-foreground">
            {symbol.exchange} · {symbol.industry ?? "未分类"}
          </div>
        </div>
        <div className="ml-auto text-right">
          <div className={`text-xl font-semibold tabular-nums ${up ? "text-emerald-500" : "text-rose-500"}`}>
            {latest_bar?.close?.toFixed(2) ?? "--"}
          </div>
          <div className={`flex items-center justify-end gap-1 text-xs ${up ? "text-emerald-500" : "text-rose-500"}`}>
            {up ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
            {pct >= 0 ? "+" : ""}
            {pct.toFixed(2)}%
          </div>
        </div>
        <Button
          size="sm"
          variant="outline"
          className="gap-1 text-xs"
          onClick={() => summaryMutation.mutate()}
          disabled={summaryMutation.isPending}
        >
          <Sparkles className="h-3.5 w-3.5" />
          {summaryMutation.isPending ? "生成中…" : "AI 研究速览"}
        </Button>
        <Button
          size="sm"
          variant="outline"
          className="gap-1 text-xs"
          onClick={() => addToWatchlistMutation.mutate()}
          disabled={addToWatchlistMutation.isPending}
          title={watchlists?.some((w) => w.symbols.includes(tsCode)) ? "已在自选" : "加入自选"}
        >
          <Plus className="h-3.5 w-3.5" />
          {watchlists?.some((w) => w.symbols.includes(tsCode)) ? "已在自选" : "加入自选"}
        </Button>
      </div>

      {summary && (
        <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
          <div className="mb-2 flex items-center justify-between">
            <h3 className="text-sm font-medium">AI 研究速览</h3>
            <button
              type="button"
              onClick={() => setSummary(null)}
              className="text-xs text-muted-foreground hover:text-foreground"
            >
              收起
            </button>
          </div>
          <ScrollArea className="h-48 w-full rounded-md bg-muted/40 p-3 text-xs leading-relaxed whitespace-pre-wrap">
            {summary}
          </ScrollArea>
        </div>
      )}

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-12">
        <div className="lg:col-span-7">
          <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
            <div className="mb-3 flex items-center justify-between">
              <h3 className="text-sm font-medium">K 线走势</h3>
              <div className="flex gap-1">
                {(["none", "qfq", "hfq"] as const).map((adj) => (
                  <button
                    key={adj}
                    type="button"
                    onClick={() => setAdjustment(adj)}
                    className={`rounded px-2 py-0.5 text-[10px] ${
                      adjustment === adj
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground hover:bg-accent"
                    }`}
                  >
                    {adj === "none" ? "不复权" : adj === "qfq" ? "前复权" : "后复权"}
                  </button>
                ))}
              </div>
            </div>
            <StockKlineChart tsCode={tsCode} adjustment={adjustment} height={256} />
          </div>
        </div>

        <div className="flex flex-col gap-4 lg:col-span-5">
          <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
            <h3 className="mb-3 text-sm font-medium">因子得分</h3>
            {factor_scores ? (
              <div className="grid grid-cols-2 gap-2 text-xs">
                <div className="rounded-md bg-muted/40 p-2">
                  <span className="text-muted-foreground">动量</span>
                  <div className="font-semibold tabular-nums">{factor_scores.momentum?.toFixed(2) ?? "--"}</div>
                </div>
                <div className="rounded-md bg-muted/40 p-2">
                  <span className="text-muted-foreground">质量</span>
                  <div className="font-semibold tabular-nums">{factor_scores.quality?.toFixed(2) ?? "--"}</div>
                </div>
                <div className="rounded-md bg-muted/40 p-2">
                  <span className="text-muted-foreground">估值</span>
                  <div className="font-semibold tabular-nums">{factor_scores.valuation?.toFixed(2) ?? "--"}</div>
                </div>
                <div className="rounded-md bg-muted/40 p-2">
                  <span className="text-muted-foreground">成长</span>
                  <div className="font-semibold tabular-nums">{factor_scores.growth?.toFixed(2) ?? "--"}</div>
                </div>
                <div className="rounded-md bg-muted/40 p-2">
                  <span className="text-muted-foreground">波动</span>
                  <div className="font-semibold tabular-nums">{factor_scores.volatility?.toFixed(2) ?? "--"}</div>
                </div>
                <div className="rounded-md bg-muted/40 p-2">
                  <span className="text-muted-foreground">流动性</span>
                  <div className="font-semibold tabular-nums">{factor_scores.liquidity?.toFixed(2) ?? "--"}</div>
                </div>
              </div>
            ) : (
              <div className="text-xs text-muted-foreground">暂无因子数据</div>
            )}
          </div>

          <FinancialSnapshot financial={latest_financial} valuation={latest_valuation} />
        </div>
      </div>

      <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
        <div className="mb-2 text-sm font-medium">数据质量</div>
        <DataQualityBadge
          status={quality.status}
          message={quality.message}
          lastSuccessAt={quality.last_success_at}
        />
      </div>
    </div>
  );
}
