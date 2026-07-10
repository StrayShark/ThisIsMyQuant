import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  createChart,
  CandlestickSeries,
  HistogramSeries,
  type IChartApi,
  type ISeriesApi,
  type Time,
} from "lightweight-charts";
import { api } from "@/api/client";
import { wsClient } from "@/ws/socket";
import { useAppStore } from "@/app/store";
import { getChartTheme } from "@/lib/chart-theme";
import {
  buildCandleOptions,
  buildChartOptions,
  buildVolumeOptions,
  volumeBarColor,
} from "@/features/chart/chart-options";
import { defaultChartConfigFromTheme, type ChartUserConfig } from "@/features/chart/chart-config";
import type {
  Interval,
  KLine,
  PlaceSimOrderRequest,
  SimOrder,
  SimOrderEstimate,
  SimPosition,
  SimTrade,
  WsMessage,
} from "@/types";

const INTERVALS: { value: Interval; label: string }[] = [
  { value: "1m", label: "1m" },
  { value: "5m", label: "5m" },
  { value: "15m", label: "15m" },
  { value: "30m", label: "30m" },
  { value: "1h", label: "1h" },
  { value: "1d", label: "1d" },
];

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
  { value: "condition", label: "条件单" },
];

function toCandle(k: KLine) {
  return {
    time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
    open: k.open,
    high: k.high,
    low: k.low,
    close: k.close,
  };
}

function toVolume(k: KLine, config: ChartUserConfig) {
  return {
    time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
    value: k.volume,
    color: volumeBarColor(k.close, k.open, config),
  };
}

export function MarketReplayPage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);

  const { data: accounts = [] } = useQuery({
    queryKey: ["sim-accounts"],
    queryFn: () => api.listSimAccounts(),
  });

  const [symbol, setSymbol] = useState("RB0");
  const [date, setDate] = useState("2024-01-15");
  const [interval, setInterval] = useState<Interval>("1m");
  const [speed, setSpeed] = useState(1);
  const [selectedAccountId, setSelectedAccountId] = useState<string | undefined>(undefined);
  const accountId = selectedAccountId ?? accounts[0]?.id;

  const { data: state, refetch: refetchState } = useQuery({
    queryKey: ["replay-state"],
    queryFn: () => api.getReplayState(),
    refetchInterval: 1000,
  });

  const { data: klinePayload, refetch: refetchKlines } = useQuery({
    queryKey: ["replay-klines"],
    queryFn: () => api.getReplayKlines(),
    enabled: !!(state as import("@/types").ReplayState | undefined)?.symbol,
    refetchInterval: 1000,
  });

  const { data: snapshot } = useQuery({
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

  useEffect(() => {
    const off = wsClient.on((msg: WsMessage) => {
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

  const startReplay = useMutation({
    mutationFn: () =>
      api.startMarketReplay({
        symbol,
        date,
        account_id: accountId,
        speed,
      }),
    onSuccess: () => {
      showToast("回放已启动");
      void refetchState();
      void refetchKlines();
    },
    onError: (err: Error) => showToast(err.message),
  });

  const stopReplay = useMutation({
    mutationFn: () => api.stopMarketReplay(),
    onSuccess: () => {
      showToast("回放已停止");
      void refetchState();
    },
    onError: (err: Error) => showToast(err.message),
  });

  const stepReplay = useMutation({
    mutationFn: () => api.stepMarketReplay(1),
    onSuccess: () => {
      void refetchState();
      void refetchKlines();
    },
    onError: (err: Error) => showToast(err.message),
  });

  const [form, setForm] = useState<Partial<PlaceSimOrderRequest>>({
    symbol: "RB0",
    side: "buy",
    offset: "open",
    order_type: "limit",
    price: 3200,
    quantity: 1,
  });

  const [estimate, setEstimate] = useState<SimOrderEstimate | null>(null);

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
      quantity: form.quantity ?? 1,
    };
    const timer = setTimeout(() => {
      void estimateMutation.mutate(payload);
    }, 300);
    return () => clearTimeout(timer);
  }, [form, accountId, estimateMutation]);

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
      quantity: form.quantity ?? 1,
    });
  };

  const needsTriggerPrice = ["stop", "stop_limit", "condition"].includes(form.order_type ?? "");
  const showPrice = form.order_type !== "market";

  const progress = useMemo(() => {
    if (!state || state.total_bars === 0) return 0;
    return Math.min(100, (state.current_index / state.total_bars) * 100);
  }, [state]);

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-7xl px-6 py-6">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <h1 className="text-xl font-semibold">回放训练</h1>
            <p className="text-sm text-muted-foreground">按历史行情练习下单，不显示未来数据</p>
          </div>
          <Badge variant="outline">历史训练</Badge>
        </div>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
          <Card className="lg:col-span-1">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-semibold">回放设置</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div>
                <label className="mb-1 block text-xs text-muted-foreground">品种</label>
                <Input
                  value={symbol}
                  onChange={(e) => setSymbol(e.target.value.toUpperCase())}
                  className="h-8 text-sm"
                  disabled={state?.running}
                />
              </div>
              <div>
                <label className="mb-1 block text-xs text-muted-foreground">日期</label>
                <Input
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  className="h-8 text-sm"
                  disabled={state?.running}
                />
              </div>
              <div>
                <label className="mb-1 block text-xs text-muted-foreground">周期</label>
                <NativeSelect
                  value={interval}
                  onChange={(e) => setInterval(e.target.value as Interval)}
                  className="h-8 text-sm"
                  disabled={state?.running}
                >
                  {INTERVALS.map((it) => (
                    <option key={it.value} value={it.value}>
                      {it.label}
                    </option>
                  ))}
                </NativeSelect>
              </div>
              <div>
                <label className="mb-1 block text-xs text-muted-foreground">倍速</label>
                <Input
                  type="number"
                  min={1}
                  max={20}
                  value={speed}
                  onChange={(e) => setSpeed(Number(e.target.value))}
                  className="h-8 text-sm"
                  disabled={state?.running}
                />
              </div>
              {accounts.length > 0 && (
                <div>
                  <label className="mb-1 block text-xs text-muted-foreground">账户</label>
                  <NativeSelect
                    value={accountId}
                    onChange={(e) => setSelectedAccountId(e.target.value)}
                    className="h-8 text-sm"
                    disabled={state?.running}
                  >
                    {accounts.map((a) => (
                      <option key={a.id} value={a.id}>
                        {a.name}
                      </option>
                    ))}
                  </NativeSelect>
                </div>
              )}
              <div className="flex gap-2">
                <Button
                  className="flex-1"
                  onClick={() => startReplay.mutate()}
                  disabled={startReplay.isPending || state?.running || !accountId}
                >
                  开始
                </Button>
                <Button
                  className="flex-1"
                  variant="secondary"
                  onClick={() => stepReplay.mutate()}
                  disabled={stepReplay.isPending || !state?.symbol || state?.completed}
                >
                  步进
                </Button>
                <Button
                  className="flex-1"
                  variant="destructive"
                  onClick={() => stopReplay.mutate()}
                  disabled={stopReplay.isPending || !state?.running}
                >
                  停止
                </Button>
              </div>

              <div className="space-y-1">
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>进度</span>
                  <span>
                    {state?.current_index ?? 0} / {state?.total_bars ?? 0}
                  </span>
                </div>
                <div className="h-2 overflow-hidden rounded-full bg-muted">
                  <div
                    className="h-full bg-primary transition-all"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="lg:col-span-2">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-semibold">回放图表</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="mb-3 grid grid-cols-2 gap-3 md:grid-cols-4">
                <StateItem label="运行中" value={state?.running ? "是" : "否"} />
                <StateItem label="品种" value={state?.symbol || "-"} />
                <StateItem label="当前价" value={state?.current_price?.toFixed(2) ?? "-"} />
                <StateItem
                  label="当前 Bar"
                  value={state?.current_bar_time ? new Date(state.current_bar_time).toLocaleTimeString() : "-"}
                />
              </div>
              <ReplayChart bars={klinePayload?.bars ?? []} />
            </CardContent>
          </Card>
        </div>

        <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-3">
          <Card className="lg:col-span-1">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-semibold">回放下单</CardTitle>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmit} className="space-y-3">
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
                      onChange={(e) =>
                        setForm({ ...form, side: e.target.value as PlaceSimOrderRequest["side"] })
                      }
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
                <Button type="submit" className="w-full" disabled={placeOrder.isPending || !accountId}>
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
            {snapshot && (
              <div className="mb-4 grid grid-cols-2 gap-3 md:grid-cols-4">
                <MetricCard label="账户权益" value={formatMoney(snapshot.account.equity)} />
                <MetricCard label="可用资金" value={formatMoney(snapshot.account.cash_balance)} />
                <MetricCard label="保证金占用" value={formatMoney(snapshot.account.margin_used)} />
                <MetricCard label="风险度" value={`${(snapshot.risk_ratio * 100).toFixed(1)}%`} />
              </div>
            )}

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
      </div>
    </div>
  );
}

