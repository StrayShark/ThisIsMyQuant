import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/api/client";
import type { AnalysisReport, SimJournalEntry, SimTrade, SimEquitySnapshot, SimPerformance } from "@/types";
import { useAppStore } from "@/app/store";

export function TradingReviewPage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const { data: snapshot } = useQuery({
    queryKey: ["sim-snapshot"],
    queryFn: () => api.getSimAccountSnapshot(),
  });
  const { data: trades } = useQuery({
    queryKey: ["sim-trades"],
    queryFn: () => api.listSimTrades(),
  });
  const { data: curve } = useQuery({
    queryKey: ["sim-equity-curve"],
    queryFn: () => api.listSimEquityCurve({ days: 30 }),
  });
  const { data: performance } = useQuery({
    queryKey: ["sim-performance"],
    queryFn: () => api.getSimPerformance(),
  });
  const { data: journals } = useQuery({
    queryKey: ["sim-journals"],
    queryFn: () => api.listSimJournalEntries(),
  });

  const [reviewReport, setReviewReport] = useState<AnalysisReport | null>(null);

  const generateReview = useMutation({
    mutationFn: () => api.generateTradeReview({}),
    onSuccess: (report) => {
      setReviewReport(report);
      showToast("LLM 复盘报告已生成");
    },
    onError: (err: Error) => showToast(err.message),
  });

  const saveJournal = useMutation({
    mutationFn: (payload: Partial<SimJournalEntry>) => api.saveSimJournalEntry(payload),
    onSuccess: () => {
      showToast("复盘日记已保存");
      setDraft({ title: "", thesis: "", execution_review: "", emotion_tags: "", score: undefined });
      void queryClient.invalidateQueries({ queryKey: ["sim-journals"] });
    },
    onError: (err: Error) => showToast(err.message),
  });

  const [draft, setDraft] = useState<Partial<SimJournalEntry>>({
    title: "",
    thesis: "",
    execution_review: "",
    emotion_tags: "",
    score: undefined,
  });

  const handleSave = () => {
    if (!draft.title) return;
    saveJournal.mutate({
      account_id: snapshot?.account.id,
      title: draft.title,
      thesis: draft.thesis,
      execution_review: draft.execution_review,
      emotion_tags: draft.emotion_tags,
      score: draft.score ? Number(draft.score) : undefined,
    });
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-7xl px-6 py-6">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <h1 className="text-xl font-semibold">交易复盘</h1>
            <p className="text-sm text-muted-foreground">记录交易计划、执行评分与绩效归因</p>
          </div>
          <Badge variant="outline">模拟交易</Badge>
        </div>

        <div className="mb-4 grid grid-cols-2 gap-3 md:grid-cols-5">
          <MetricCard label="总盈亏" value={formatMoney((snapshot?.account.realized_pnl ?? 0) + (snapshot?.account.unrealized_pnl ?? 0))} />
          <MetricCard label="已实现盈亏" value={formatMoney(snapshot?.account.realized_pnl ?? 0)} />
          <MetricCard label="未实现盈亏" value={formatMoney(snapshot?.account.unrealized_pnl ?? 0)} />
          <MetricCard label="交易笔数" value={`${(trades ?? []).length} 笔`} />
          <div className="flex items-center">
            <Button onClick={() => generateReview.mutate()} disabled={generateReview.isPending} className="w-full">
              {generateReview.isPending ? "生成中…" : "生成 LLM 复盘"}
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
          <div className="lg:col-span-2 space-y-4">
            {reviewReport && (
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-semibold">LLM 复盘摘要</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground">
                    {reviewReport.content}
                  </div>
                  <p className="mt-2 text-xs text-muted-foreground">
                    生成时间：{new Date(reviewReport.created_at).toLocaleString()} · 模型：{reviewReport.provider}
                  </p>
                </CardContent>
              </Card>
            )}
            <Tabs defaultValue="curve">
              <TabsList>
                <TabsTrigger value="curve">资金曲线</TabsTrigger>
                <TabsTrigger value="performance">绩效统计</TabsTrigger>
                <TabsTrigger value="trades">成交记录</TabsTrigger>
                <TabsTrigger value="journals">复盘日记</TabsTrigger>
              </TabsList>
              <TabsContent value="curve">
                <EquityCurveCard data={curve ?? []} />
              </TabsContent>
              <TabsContent value="performance">
                <PerformancePanel performance={performance ?? null} />
              </TabsContent>
              <TabsContent value="trades">
                <TradeTable trades={trades ?? []} />
              </TabsContent>
              <TabsContent value="journals">
                <JournalList journals={journals ?? []} />
              </TabsContent>
            </Tabs>
          </div>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-semibold">新增复盘日记</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <Input
                placeholder="标题"
                value={draft.title}
                onChange={(e) => setDraft({ ...draft, title: e.target.value })}
                className="h-8 text-sm"
              />
              <Textarea
                placeholder="交易计划 / 入场理由"
                value={draft.thesis ?? ""}
                onChange={(e) => setDraft({ ...draft, thesis: e.target.value })}
                className="min-h-[80px] text-sm"
              />
              <Textarea
                placeholder="执行回顾"
                value={draft.execution_review ?? ""}
                onChange={(e) => setDraft({ ...draft, execution_review: e.target.value })}
                className="min-h-[80px] text-sm"
              />
              <Input
                placeholder="情绪标签，用逗号分隔"
                value={draft.emotion_tags ?? ""}
                onChange={(e) => setDraft({ ...draft, emotion_tags: e.target.value })}
                className="h-8 text-sm"
              />
              <Input
                type="number"
                min={1}
                max={10}
                placeholder="执行评分 1-10"
                value={draft.score ?? ""}
                onChange={(e) => setDraft({ ...draft, score: Number(e.target.value) })}
                className="h-8 text-sm"
              />
              <Button className="w-full" onClick={handleSave} disabled={saveJournal.isPending || !draft.title}>
                {saveJournal.isPending ? "保存中…" : "保存日记"}
              </Button>
            </CardContent>
          </Card>
        </div>
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

function EquityCurveCard({ data }: { data: SimEquitySnapshot[] }) {
  if (data.length === 0) return <Empty text="暂无资金曲线数据" />;
  const min = Math.min(...data.map((d) => d.equity));
  const max = Math.max(...data.map((d) => d.equity));
  const range = Math.max(max - min, 1);
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">权益走势</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex h-48 items-end gap-1">
          {data.map((d, i) => {
            const h = ((d.equity - min) / range) * 100;
            return (
              <div key={i} className="group relative flex-1">
                <div className="w-full rounded-sm bg-primary/70 hover:bg-primary" style={{ height: `${Math.max(h, 5)}%` }} />
                <div className="pointer-events-none absolute bottom-full left-1/2 mb-1 hidden -translate-x-1/2 whitespace-nowrap rounded bg-popover px-2 py-1 text-xs text-popover-foreground group-hover:block">
                  {new Date(d.snapshot_at).toLocaleDateString()} ¥{d.equity.toFixed(0)}
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

function PerformancePanel({ performance }: { performance: SimPerformance | null }) {
  if (!performance) return <Empty text="暂无绩效数据" />;
  const symbolEntries = Object.entries(performance.symbol_contribution).sort((a, b) => b[1] - a[1]);
  const hourlyEntries = Object.entries(performance.hourly_contribution).sort((a, b) => b[1] - a[1]);
  const maxSymbol = Math.max(...symbolEntries.map(([, v]) => Math.abs(v)), 1);
  const maxHourly = Math.max(...hourlyEntries.map(([, v]) => Math.abs(v)), 1);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-semibold">核心指标</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
            <MetricCard label="总收益率" value={`${(performance.total_return_pct * 100).toFixed(2)}%`} />
            <MetricCard label="总盈亏" value={formatMoney(performance.total_pnl)} />
            <MetricCard label="最大回撤" value={`${(performance.max_drawdown_pct * 100).toFixed(2)}%`} />
            <MetricCard label="胜率" value={`${(performance.win_rate * 100).toFixed(1)}%`} />
            <MetricCard label="盈亏比" value={performance.profit_loss_ratio.toFixed(2)} />
            <MetricCard label="风险回报比" value={performance.risk_return_ratio.toFixed(2)} />
            <MetricCard label="平均持仓" value={`${performance.avg_holding_hours.toFixed(1)}h`} />
            <MetricCard label="隔夜次数" value={`${performance.overnight_count}`} />
          </div>
          <div className="mt-3 grid grid-cols-3 gap-3 text-center text-sm">
            <div className="rounded-md border p-2">
              <div className="text-xs text-muted-foreground">总交易</div>
              <div className="font-semibold">{performance.total_trades}</div>
            </div>
            <div className="rounded-md border p-2">
              <div className="text-xs text-muted-foreground">盈利笔</div>
              <div className="font-semibold text-green-500">{performance.winning_trades}</div>
            </div>
            <div className="rounded-md border p-2">
              <div className="text-xs text-muted-foreground">亏损笔</div>
              <div className="font-semibold text-red-500">{performance.losing_trades}</div>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">品种贡献</CardTitle>
          </CardHeader>
          <CardContent>
            {symbolEntries.length === 0 ? (
              <Empty text="暂无品种贡献数据" />
            ) : (
              <div className="space-y-2">
                {symbolEntries.map(([sym, val]) => (
                  <div key={sym} className="flex items-center gap-2 text-sm">
                    <span className="w-16 font-mono">{sym}</span>
                    <div className="flex-1">
                      <div
                        className={`h-2 rounded-sm ${val >= 0 ? "bg-green-500" : "bg-red-500"}`}
                        style={{ width: `${(Math.abs(val) / maxSymbol) * 100}%` }}
                      />
                    </div>
                    <span className={`tabular-nums ${val >= 0 ? "text-green-500" : "text-red-500"}`}>
                      {formatMoney(val)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">时段贡献</CardTitle>
          </CardHeader>
          <CardContent>
            {hourlyEntries.length === 0 ? (
              <Empty text="暂无时段贡献数据" />
            ) : (
              <div className="space-y-2">
                {hourlyEntries.map(([hour, val]) => (
                  <div key={hour} className="flex items-center gap-2 text-sm">
                    <span className="w-12">{hour}</span>
                    <div className="flex-1">
                      <div
                        className={`h-2 rounded-sm ${val >= 0 ? "bg-green-500" : "bg-red-500"}`}
                        style={{ width: `${(Math.abs(val) / maxHourly) * 100}%` }}
                      />
                    </div>
                    <span className={`tabular-nums ${val >= 0 ? "text-green-500" : "text-red-500"}`}>
                      {formatMoney(val)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
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
              <td className="px-3 py-2 text-xs text-muted-foreground">{new Date(t.traded_at).toLocaleString()}</td>
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

function JournalList({ journals }: { journals: SimJournalEntry[] }) {
  if (journals.length === 0) return <Empty text="暂无复盘日记" />;
  return (
    <div className="space-y-3">
      {journals.map((j) => (
        <Card key={j.id}>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">{j.title}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            {j.thesis && <p className="text-muted-foreground">计划：{j.thesis}</p>}
            {j.execution_review && <p className="text-muted-foreground">执行：{j.execution_review}</p>}
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              {j.score && <Badge variant="outline">评分 {j.score}</Badge>}
              {j.emotion_tags && <span>情绪：{j.emotion_tags}</span>}
              <span>{new Date(j.created_at).toLocaleString()}</span>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function Empty({ text }: { text: string }) {
  return <div className="rounded-lg border py-8 text-center text-sm text-muted-foreground">{text}</div>;
}

function formatMoney(n: number) {
  const sign = n >= 0 ? "+" : "";
  return `${sign}¥${n.toLocaleString("zh-CN", { maximumFractionDigits: 2 })}`;
}

function tradeDirLabel(t: SimTrade) {
  const side = t.side === "buy" ? "买" : "卖";
  const offset = t.offset === "open" ? "开" : t.offset === "close_today" ? "平今" : t.offset === "close_yesterday" ? "平昨" : "平";
  return `${side}${offset}`;
}

export default TradingReviewPage;
