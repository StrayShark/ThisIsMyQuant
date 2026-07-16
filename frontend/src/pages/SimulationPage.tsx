import { useEffect, useMemo, useState } from "react";
import type React from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { PageShell } from "@/components/layout/PageShell";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { AiSummaryModal } from "@/features/ai/AiSummaryModal";
import { useAiSummary } from "@/features/ai/useAiSummary";
import { Sparkles } from "lucide-react";
import type {
  PlaceSimOrderRequest,
  AnalysisReport,
  SimAccount,
  SimEquitySnapshot,
  SimOrder,
  SimOrderEstimate,
  SimPerformance,
  SimPosition,
  SimTrade,
} from "@/types";

const SIDE_OPTIONS = [
  { value: "buy", label: "买入" },
  { value: "sell", label: "卖出" },
] as const;

const OFFSET_OPTIONS = [
  { value: "open", label: "开仓" },
  { value: "close", label: "平仓" },
] as const;

const TYPE_OPTIONS = [
  { value: "limit", label: "限价" },
  { value: "market", label: "市价" },
] as const;

const SIM_QUERY_KEYS = [
  ["sim-accounts"],
  ["sim-snapshot"],
  ["sim-positions"],
  ["sim-orders"],
  ["sim-trades"],
  ["sim-equity-curve"],
  ["sim-performance"],
] as const;

