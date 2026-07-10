import { useEffect, useMemo, useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/api/client";
import { wsClient } from "@/ws/socket";
import { ensureMarketSubscription } from "@/lib/market-subscribe";
import type {
  SimOrder,
  SimPosition,
  SimTrade,
  PlaceSimOrderRequest,
  SimOrderEstimate,
} from "@/types";
import { useAppStore } from "@/app/store";

const SIDE_OPTIONS = [
  { value: "buy", label: "买入" },
  { value: "sell", label: "卖出" },
];

const OFFSET_OPTIONS = [
  { value: "open", label: "开仓" },
  { value: "close", label: "平仓" },
  { value: "close_today", label: "平今" },
  { value: "close_yesterday", label: "平昨" },
];

const TYPE_OPTIONS = [
  { value: "market", label: "市价" },
  { value: "limit", label: "限价" },
  { value: "stop", label: "止损" },
  { value: "stop_limit", label: "止损限价" },
  { value: "take_profit", label: "止盈" },
  { value: "take_profit_limit", label: "止盈限价" },
  { value: "trailing_stop", label: "移动止损" },
  { value: "condition", label: "条件单" },
];

const CONDITION_OPERATOR_OPTIONS = [
  { value: ">=", label: "≥" },
  { value: "<=", label: "≤" },
];

const TIF_OPTIONS = [
  { value: "GTC", label: "GTC 长期有效" },
  { value: "DAY", label: "当日有效" },
];

export function SimulationPage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);

  const { data: accounts = [] } = useQuery({
    queryKey: ["sim-accounts"],
    queryFn: () => api.listSimAccounts(),
  });

  const [selectedAccountId, setSelectedAccountId] = useState<string | undefined>(undefined);

  const accountId = useMemo(() => {
    if (selectedAccountId) return selectedAccountId;
    return accounts[0]?.id;
  }, [selectedAccountId, accounts]);

  const { data: snapshot, isLoading: snapshotLoading } = useQuery({
    queryKey: ["sim-snapshot", accountId],
    queryFn: () => api.getSimAccountSnapshot(accountId),
    enabled: !!accountId,
  });

  const { data: positions, refetch: refetchPositions } = useQuery({
    queryKey: ["sim-positions", accountId],
    queryFn: () => api.listSimPositions(accountId),
    enabled: !!accountId,
  });

  const { data: orders, refetch: refetchOrders } = useQuery({
    queryKey: ["sim-orders", accountId],
    queryFn: () => api.listSimOrders({ account_id: accountId }),
    enabled: !!accountId,
  });

  const { data: trades, refetch: refetchTrades } = useQuery({
    queryKey: ["sim-trades", accountId],
    queryFn: () => api.listSimTrades({ account_id: accountId }),
    enabled: !!accountId,
  });

  const [form, setForm] = useState<Partial<PlaceSimOrderRequest>>({
    symbol: "RB0",
    side: "buy",
    offset: "open",
    order_type: "limit",
    price: 3200,
    quantity: 1,
    tif: "GTC",
  });

  const [estimate, setEstimate] = useState<SimOrderEstimate | null>(null);

  const symbol = form.symbol ?? "RB0";

  const { data: quotes } = useQuery({
    queryKey: ["realtime-quotes", symbol],
    queryFn: () => api.getRealtimeQuotes([symbol]),
    enabled: !!symbol,
    refetchInterval: 5000,
  });

  const quote = useMemo(() => {
    const q = quotes?.find((x) => x.symbol.toLowerCase() === symbol.toLowerCase());
    if (!q) return null;
    return q;
  }, [quotes, symbol]);

  useEffect(() => {
    if (symbol) {
      void ensureMarketSubscription(symbol);
    }
  }, [symbol]);

  useEffect(() => {
    const off = wsClient.on((msg) => {
      if (msg.type === "quote") return;
      if (msg.type !== "ping" && msg.type !== "pong" && msg.type !== "system") {
        if ("account_id" in msg && (msg.account_id === accountId || !accountId)) {
          void refetchPositions();
          void refetchOrders();
          void refetchTrades();
          void queryClient.invalidateQueries({ queryKey: ["sim-snapshot", accountId] });
        }
      }
    });
    return () => off();
  }, [accountId, queryClient, refetchOrders, refetchPositions, refetchTrades]);

  const placeOrder = useMutation({
    mutationFn: (payload: PlaceSimOrderRequest) => api.placeSimOrder(payload),
    onSuccess: () => {
      showToast("模拟委托已提交");
      void queryClient.invalidateQueries({ queryKey: ["sim-orders"] });
      void queryClient.invalidateQueries({ queryKey: ["sim-snapshot"] });
      void queryClient.invalidateQueries({ queryKey: ["sim-positions"] });
      void queryClient.invalidateQueries({ queryKey: ["sim-trades"] });
      setEstimate(null);
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

  const estimateMutation = useMutation({
    mutationFn: (payload: PlaceSimOrderRequest) => api.estimateSimOrder(payload),
    onSuccess: (data) => setEstimate(data),
    onError: () => setEstimate(null),
  });

  useEffect(() => {
    if (!accountId) return;
    const payload: PlaceSimOrderRequest = {
      account_id: accountId,
      symbol: form.symbol ?? "RB0",
      side: form.side ?? "buy",
      offset: form.offset ?? "open",
      order_type: form.order_type ?? "limit",
      price: form.order_type === "market" ? null : form.price ?? 0,
      trigger_price: form.trigger_price,
      stop_loss_price: form.stop_loss_price,
      take_profit_price: form.take_profit_price,
      oco_group_id: form.oco_group_id,
      parent_order_id: form.parent_order_id,
      tif: form.tif,
      condition_operator: form.condition_operator,
      trailing_distance_ticks: form.trailing_distance_ticks,
      quantity: form.quantity ?? 1,
    };
    const timer = setTimeout(() => {
      void estimateMutation.mutate(payload);
    }, 300);
    return () => clearTimeout(timer);
  }, [form, accountId, estimateMutation]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!accountId) return;
    placeOrder.mutate({
      account_id: accountId,
      symbol: form.symbol ?? "RB0",
      side: form.side ?? "buy",
      offset: form.offset ?? "open",
      order_type: form.order_type ?? "limit",
      price: form.order_type === "market" ? null : form.price ?? 0,
      trigger_price: form.trigger_price,
      stop_loss_price: form.stop_loss_price,
      take_profit_price: form.take_profit_price,
      oco_group_id: form.oco_group_id,
      parent_order_id: form.parent_order_id,
      tif: form.tif,
      condition_operator: form.condition_operator,
      trailing_distance_ticks: form.trailing_distance_ticks,
      quantity: form.quantity ?? 1,
    });
  };

  const needsTriggerPrice = [
    "stop",
    "stop_limit",
    "take_profit",
    "take_profit_limit",
    "trailing_stop",
    "condition",
  ].includes(form.order_type ?? "");
  const needsConditionOperator = form.order_type === "condition";
  const needsTrailingDistance = form.order_type === "trailing_stop";
  const showPrice = !["market", "stop", "take_profit", "trailing_stop", "condition"].includes(
    form.order_type ?? ""
  );
  const showAttachFields = form.offset === "open";

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-7xl px-6 py-6">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <h1 className="text-xl font-semibold">模拟盘</h1>
            <p className="text-sm text-muted-foreground">虚拟资金练习国内期货下单与持仓管理</p>
          </div>
          <Badge variant="outline">模拟交易</Badge>
        </div>

        {snapshotLoading || !snapshot ? (
          <div className="text-sm text-muted-foreground">加载账户信息…</div>
        ) : (
          <>
            <div className="mb-4 grid grid-cols-2 gap-3 md:grid-cols-4">
              <MetricCard label="账户权益" value={formatMoney(snapshot.account.equity)} />
              <MetricCard label="可用资金" value={formatMoney(snapshot.account.cash_balance)} />
              <MetricCard label="保证金占用" value={formatMoney(snapshot.account.margin_used)} />
              <MetricCard label="风险度" value={`${(snapshot.risk_ratio * 100).toFixed(1)}%`} />
            </div>

            {quote && (
              <div className="mb-4 grid grid-cols-2 gap-3 md:grid-cols-4">
                <MetricCard label="最新价" value={quote.last_price.toFixed(2)} />
                <MetricCard label="买一" value={(quote.bid_price ?? quote.last_price).toFixed(2)} />
                <MetricCard label="卖一" value={(quote.ask_price ?? quote.last_price).toFixed(2)} />
                <MetricCard
                  label="涨跌幅"
                  value={`${quote.change_pct >= 0 ? "+" : ""}${quote.change_pct.toFixed(2)}%`}
                />
              </div>
            )}

            <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
              <Card className="lg:col-span-1">
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-semibold">下单面板</CardTitle>
                </CardHeader>
                <CardContent>
                  <form onSubmit={handleSubmit} className="space-y-3">
                    {accounts.length > 1 && (
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">账户</label>
                        <NativeSelect
                          value={accountId}
                          onChange={(e) => setSelectedAccountId(e.target.value)}
                          className="h-8 text-sm"
                        >
                          {accounts.map((a) => (
                            <option key={a.id} value={a.id}>
                              {a.name}
                            </option>
                          ))}
                        </NativeSelect>
                      </div>
                    )}
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">品种</label>
                        <Input
                          value={form.symbol}
                          onChange={(e) => setForm({ ...form, symbol: e.target.value.toUpperCase() })}
                          className="h-8 font-mono text-sm"
                        />
                      </div>
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">手数</label>
                        <Input
                          type="number"
                          min={1}
                          value={form.quantity}
                          onChange={(e) => setForm({ ...form, quantity: Number(e.target.value) })}
                          className="h-8 text-sm"
                        />
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">方向</label>
                        <NativeSelect
                          value={form.side}
                          onChange={(e) => setForm({ ...form, side: e.target.value as PlaceSimOrderRequest["side"] })}
                          className="h-8 text-sm"
                        >
                          {SIDE_OPTIONS.map((o) => (
                            <option key={o.value} value={o.value}>
                              {o.label}
                            </option>
                          ))}
                        </NativeSelect>
                      </div>
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">开平</label>
                        <NativeSelect
                          value={form.offset}
                          onChange={(e) =>
                            setForm({ ...form, offset: e.target.value as PlaceSimOrderRequest["offset"] })
                          }
                          className="h-8 text-sm"
                        >
                          {OFFSET_OPTIONS.map((o) => (
                            <option key={o.value} value={o.value}>
                              {o.label}
                            </option>
                          ))}
                        </NativeSelect>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">类型</label>
                        <NativeSelect
                          value={form.order_type}
                          onChange={(e) =>
                            setForm({ ...form, order_type: e.target.value as PlaceSimOrderRequest["order_type"] })
                          }
                          className="h-8 text-sm"
                        >
                          {TYPE_OPTIONS.map((o) => (
                            <option key={o.value} value={o.value}>
                              {o.label}
                            </option>
                          ))}
                        </NativeSelect>
                      </div>
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">有效期</label>
                        <NativeSelect
                          value={form.tif ?? "GTC"}
                          onChange={(e) => setForm({ ...form, tif: e.target.value })}
                          className="h-8 text-sm"
                        >
                          {TIF_OPTIONS.map((o) => (
                            <option key={o.value} value={o.value}>
                              {o.label}
                            </option>
                          ))}
                        </NativeSelect>
                      </div>
                    </div>
                    {showPrice && (
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">
                          {form.order_type === "stop_limit" ? "限价" : "价格"}
                        </label>
                        <Input
                          type="number"
                          value={form.price ?? ""}
                          onChange={(e) => setForm({ ...form, price: Number(e.target.value) })}
                          className="h-8 text-sm"
                        />
                      </div>
                    )}
                    {needsTriggerPrice && (
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">触发价</label>
                        <Input
                          type="number"
                          value={form.trigger_price ?? ""}
                          onChange={(e) =>
                            setForm({ ...form, trigger_price: Number(e.target.value) })
                          }
                          className="h-8 text-sm"
                        />
                      </div>
                    )}
                    {needsConditionOperator && (
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">条件方向</label>
                        <NativeSelect
                          value={form.condition_operator ?? ">="}
                          onChange={(e) =>
                            setForm({ ...form, condition_operator: e.target.value })
                          }
                          className="h-8 text-sm"
                        >
                          {CONDITION_OPERATOR_OPTIONS.map((o) => (
                            <option key={o.value} value={o.value}>
                              {o.label}
                            </option>
                          ))}
                        </NativeSelect>
                      </div>
                    )}
                    {needsTrailingDistance && (
                      <div>
                        <label className="mb-1 block text-xs text-muted-foreground">
                          回撤 tick 数
                        </label>
                        <Input
                          type="number"
                          min={1}
                          value={form.trailing_distance_ticks ?? ""}
                          onChange={(e) =>
                            setForm({
                              ...form,
                              trailing_distance_ticks: e.target.value
                                ? Number(e.target.value)
                                : undefined,
                            })
                          }
                          className="h-8 text-sm"
                        />
                      </div>
                    )}
                    {showAttachFields && (
                      <>
                        <div className="grid grid-cols-2 gap-3">
                          <div>
                            <label className="mb-1 block text-xs text-muted-foreground">止损价</label>
                            <Input
                              type="number"
                              value={form.stop_loss_price ?? ""}
                              onChange={(e) =>
                                setForm({
                                  ...form,
                                  stop_loss_price: e.target.value ? Number(e.target.value) : undefined,
                                })
                              }
                              className="h-8 text-sm"
                            />
                          </div>
                          <div>
                            <label className="mb-1 block text-xs text-muted-foreground">止盈价</label>
                            <Input
                              type="number"
                              value={form.take_profit_price ?? ""}
                              onChange={(e) =>
                                setForm({
                                  ...form,
                                  take_profit_price: e.target.value ? Number(e.target.value) : undefined,
                                })
                              }
                              className="h-8 text-sm"
                            />
                          </div>
                        </div>
                        <p className="text-xs text-muted-foreground">
                          同时填写止损和止盈将自动组成 OCO 单，任一成交后撤销另一单。
                        </p>
                      </>
                    )}
                    <Button type="submit" className="w-full" disabled={placeOrder.isPending}>
                      {placeOrder.isPending ? "提交中…" : "提交模拟委托"}
                    </Button>
                    {estimate && (
                      <div className="rounded-md border p-2 text-xs text-muted-foreground">
                        <div className="flex justify-between">
                          <span>预估保证金</span>
                          <span>{formatMoney(estimate.margin_required)}</span>
                        </div>
                        <div className="flex justify-between">
                          <span>预估手续费</span>
                          <span>{formatMoney(estimate.commission_estimate)}</span>
                        </div>
                        <div className="flex justify-between">
                          <span>预估滑点成本</span>
                          <span>{formatMoney(estimate.slippage_estimate)}</span>
                        </div>
                        <div className="mt-1 flex justify-between font-medium text-foreground">
                          <span>预估总成本</span>
                          <span>{formatMoney(estimate.total_cost)}</span>
                        </div>
                      </div>
                    )}
                    <p className="text-xs text-muted-foreground">所有委托均为模拟，不产生真实交易。</p>
                  </form>
                </CardContent>
              </Card>

              <div className="lg:col-span-2">
                <Tabs defaultValue="positions">
                  <TabsList>
                    <TabsTrigger value="positions">持仓</TabsTrigger>
                    <TabsTrigger value="orders">委托</TabsTrigger>
                    <TabsTrigger value="trades">成交</TabsTrigger>
                  </TabsList>
                  <TabsContent value="positions">
                    <PositionTable positions={positions ?? []} />
                  </TabsContent>
                  <TabsContent value="orders">
                    <OrderTable orders={orders ?? []} onCancel={(id) => cancelOrder.mutate(id)} />
                  </TabsContent>
                  <TabsContent value="trades">
                    <TradeTable trades={trades ?? []} />
                  </TabsContent>
                </Tabs>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border p-3">
      <div className="text-xs text-muted-foreground">{label}</div>
      <div className="mt-1 text-lg font-semibold tabular-nums">{value}</div>
    </div>
  );
}

function PositionTable({ positions }: { positions: SimPosition[] }) {
  if (positions.length === 0) return <Empty text="暂无持仓" />;
  return (
    <div className="rounded-lg border">
      <table className="w-full text-sm">
        <thead className="bg-muted/40">
          <tr>
            <th className="px-3 py-2 text-left">品种</th>
            <th className="px-3 py-2 text-left">方向</th>
            <th className="px-3 py-2 text-right">手数</th>
            <th className="px-3 py-2 text-right">均价</th>
            <th className="px-3 py-2 text-right">浮盈</th>
            <th className="px-3 py-2 text-right">保证金</th>
          </tr>
        </thead>
        <tbody>
          {positions.map((p) => (
            <tr key={`${p.account_id}-${p.symbol}-${p.position_side}`} className="border-t">
              <td className="px-3 py-2">{p.name}</td>
              <td className="px-3 py-2">{p.position_side === "long" ? "多" : "空"}</td>
              <td className="px-3 py-2 text-right">{p.total_qty}</td>
              <td className="px-3 py-2 text-right">{p.avg_price.toFixed(2)}</td>
              <td className={`px-3 py-2 text-right ${p.unrealized_pnl >= 0 ? "text-green-500" : "text-red-500"}`}>
                {formatMoney(p.unrealized_pnl)}
              </td>
              <td className="px-3 py-2 text-right">{formatMoney(p.margin)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function OrderTable({ orders, onCancel }: { orders: SimOrder[]; onCancel: (id: string) => void }) {
  if (orders.length === 0) return <Empty text="暂无委托" />;
  return (
    <div className="rounded-lg border">
      <table className="w-full text-sm">
        <thead className="bg-muted/40">
          <tr>
            <th className="px-3 py-2 text-left">时间</th>
            <th className="px-3 py-2 text-left">品种</th>
            <th className="px-3 py-2 text-left">方向</th>
            <th className="px-3 py-2 text-left">类型</th>
            <th className="px-3 py-2 text-right">价格</th>
            <th className="px-3 py-2 text-right">触发价</th>
            <th className="px-3 py-2 text-right">数量</th>
            <th className="px-3 py-2 text-left">状态</th>
            <th className="px-3 py-2 text-right">操作</th>
          </tr>
        </thead>
        <tbody>
          {orders.map((o) => (
            <tr key={o.id} className="border-t">
              <td className="px-3 py-2 text-xs text-muted-foreground">{new Date(o.created_at).toLocaleTimeString()}</td>
              <td className="px-3 py-2">{o.name}</td>
              <td className="px-3 py-2">{orderDirLabel(o)}</td>
              <td className="px-3 py-2 text-xs">{orderTypeLabel(o)}</td>
              <td className="px-3 py-2 text-right">{orderPriceLabel(o)}</td>
              <td className="px-3 py-2 text-right">{o.trigger_price ?? "—"}</td>
              <td className="px-3 py-2 text-right">
                {o.filled_quantity}/{o.quantity}
              </td>
              <td className="px-3 py-2">
                {orderStatusLabel(o.status)}
                {o.reason && <span className="ml-1 text-xs text-muted-foreground">({o.reason})</span>}
              </td>
              <td className="px-3 py-2 text-right">
                {o.status === "open" && (
                  <Button size="sm" variant="ghost" onClick={() => onCancel(o.id)}>
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
  if (trades.length === 0) return <Empty text="暂无成交" />;
  return (
    <div className="rounded-lg border">
      <table className="w-full text-sm">
        <thead className="bg-muted/40">
          <tr>
            <th className="px-3 py-2 text-left">时间</th>
            <th className="px-3 py-2 text-left">品种</th>
            <th className="px-3 py-2 text-left">方向</th>
            <th className="px-3 py-2 text-right">成交价</th>
            <th className="px-3 py-2 text-right">手数</th>
            <th className="px-3 py-2 text-right">手续费</th>
          </tr>
        </thead>
        <tbody>
          {trades.map((t) => (
            <tr key={t.id} className="border-t">
              <td className="px-3 py-2 text-xs text-muted-foreground">{new Date(t.traded_at).toLocaleTimeString()}</td>
              <td className="px-3 py-2">{t.name}</td>
              <td className="px-3 py-2">{tradeDirLabel(t)}</td>
              <td className="px-3 py-2 text-right">{t.price.toFixed(2)}</td>
              <td className="px-3 py-2 text-right">{t.quantity}</td>
              <td className="px-3 py-2 text-right">{formatMoney(t.commission)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Empty({ text }: { text: string }) {
  return <div className="rounded-lg border py-8 text-center text-sm text-muted-foreground">{text}</div>;
}

function formatMoney(n: number) {
  return `¥${n.toLocaleString("zh-CN", { maximumFractionDigits: 2 })}`;
}

function orderDirLabel(o: SimOrder) {
  const side = o.side === "buy" ? "买" : "卖";
  const offset =
    o.offset === "open" ? "开" : o.offset === "close_today" ? "平今" : o.offset === "close_yesterday" ? "平昨" : "平";
  return `${side}${offset}`;
}

function orderTypeLabel(o: SimOrder) {
  const map: Record<string, string> = {
    market: "市价",
    limit: "限价",
    stop: "止损",
    stop_limit: "止损限价",
    take_profit: "止盈",
    take_profit_limit: "止盈限价",
    trailing_stop: "移动止损",
    condition: "条件单",
  };
  return map[o.order_type] ?? o.order_type;
}

function orderPriceLabel(o: SimOrder) {
  if (o.order_type === "market") return "市价";
  if (["stop", "take_profit", "trailing_stop", "condition"].includes(o.order_type)) {
    const op = o.condition_operator ? ` ${o.condition_operator}` : "";
    return `${orderTypeLabel(o)}${op} ${o.trigger_price ?? "—"}`;
  }
  if (["stop_limit", "take_profit_limit"].includes(o.order_type)) {
    return `${orderTypeLabel(o)} ${o.price ?? "—"} / 触 ${o.trigger_price ?? "—"}`;
  }
  return o.price?.toFixed(2) ?? "—";
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

function tradeDirLabel(t: { side: string; offset: string }) {
  const side = t.side === "buy" ? "买" : "卖";
  const offset =
    t.offset === "open" ? "开" : t.offset === "close_today" ? "平今" : t.offset === "close_yesterday" ? "平昨" : "平";
  return `${side}${offset}`;
}

export default SimulationPage;