function ReplayChart({ bars }: { bars: KLine[] }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const config = useMemo(() => defaultChartConfigFromTheme(getChartTheme()), []);

  useEffect(() => {
    if (!containerRef.current) return;
    const chart = createChart(containerRef.current, buildChartOptions(config));
    chartRef.current = chart;
    const candle = chart.addSeries(CandlestickSeries, buildCandleOptions(config));
    candleRef.current = candle;
    chart.addPane();
    const volume = chart.addSeries(HistogramSeries, buildVolumeOptions(config));
    volumeRef.current = volume;
    chart.timeScale().fitContent();

    return () => {
      chart.remove();
      chartRef.current = null;
      candleRef.current = null;
      volumeRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!candleRef.current || !volumeRef.current) return;
    candleRef.current.setData(bars.map(toCandle));
    volumeRef.current.setData(bars.map((k) => toVolume(k, config)));
    chartRef.current?.timeScale().fitContent();
  }, [bars, config]);

  return <div ref={containerRef} className="h-80 w-full rounded-md border" />;
}

function StateItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border p-3">
      <div className="text-xs text-muted-foreground">{label}</div>
      <div className="mt-1 text-sm font-medium">{value}</div>
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
              <td className="px-3 py-2 text-xs text-muted-foreground">
                {new Date(o.created_at).toLocaleTimeString()}
              </td>
              <td className="px-3 py-2">{o.name}</td>
              <td className="px-3 py-2">{orderDirLabel(o)}</td>
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
              <td className="px-3 py-2 text-xs text-muted-foreground">
                {new Date(t.traded_at).toLocaleTimeString()}
              </td>
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

function orderPriceLabel(o: SimOrder) {
  if (o.order_type === "market") return "市价";
  if (o.order_type === "stop") return `止损 ${o.trigger_price ?? "—"}`;
  if (o.order_type === "stop_limit") return `限价 ${o.price ?? "—"}`;
  if (o.order_type === "condition") return `条件 ${o.trigger_price ?? "—"}`;
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

export default MarketReplayPage;