export function SimulationPage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const aiSummary = useAiSummary();

  const { data: accounts = [] } = useQuery({
    queryKey: ["sim-accounts"],
    queryFn: () => api.listSimAccounts(),
  });

  const [selectedAccountId, setSelectedAccountId] = useState<string | undefined>();
  const accountId = selectedAccountId ?? accounts[0]?.id;

  const { data: snapshot, isLoading: snapshotLoading } = useQuery({
    queryKey: ["sim-snapshot", accountId],
    queryFn: () => api.getSimAccountSnapshot(accountId),
    enabled: !!accountId,
  });

  const { data: positions = [] } = useQuery({
    queryKey: ["sim-positions", accountId],
    queryFn: () => api.listSimPositions(accountId),
    enabled: !!accountId,
  });

  const { data: orders = [] } = useQuery({
    queryKey: ["sim-orders", accountId],
    queryFn: () => api.listSimOrders({ account_id: accountId }),
    enabled: !!accountId,
  });

  const { data: trades = [] } = useQuery({
    queryKey: ["sim-trades", accountId],
    queryFn: () => api.listSimTrades({ account_id: accountId }),
    enabled: !!accountId,
  });

  const { data: equityCurve = [] } = useQuery({
    queryKey: ["sim-equity-curve", accountId],
    queryFn: () => api.listSimEquityCurve({ account_id: accountId }),
    enabled: !!accountId,
  });

  const { data: performance } = useQuery({
    queryKey: ["sim-performance", accountId],
    queryFn: () => api.getSimPerformance({ account_id: accountId }),
    enabled: !!accountId,
  });

  const [form, setForm] = useState<Pick<
    PlaceSimOrderRequest,
    "symbol" | "side" | "offset" | "order_type" | "price" | "quantity"
  >>({
    symbol: "RB0",
    side: "buy",
    offset: "open",
    order_type: "limit",
    price: 3200,
    quantity: 1,
  });

  const [estimate, setEstimate] = useState<SimOrderEstimate | null>(null);
  const [reviewReport, setReviewReport] = useState<AnalysisReport | null>(null);
  const symbol = form.symbol || "RB0";

  const { data: quotes } = useQuery({
    queryKey: ["realtime-quotes", symbol],
    queryFn: () => api.getRealtimeQuotes([symbol]),
    enabled: !!symbol,
    refetchInterval: 10_000,
  });

  const quote = useMemo(
    () => quotes?.find((item) => item.symbol.toLowerCase() === symbol.toLowerCase()) ?? null,
    [quotes, symbol]
  );

  const estimateMutation = useMutation({
    mutationFn: (payload: PlaceSimOrderRequest) => api.estimateSimOrder(payload),
    onSuccess: setEstimate,
    onError: () => setEstimate(null),
  });

  const placeOrder = useMutation({
    mutationFn: (payload: PlaceSimOrderRequest) => api.placeSimOrder(payload),
    onSuccess: () => {
      showToast("模拟委托已提交");
      setEstimate(null);
      invalidateSimQueries(queryClient);
    },
    onError: (err: Error) => showToast(err.message),
  });

  const cancelOrder = useMutation({
    mutationFn: (orderId: string) => api.cancelSimOrder(orderId),
    onSuccess: () => {
      showToast("撤单成功");
      void queryClient.invalidateQueries({ queryKey: ["sim-orders"] });
    },
    onError: (err: Error) => showToast(err.message),
  });

  const resetAccount = useMutation({
    mutationFn: (targetAccountId: string) => api.resetSimAccount(targetAccountId),
    onSuccess: () => {
      showToast("账户已重置");
      invalidateSimQueries(queryClient);
    },
    onError: (err: Error) => showToast(err.message),
  });

  const generateReview = useMutation({
    mutationFn: () => api.generateTradeReview({ account_id: accountId, days: 30 }),
    onSuccess: (report) => {
      setReviewReport(report);
      showToast("AI 复盘摘要已生成");
    },
    onError: (err: Error) => showToast(err.message || "复盘摘要生成失败"),
  });

  useEffect(() => {
    if (!accountId) return;
    const payload = buildOrderPayload(accountId, form);
    const timer = window.setTimeout(() => {
      estimateMutation.mutate(payload);
    }, 300);
    return () => window.clearTimeout(timer);
  }, [accountId, form, estimateMutation]);

  const submit = (event: React.FormEvent) => {
    event.preventDefault();
    if (!accountId) return;
    placeOrder.mutate(buildOrderPayload(accountId, form));
  };

  const handleReset = () => {
    if (!accountId) return;
    const confirmed = window.confirm(
      "确定要重置该模拟账户吗？此操作将清空持仓、委托和成交记录，资金恢复到初始值，且不可撤销。"
    );
    if (confirmed) {
      resetAccount.mutate(accountId);
    }
  };

  const openOrders = orders.filter((order) => ["pending", "open", "partially_filled"].includes(order.status));

  return (
    <PageShell>
      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="mb-2 flex items-center gap-2">
            <h1 className="text-2xl font-normal tracking-tight">模拟盘</h1>
            <Badge variant="outline">仅模拟</Badge>
          </div>
          <p className="text-sm text-muted-foreground">
            基础虚拟账户、基础下单、持仓和成交记录。不连接实盘柜台。
          </p>
        </div>
        <div className="flex items-center gap-2">
          {accounts.length > 0 && (
            <NativeSelect
              value={accountId}
              onChange={(event) => setSelectedAccountId(event.target.value)}
              className="w-[220px]"
            >
              {accounts.map((account) => (
                <option key={account.id} value={account.id}>
                  {account.name}
                </option>
              ))}
            </NativeSelect>
          )}
          <Button
            variant="outline"
            onClick={handleReset}
            disabled={!accountId || resetAccount.isPending}
          >
            重置账户
          </Button>
        </div>
      </div>

      {snapshotLoading || !snapshot ? (
        <Card>
          <CardContent className="py-10 text-sm text-muted-foreground">加载模拟账户...</CardContent>
        </Card>
      ) : (
        <div className="space-y-5">
          <section>
            <div className="grid gap-4 md:grid-cols-4">
              <MetricCard label="账户权益" value={formatMoney(snapshot.account.equity)} />
              <MetricCard label="可用资金" value={formatMoney(snapshot.account.cash_balance)} />
              <MetricCard label="保证金占用" value={formatMoney(snapshot.account.margin_used)} />
              <MetricCard label="风险度" value={`${(snapshot.risk_ratio * 100).toFixed(1)}%`} />
            </div>
            <PerformanceSummary performance={performance ?? null} />
            <TradeReviewSummary
              report={reviewReport}
              isLoading={generateReview.isPending}
              disabled={!accountId}
              onGenerate={() => generateReview.mutate()}
            />
            <div className="mt-2 flex flex-wrap items-center justify-between gap-2">
              <p className="text-xs text-muted-foreground">
                账户数据均为模拟，初始资金 {formatMoney(snapshot.account.initial_balance)}，不构成投资建议。
              </p>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 gap-1.5 rounded-full text-xs"
                onClick={() =>
                  aiSummary.generate({
                    task_type: "position_risk",
                    target_symbol: accountId,
                  })
                }
              >
                <Sparkles className="h-3.5 w-3.5" />
                AI 持仓风险
              </Button>
            </div>
          </section>

          <section className="grid gap-5 xl:grid-cols-[380px_minmax(0,1fr)]">
            <Card>
              <CardHeader>
                <CardTitle>基础下单</CardTitle>
              </CardHeader>
              <CardContent>
                <form onSubmit={submit} className="space-y-4">
                  <div>
                    <label className="mb-1.5 block text-xs font-medium text-muted-foreground">
                      品种
                    </label>
                    <Input
                      value={form.symbol}
                      onChange={(event) =>
                        setForm({ ...form, symbol: event.target.value.toUpperCase() })
                      }
                      className="font-mono"
                    />
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <FieldSelect
                      label="方向"
                      value={form.side}
                      options={SIDE_OPTIONS}
                      onChange={(value) =>
                        setForm({ ...form, side: value as PlaceSimOrderRequest["side"] })
                      }
                    />
                    <FieldSelect
                      label="开平"
                      value={form.offset}
                      options={OFFSET_OPTIONS}
                      onChange={(value) =>
                        setForm({ ...form, offset: value as PlaceSimOrderRequest["offset"] })
                      }
                    />
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <FieldSelect
                      label="类型"
                      value={form.order_type}
                      options={TYPE_OPTIONS}
                      onChange={(value) =>
                        setForm({
                          ...form,
                          order_type: value as PlaceSimOrderRequest["order_type"],
                        })
                      }
                    />
                    <div>
                      <label className="mb-1.5 block text-xs font-medium text-muted-foreground">
                        手数
                      </label>
                      <Input
                        type="number"
                        min={1}
                        value={form.quantity}
                        onChange={(event) =>
                          setForm({ ...form, quantity: Number(event.target.value) })
                        }
                      />
                    </div>
                  </div>

                  {form.order_type === "limit" && (
                    <div>
                      <label className="mb-1.5 block text-xs font-medium text-muted-foreground">
                        限价
                      </label>
                      <Input
                        type="number"
                        value={form.price ?? ""}
                        onChange={(event) =>
                          setForm({ ...form, price: Number(event.target.value) })
                        }
                      />
                    </div>
                  )}

                  <div className="rounded-2xl bg-[var(--color-surface-elevated)] p-4 text-sm">
                    <div className="mb-3 flex items-center justify-between">
                      <span className="text-muted-foreground">参考行情</span>
                      <span className="font-mono text-foreground">
                        {quote ? quote.last_price.toFixed(2) : "--"}
                      </span>
                    </div>
                    <CostRow label="预估保证金" value={formatMoney(estimate?.margin_required ?? 0)} />
                    <CostRow label="预估手续费" value={formatMoney(estimate?.commission_estimate ?? 0)} />
                    <CostRow label="预估总成本" value={formatMoney(estimate?.total_cost ?? 0)} strong />
                  </div>

                  <Button type="submit" className="h-11 w-full" disabled={placeOrder.isPending}>
                    {placeOrder.isPending ? "提交中..." : "提交模拟委托"}
                  </Button>
                  <p className="text-xs text-muted-foreground">
                    仅支持市价/限价与开仓/平仓。
                  </p>
                </form>
              </CardContent>
            </Card>

            <div className="space-y-5">
              <Card>
                <CardHeader className="flex-row items-center justify-between">
                  <CardTitle>持仓</CardTitle>
                  <Badge variant="secondary">{positions.length} 个品种</Badge>
                </CardHeader>
                <CardContent>
                  <PositionTable positions={positions} />
                  <SimulationNotice />
                </CardContent>
              </Card>

              <div className="grid gap-5 lg:grid-cols-2">
                <Card>
                  <CardHeader className="flex-row items-center justify-between">
                    <CardTitle>委托</CardTitle>
                    <Badge variant="secondary">{openOrders.length} 笔未完成</Badge>
                  </CardHeader>
                  <CardContent>
                    <OrderTable orders={orders} onCancel={(id) => cancelOrder.mutate(id)} />
                    <SimulationNotice />
                  </CardContent>
                </Card>
                <Card>
                  <CardHeader>
                    <CardTitle>成交</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <TradeTable trades={trades} />
                    <SimulationNotice />
                  </CardContent>
                </Card>
              </div>

              <Card>
                <CardHeader>
                  <CardTitle>资金流水</CardTitle>
                </CardHeader>
                <CardContent>
                  <EquityFlowTable snapshots={equityCurve} account={snapshot.account} />
                  <SimulationNotice />
                </CardContent>
              </Card>
            </div>
          </section>
        </div>
      )}

      <footer className="mt-8 border-t pt-4 text-center text-xs text-muted-foreground">
        本页面所有交易均为模拟，不构成投资建议，不连接实盘柜台。
      </footer>

      <AiSummaryModal
        isOpen={aiSummary.isOpen}
        onClose={aiSummary.close}
        title="AI 持仓风险"
        report={aiSummary.report}
        isLoading={aiSummary.isLoading}
        error={aiSummary.error}
      />
    </PageShell>
  );
}

