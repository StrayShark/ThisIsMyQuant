import { useEffect, useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { PaperPortfolioSummary } from "./components/PaperPortfolioSummary";
import { StockOrderTicket } from "./components/StockOrderTicket";
import { Sparkles } from "lucide-react";
import type { StockPaperAccount } from "@/types";

export function StockPaperPortfolio() {
  const queryClient = useQueryClient();
  const [newAccountName, setNewAccountName] = useState("");
  const [newAccountBalance, setNewAccountBalance] = useState("1000000");
  const [selectedAccountId, setSelectedAccountId] = useState<string | null>(null);
  const [review, setReview] = useState<string | null>(null);

  const accountsQuery = useQuery({
    queryKey: ["stock-paper-accounts"],
    queryFn: () => api.listStockPaperAccounts(),
  });

  const portfolioQuery = useQuery({
    queryKey: ["stock-paper-portfolio", selectedAccountId],
    queryFn: () => api.getStockPaperPortfolio(selectedAccountId!),
    enabled: !!selectedAccountId,
  });

  useEffect(() => {
    if (!selectedAccountId && accountsQuery.data && accountsQuery.data.length > 0) {
      setSelectedAccountId(accountsQuery.data[0].id);
    }
  }, [selectedAccountId, accountsQuery.data]);

  const createMutation = useMutation({
    mutationFn: () =>
      api.createStockPaperAccount({
        name: newAccountName || "A股模拟组合",
        initial_balance: parseFloat(newAccountBalance) || 1_000_000,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stock-paper-accounts"] });
      setNewAccountName("");
    },
  });

  const activeAccount: StockPaperAccount | undefined =
    selectedAccountId && portfolioQuery.data
      ? portfolioQuery.data.account
      : accountsQuery.data?.[0];

  const reviewMutation = useMutation({
    mutationFn: () => api.generateStockPortfolioReview(activeAccount!.id),
    onSuccess: (report) => setReview(report.content),
  });

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
        <h3 className="mb-3 text-sm font-medium">账户管理</h3>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end">
          <div className="flex-1">
            <label className="text-xs text-muted-foreground">账户名称</label>
            <Input
              value={newAccountName}
              onChange={(e) => setNewAccountName(e.target.value)}
              placeholder="A股模拟组合"
              className="mt-1 h-8 text-xs"
            />
          </div>
          <div className="w-32">
            <label className="text-xs text-muted-foreground">初始资金</label>
            <Input
              type="number"
              value={newAccountBalance}
              onChange={(e) => setNewAccountBalance(e.target.value)}
              className="mt-1 h-8 text-xs"
            />
          </div>
          <Button size="sm" onClick={() => createMutation.mutate()} disabled={createMutation.isPending}>
            {createMutation.isPending ? "创建中…" : "创建账户"}
          </Button>
        </div>

        {accountsQuery.data && accountsQuery.data.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-2">
            {accountsQuery.data.map((acc) => (
              <button
                key={acc.id}
                type="button"
                onClick={() => setSelectedAccountId(acc.id)}
                className={`rounded-md px-2 py-1 text-xs ${
                  activeAccount?.id === acc.id
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted text-muted-foreground hover:bg-accent"
                }`}
              >
                {acc.name}
              </button>
            ))}
          </div>
        )}
      </div>

      {activeAccount && portfolioQuery.data && (
        <>
          <div className="flex justify-end">
            <Button
              size="sm"
              variant="outline"
              className="gap-1 text-xs"
              onClick={() => reviewMutation.mutate()}
              disabled={reviewMutation.isPending}
            >
              <Sparkles className="h-3.5 w-3.5" />
              {reviewMutation.isPending ? "生成中…" : "AI 组合复盘"}
            </Button>
          </div>
          <PaperPortfolioSummary account={activeAccount} />

          {review && (
            <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
              <div className="mb-2 flex items-center justify-between">
                <h3 className="text-sm font-medium">AI 组合复盘</h3>
                <button
                  type="button"
                  onClick={() => setReview(null)}
                  className="text-xs text-muted-foreground hover:text-foreground"
                >
                  收起
                </button>
              </div>
              <ScrollArea className="h-48 w-full rounded-md bg-muted/40 p-3 text-xs leading-relaxed whitespace-pre-wrap">
                {review}
              </ScrollArea>
            </div>
          )}

          <div className="grid grid-cols-1 gap-4 lg:grid-cols-12">
            <div className="lg:col-span-4">
              <StockOrderTicket
                accountId={activeAccount.id}
                onSuccess={() => queryClient.invalidateQueries({ queryKey: ["stock-paper-portfolio", activeAccount.id] })}
              />
            </div>

            <div className="flex flex-col gap-4 lg:col-span-8">
              <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
                <h3 className="mb-3 text-sm font-medium">持仓</h3>
                {portfolioQuery.data.positions.length === 0 ? (
                  <div className="text-xs text-muted-foreground">暂无持仓</div>
                ) : (
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="border-b border-border text-muted-foreground">
                        <th className="py-2 text-left">代码</th>
                        <th className="py-2 text-left">名称</th>
                        <th className="py-2 text-right">持仓量</th>
                        <th className="py-2 text-right">可卖</th>
                        <th className="py-2 text-right">成本</th>
                        <th className="py-2 text-right">市值</th>
                        <th className="py-2 text-right">盈亏</th>
                      </tr>
                    </thead>
                    <tbody>
                      {portfolioQuery.data.positions.map((pos) => (
                        <tr key={pos.ts_code} className="border-b border-border/50">
                          <td className="py-2 tabular-nums">{pos.ts_code}</td>
                          <td className="py-2">{pos.name}</td>
                          <td className="py-2 text-right tabular-nums">{pos.quantity}</td>
                          <td className="py-2 text-right tabular-nums">{pos.available_quantity}</td>
                          <td className="py-2 text-right tabular-nums">{pos.avg_cost.toFixed(2)}</td>
                          <td className="py-2 text-right tabular-nums">{pos.market_value.toFixed(2)}</td>
                          <td
                            className={`py-2 text-right tabular-nums ${
                              pos.unrealized_pnl >= 0 ? "text-emerald-500" : "text-rose-500"
                            }`}
                          >
                            {pos.unrealized_pnl >= 0 ? "+" : ""}
                            {pos.unrealized_pnl.toFixed(2)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>

              <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
                <h3 className="mb-3 text-sm font-medium">近期成交</h3>
                {portfolioQuery.data.trades.length === 0 ? (
                  <div className="text-xs text-muted-foreground">暂无成交</div>
                ) : (
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="border-b border-border text-muted-foreground">
                        <th className="py-2 text-left">时间</th>
                        <th className="py-2 text-left">代码</th>
                        <th className="py-2 text-left">方向</th>
                        <th className="py-2 text-right">价格</th>
                        <th className="py-2 text-right">数量</th>
                        <th className="py-2 text-right">费用</th>
                      </tr>
                    </thead>
                    <tbody>
                      {portfolioQuery.data.trades.slice(0, 10).map((trade) => (
                        <tr key={trade.id} className="border-b border-border/50">
                          <td className="py-2 tabular-nums">{new Date(trade.traded_at).toLocaleString()}</td>
                          <td className="py-2 tabular-nums">{trade.ts_code}</td>
                          <td className={`py-2 ${trade.side === "buy" ? "text-emerald-500" : "text-rose-500"}`}>
                            {trade.side === "buy" ? "买入" : "卖出"}
                          </td>
                          <td className="py-2 text-right tabular-nums">{trade.price.toFixed(2)}</td>
                          <td className="py-2 text-right tabular-nums">{trade.quantity}</td>
                          <td className="py-2 text-right tabular-nums">{trade.commission.toFixed(2)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </div>
          </div>
        </>
      )}

      {!activeAccount && !accountsQuery.isLoading && (
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
          请创建 A 股模拟组合账户
        </div>
      )}
    </div>
  );
}