function invalidateSimQueries(queryClient: ReturnType<typeof useQueryClient>) {
  for (const key of SIM_QUERY_KEYS) {
    void queryClient.invalidateQueries({ queryKey: key });
  }
}

function buildOrderPayload(
  accountId: string,
  form: Pick<
    PlaceSimOrderRequest,
    "symbol" | "side" | "offset" | "order_type" | "price" | "quantity"
  >
): PlaceSimOrderRequest {
  return {
    account_id: accountId,
    symbol: form.symbol || "RB0",
    side: form.side || "buy",
    offset: form.offset || "open",
    order_type: form.order_type || "limit",
    price: form.order_type === "market" ? null : form.price ?? 0,
    quantity: form.quantity || 1,
    tif: "DAY",
  };
}

function FieldSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value?: string | null;
  options: readonly { value: string; label: string }[];
  onChange: (value: string) => void;
}) {
  return (
    <div>
      <label className="mb-1.5 block text-xs font-medium text-muted-foreground">{label}</label>
      <NativeSelect value={value ?? ""} onChange={(event) => onChange(event.target.value)}>
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </NativeSelect>
    </div>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <Card>
      <CardContent className="p-5">
        <div className="text-xs font-medium text-muted-foreground">{label}</div>
        <div className="mt-2 font-mono text-xl font-medium text-foreground">{value}</div>
      </CardContent>
    </Card>
  );
}

function PerformanceSummary({ performance }: { performance: SimPerformance | null }) {
  if (!performance) return null;
  const topContribution = Object.entries(performance.symbol_contribution ?? {})
    .sort((a, b) => Math.abs(b[1]) - Math.abs(a[1]))
    .slice(0, 3);
  return (
    <div className="mt-4 grid gap-3 md:grid-cols-5">
      <MetricCard label="累计收益率" value={formatRatioPercent(performance.total_return_pct)} />
      <MetricCard label="最大回撤" value={formatRatioPercent(performance.max_drawdown_pct)} />
      <MetricCard label="胜率" value={formatRatioPercent(performance.win_rate)} />
      <MetricCard label="盈亏比" value={performance.profit_loss_ratio.toFixed(2)} />
      <Card>
        <CardContent className="p-5">
          <div className="text-xs font-medium text-muted-foreground">品种贡献</div>
          <div className="mt-2 space-y-1 text-xs">
            {topContribution.length === 0 ? (
              <span className="text-muted-foreground">暂无成交</span>
            ) : (
              topContribution.map(([symbol, pnl]) => (
                <div key={symbol} className="flex items-center justify-between gap-2">
                  <span className="font-mono text-foreground">{symbol}</span>
                  <span className={pnl >= 0 ? "text-[var(--color-up)]" : "text-[var(--color-down)]"}>
                    {formatMoney(pnl)}
                  </span>
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function TradeReviewSummary({
  report,
  isLoading,
  disabled,
  onGenerate,
}: {
  report: AnalysisReport | null;
  isLoading: boolean;
  disabled: boolean;
  onGenerate: () => void;
}) {
  return (
    <Card className="mt-4">
      <CardContent className="p-5">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <div className="text-sm font-medium text-foreground">AI 复盘摘要</div>
            <p className="mt-1 text-xs text-muted-foreground">
              基于最近 30 天成交、持仓、资金曲线和复盘日记，生成执行纪律与风险改进建议。
            </p>
          </div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="h-8 gap-1.5 rounded-full text-xs"
            onClick={onGenerate}
            disabled={disabled || isLoading}
          >
            <Sparkles className="h-3.5 w-3.5" />
            {isLoading ? "生成中..." : "生成复盘摘要"}
          </Button>
        </div>
        {report ? (
          <div className="mt-4 rounded-xl border border-border bg-muted/30 p-4">
            <div className="whitespace-pre-wrap text-sm leading-relaxed text-foreground">
              {report.content}
            </div>
            <p className="mt-3 text-xs text-muted-foreground">
              生成时间：{new Date(report.created_at).toLocaleString("zh-CN")} · 模型：{report.provider}
            </p>
          </div>
        ) : (
          <div className="mt-4 rounded-xl border border-dashed border-border p-4 text-xs text-muted-foreground">
            暂无复盘摘要。完成若干模拟成交后，可生成本周期的纪律、风险和改进动作。
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function CostRow({ label, value, strong = false }: { label: string; value: string; strong?: boolean }) {
  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-muted-foreground">{label}</span>
      <span className={strong ? "font-mono font-semibold text-foreground" : "font-mono text-foreground"}>
        {value}
      </span>
    </div>
  );
}

function SimulationNotice() {
  return (
    <p className="mt-3 text-xs text-muted-foreground">
      数据仅为模拟，不构成投资建议，不连接实盘柜台。
    </p>
  );
}

function PositionTable({ positions }: { positions: SimPosition[] }) {
  if (positions.length === 0) return <Empty text="当前没有持仓" />;
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[620px] text-sm">
        <thead>
          <tr className="border-b text-xs text-muted-foreground">
            <th className="py-2 text-left font-medium">品种</th>
            <th className="py-2 text-left font-medium">方向</th>
            <th className="py-2 text-right font-medium">手数</th>
            <th className="py-2 text-right font-medium">均价</th>
            <th className="py-2 text-right font-medium">浮盈</th>
            <th className="py-2 text-right font-medium">保证金</th>
          </tr>
        </thead>
        <tbody>
          {positions.map((position) => (
            <tr key={`${position.account_id}-${position.symbol}-${position.position_side}`} className="border-b last:border-0">
              <td className="py-3">{position.name}</td>
              <td className="py-3">{position.position_side === "long" ? "多头" : "空头"}</td>
              <td className="py-3 text-right font-mono">{position.total_qty}</td>
              <td className="py-3 text-right font-mono">{position.avg_price.toFixed(2)}</td>
              <td className={`py-3 text-right font-mono ${position.unrealized_pnl >= 0 ? "text-[var(--color-up)]" : "text-[var(--color-down)]"}`}>
                {formatMoney(position.unrealized_pnl)}
              </td>
              <td className="py-3 text-right font-mono">{formatMoney(position.margin)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function OrderTable({ orders, onCancel }: { orders: SimOrder[]; onCancel: (id: string) => void }) {
  if (orders.length === 0) return <Empty text="当前没有委托" />;
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[560px] text-sm">
        <thead>
          <tr className="border-b text-xs text-muted-foreground">
            <th className="py-2 text-left font-medium">时间</th>
            <th className="py-2 text-left font-medium">品种</th>
            <th className="py-2 text-left font-medium">方向</th>
            <th className="py-2 text-right font-medium">价格</th>
            <th className="py-2 text-right font-medium">数量</th>
            <th className="py-2 text-left font-medium">状态</th>
            <th className="py-2 text-right font-medium">操作</th>
          </tr>
        </thead>
        <tbody>
          {orders.slice(0, 8).map((order) => (
            <tr key={order.id} className="border-b last:border-0">
              <td className="py-3 text-xs text-muted-foreground">{new Date(order.created_at).toLocaleTimeString()}</td>
              <td className="py-3">{order.name}</td>
              <td className="py-3">{orderDirLabel(order)}</td>
              <td className="py-3 text-right font-mono">{orderPriceLabel(order)}</td>
              <td className="py-3 text-right font-mono">
                {order.filled_quantity}/{order.quantity}
              </td>
              <td className="py-3">{orderStatusLabel(order.status)}</td>
              <td className="py-3 text-right">
                {order.status === "open" && (
                  <Button size="sm" variant="secondary" onClick={() => onCancel(order.id)}>
                    撤单
                  </Button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function TradeTable({ trades }: { trades: SimTrade[] }) {
  if (trades.length === 0) return <Empty text="当前没有成交" />;
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[460px] text-sm">
        <thead>
          <tr className="border-b text-xs text-muted-foreground">
            <th className="py-2 text-left font-medium">时间</th>
            <th className="py-2 text-left font-medium">品种</th>
            <th className="py-2 text-left font-medium">方向</th>
            <th className="py-2 text-right font-medium">价格</th>
            <th className="py-2 text-right font-medium">手数</th>
          </tr>
        </thead>
        <tbody>
          {trades.slice(0, 8).map((trade) => (
            <tr key={trade.id} className="border-b last:border-0">
              <td className="py-3 text-xs text-muted-foreground">{new Date(trade.traded_at).toLocaleTimeString()}</td>
              <td className="py-3">{trade.name}</td>
              <td className="py-3">{tradeDirLabel(trade)}</td>
              <td className="py-3 text-right font-mono">{trade.price.toFixed(2)}</td>
              <td className="py-3 text-right font-mono">{trade.quantity}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface EquityFlowRow {
  time: string;
  type: string;
  amount: number;
  balance: number;
}

function EquityFlowTable({ snapshots, account }: { snapshots: SimEquitySnapshot[]; account: SimAccount }) {
  const rows = useMemo(() => buildEquityFlow(snapshots, account), [snapshots, account]);
  if (rows.length === 0) return <Empty text="暂无资金流水" />;
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[480px] text-sm">
        <thead>
          <tr className="border-b text-xs text-muted-foreground">
            <th className="py-2 text-left font-medium">时间</th>
            <th className="py-2 text-left font-medium">类型</th>
            <th className="py-2 text-right font-medium">金额</th>
            <th className="py-2 text-right font-medium">余额</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={`${row.time}-${index}`} className="border-b last:border-0">
              <td className="py-3 text-xs text-muted-foreground">{new Date(row.time).toLocaleString()}</td>
              <td className="py-3">{row.type}</td>
              <td className={`py-3 text-right font-mono ${row.amount >= 0 ? "text-[var(--color-up)]" : "text-[var(--color-down)]"}`}>
                {row.amount >= 0 ? `+${formatMoney(row.amount)}` : formatMoney(row.amount)}
              </td>
              <td className="py-3 text-right font-mono">{formatMoney(row.balance)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function buildEquityFlow(snapshots: SimEquitySnapshot[], account: SimAccount): EquityFlowRow[] {
  const sorted = [...snapshots].sort(
    (a, b) => new Date(a.snapshot_at).getTime() - new Date(b.snapshot_at).getTime()
  );
  return sorted.map((snapshot, index) => {
    const previous = index > 0 ? sorted[index - 1].equity : account.initial_balance;
    return {
      time: snapshot.snapshot_at,
      type: "权益快照",
      amount: snapshot.equity - previous,
      balance: snapshot.equity,
    };
  });
}

function Empty({ text }: { text: string }) {
  return (
    <div className="rounded-2xl border border-dashed border-border py-8 text-center text-sm text-muted-foreground">
      {text}
    </div>
  );
}

function formatMoney(value: number) {
  return `¥${value.toLocaleString("zh-CN", { maximumFractionDigits: 2 })}`;
}

function formatRatioPercent(value: number) {
  return `${(value * 100).toFixed(2)}%`;
}

function orderDirLabel(order: SimOrder) {
  const side = order.side === "buy" ? "买" : "卖";
  return `${side}${order.offset === "open" ? "开" : "平"}`;
}

function orderPriceLabel(order: SimOrder) {
  if (order.order_type === "market") return "市价";
  return order.price?.toFixed(2) ?? "--";
}

function orderStatusLabel(status: SimOrder["status"]) {
  const map: Record<string, string> = {
    pending: "待报",
    open: "挂单中",
    partially_filled: "部分成交",
    filled: "已成交",
    cancelled: "已撤单",
    rejected: "已拒单",
  };
  return map[status] ?? status;
}

function tradeDirLabel(trade: { side: string; offset: string }) {
  const side = trade.side === "buy" ? "买" : "卖";
  return `${side}${trade.offset === "open" ? "开" : "平"}`;
}

export default SimulationPage;
