import { useState } from "react";
import { Link, Navigate, useSearchParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { PanelSkeleton } from "@/components/ui/panel-skeleton";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { NativeSelect } from "@/components/ui/native-select";
import { useAppStore } from "@/app/store";
import { api } from "@/api/client";
import { dimensionLabel } from "@/data/dimensions";
import { Skeleton } from "@/components/ui/skeleton";
import {
  parseSettingsSection,
  type SettingsSectionId,
} from "@/features/settings/settings-sections";
import { AppearanceSettingsPanel } from "@/features/settings/AppearanceSettingsPanel";
import { SimulationRulesPanel } from "@/features/settings/SimulationRulesPanel";
import { useUserPreferences } from "@/hooks/useUserPreferences";
import type { NewsRecord, ScheduleStatus } from "@/types";

function SettingsSectionShell({ children }: { children: ReactNode }) {
  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-2xl px-8 py-8">
        <div className="space-y-4">{children}</div>
      </div>
    </div>
  );
}

function SettingRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-center justify-between border-b border-border py-3 last:border-0">
      <span className="text-sm text-foreground">{label}</span>
      <span className="text-sm text-muted-foreground">{value}</span>
    </div>
  );
}

function scheduleStatusLabel(
  status: ScheduleStatus | undefined,
  configEnabled: boolean
): { text: string; variant: "default" | "secondary" | "outline" | "up" | "down" } {
  if (status?.cycle_in_progress) {
    return { text: "执行中", variant: "default" };
  }
  if (!configEnabled) {
    return { text: "已关闭", variant: "secondary" };
  }
  if (status?.enabled) {
    return { text: "运行中", variant: "default" };
  }
  return { text: "待命", variant: "outline" };
}

function nextCycleHint(status: ScheduleStatus | undefined, configEnabled: boolean): string {
  if (!configEnabled) return "定时任务已关闭";
  if (status?.cycle_in_progress) return "当前周期执行中";
  if (status?.last_cycle_at) {
    const next = new Date(status.last_cycle_at);
    next.setHours(next.getHours() + (status.interval_hours || 6));
    if (next.getTime() > Date.now()) {
      return `约 ${next.toLocaleString("zh-CN")}`;
    }
    return "上一周期已完成，等待下一窗口";
  }
  return "应用启动后约 90 秒首次执行";
}

function PrefCheckbox({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center justify-between border-b border-border py-3 last:border-0">
      <span className="text-sm text-foreground">{label}</span>
      <input
        type="checkbox"
        className="h-4 w-4 accent-primary"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
    </label>
  );
}

function PrefNumber({
  label,
  value,
  onChange,
  min,
  max,
  step = 1,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
}) {
  return (
    <div className="flex items-center justify-between gap-3 border-b border-border py-3 last:border-0">
      <span className="text-sm text-foreground">{label}</span>
      <Input
        type="number"
        className="h-8 w-24 font-mono text-sm"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </div>
  );
}

function AppPreferencesEditor() {
  const { prefs, isLoading, update } = useUserPreferences();

  if (isLoading || !prefs) {
    return <Skeleton className="h-64 rounded-lg" />;
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">运营配置</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-x-8 md:grid-cols-2">
          <div>
            <p className="mb-2 text-xs font-medium text-muted-foreground">数据源</p>
            <PrefCheckbox
              label="AKShare K 线"
              checked={prefs.akshare_enabled}
              onChange={(v) => update({ akshare_enabled: v })}
            />
            <PrefCheckbox
              label="实时行情轮询"
              checked={prefs.akshare_realtime_enabled}
              onChange={(v) => update({ akshare_realtime_enabled: v })}
            />
            <PrefNumber
              label="行情轮询间隔（秒）"
              value={prefs.realtime_poll_interval}
              min={1}
              step={0.5}
              onChange={(v) => update({ realtime_poll_interval: v }, { debounceMs: 400 })}
            />
            <PrefCheckbox
              label="金十资讯"
              checked={prefs.jinshi_enabled}
              onChange={(v) => update({ jinshi_enabled: v })}
            />
            <PrefNumber
              label="资讯轮询间隔（秒）"
              value={prefs.jinshi_poll_interval}
              min={30}
              onChange={(v) => update({ jinshi_poll_interval: v }, { debounceMs: 400 })}
            />
          </div>
          <div>
            <p className="mb-2 text-xs font-medium text-muted-foreground">分析与异动</p>
            <div className="flex items-center justify-between gap-3 border-b border-border py-3">
              <span className="text-sm text-foreground">默认 LLM</span>
              <Input
                value={prefs.default_llm_provider}
                onChange={(e) =>
                  update({ default_llm_provider: e.target.value }, { debounceMs: 500 })
                }
                className="h-8 w-28 font-mono text-sm"
              />
            </div>
            <PrefCheckbox
              label="资讯 LLM 分类"
              checked={prefs.news_classify_enabled}
              onChange={(v) => update({ news_classify_enabled: v })}
            />
            <PrefNumber
              label="分类 batch 大小"
              value={prefs.news_classify_batch}
              min={1}
              max={50}
              onChange={(v) => update({ news_classify_batch: v }, { debounceMs: 400 })}
            />
            <PrefCheckbox
              label="异动检测"
              checked={prefs.anomaly_enabled}
              onChange={(v) => update({ anomaly_enabled: v })}
            />
            <PrefNumber
              label="异动阈值（%）"
              value={prefs.anomaly_price_pct}
              min={0.1}
              step={0.1}
              onChange={(v) => update({ anomaly_price_pct: v }, { debounceMs: 400 })}
            />
            <PrefNumber
              label="异动窗口（秒）"
              value={prefs.anomaly_window_secs}
              min={60}
              onChange={(v) => update({ anomaly_window_secs: v }, { debounceMs: 400 })}
            />
            <PrefNumber
              label="异动冷却（秒）"
              value={prefs.anomaly_cooldown_secs}
              min={60}
              onChange={(v) => update({ anomaly_cooldown_secs: v }, { debounceMs: 400 })}
            />
          </div>
          <div>
            <p className="mb-2 text-xs font-medium text-muted-foreground">数据维护</p>
            <PrefNumber
              label="日 K 回填天数"
              value={prefs.backfill_days_daily}
              min={1}
              onChange={(v) => update({ backfill_days_daily: v }, { debounceMs: 400 })}
            />
            <PrefNumber
              label="1 分钟 K 回填天数"
              value={prefs.backfill_days_minute}
              min={1}
              onChange={(v) => update({ backfill_days_minute: v }, { debounceMs: 400 })}
            />
            <PrefCheckbox
              label="Tick 入库"
              checked={prefs.ticks_enabled}
              onChange={(v) => update({ ticks_enabled: v })}
            />
            <PrefNumber
              label="K 线保留天数"
              value={prefs.retention_days_klines}
              min={7}
              onChange={(v) => update({ retention_days_klines: v }, { debounceMs: 400 })}
            />
            <PrefNumber
              label="Tick 保留天数"
              value={prefs.retention_days_ticks}
              min={1}
              onChange={(v) => update({ retention_days_ticks: v }, { debounceMs: 400 })}
            />
            <PrefCheckbox
              label="日历事件提醒"
              checked={prefs.calendar_reminder_enabled}
              onChange={(v) => update({ calendar_reminder_enabled: v })}
            />
            <PrefNumber
              label="提醒提前（分钟）"
              value={prefs.calendar_reminder_mins}
              min={1}
              onChange={(v) => update({ calendar_reminder_mins: v }, { debounceMs: 400 })}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function ScheduleTaskPanel() {
  const queryClient = useQueryClient();
  const { prefs, isLoading: prefsLoading, update } = useUserPreferences();
  const [actionMsg, setActionMsg] = useState("");
  const [acting, setActing] = useState(false);

  const { data: settings } = useQuery({
    queryKey: ["app-settings"],
    queryFn: () => api.getSettings(),
    staleTime: 60_000,
  });

  const { data: status, refetch: refetchStatus } = useQuery({
    queryKey: ["schedule-status"],
    queryFn: () => api.getScheduleStatus(),
    refetchInterval: 10_000,
  });

  const coreCount = settings?.core_product_count ?? 0;
  const scheduleEnabled = prefs?.schedule_enabled ?? true;
  const intervalHours = prefs?.schedule_interval_hours ?? 6;
  const analysisTrigger = prefs?.schedule_analysis_trigger ?? "scheduled";
  const dailyBriefingEnabled = prefs?.daily_briefing_enabled ?? true;
  const dailyBriefingHour = prefs?.daily_briefing_hour ?? 17;
  const statusBadge = scheduleStatusLabel(status, scheduleEnabled);

  const runAction = async (fn: () => Promise<void>) => {
    setActing(true);
    setActionMsg("");
    try {
      await fn();
      await refetchStatus();
      queryClient.invalidateQueries({ queryKey: ["runtime-status"] });
    } catch (e) {
      setActionMsg(e instanceof Error ? e.message : "操作失败");
    } finally {
      setActing(false);
    }
  };

  if (prefsLoading || !prefs) {
    return <Skeleton className="h-72 rounded-lg" />;
  }

  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between space-y-0 pb-2">
        <div className="space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <CardTitle className="text-sm font-semibold">定时任务</CardTitle>
            <Badge variant={statusBadge.variant}>{statusBadge.text}</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="grid gap-6 md:grid-cols-2">
          <div className="space-y-3">
            <p className="text-xs font-medium text-muted-foreground">配置</p>
            <p className="text-[11px] text-muted-foreground">
              定时分析全部核心品种（共 {coreCount} 个）
            </p>
            <PrefCheckbox
              label="启用定时任务"
              checked={scheduleEnabled}
              onChange={(v) => update({ schedule_enabled: v })}
            />
            <PrefNumber
              label="执行间隔（小时）"
              value={intervalHours}
              min={1}
              max={168}
              onChange={(v) => update({ schedule_interval_hours: v }, { debounceMs: 400 })}
            />
            <div className="flex items-center justify-between gap-3 border-b border-border py-3 last:border-0">
              <span className="text-sm text-foreground">周期分析类型</span>
              <NativeSelect
                className="h-8 w-36 text-xs"
                value={analysisTrigger}
                onChange={(e) => update({ schedule_analysis_trigger: e.target.value })}
              >
                <option value="scheduled">定时全面</option>
                <option value="tomorrow">明日展望</option>
                <option value="short_term">短期研判</option>
                <option value="manual">手动风格</option>
              </NativeSelect>
            </div>
            <PrefCheckbox
              label="每日简报（明日展望）"
              checked={dailyBriefingEnabled}
              onChange={(v) => update({ daily_briefing_enabled: v })}
            />
            <PrefNumber
              label="简报时刻（小时 0–23）"
              value={dailyBriefingHour}
              min={0}
              max={23}
              onChange={(v) => update({ daily_briefing_hour: v }, { debounceMs: 400 })}
            />
          </div>

          <div className="space-y-1">
            <div className="mb-2 flex items-center justify-between">
              <p className="text-xs font-medium text-muted-foreground">运行状态</p>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs"
                onClick={() => refetchStatus()}
              >
                刷新
              </Button>
            </div>
            <SettingRow
              label="有效周期"
              value={
                <span className="font-mono text-xs">
                  每 {status?.interval_hours ?? intervalHours} 小时
                </span>
              }
            />
            <SettingRow
              label="预计下次"
              value={
                <span className="font-mono text-xs">
                  {nextCycleHint(status, scheduleEnabled)}
                </span>
              }
            />
            <SettingRow
              label="上次完成"
              value={
                status?.last_cycle_at ? (
                  <span className="font-mono text-xs">
                    {new Date(status.last_cycle_at).toLocaleString("zh-CN")}
                  </span>
                ) : (
                  <span className="text-muted-foreground">尚未执行</span>
                )
              }
            />
            <SettingRow
              label="上次分析"
              value={
                status?.last_analysis_total ? (
                  <span className="font-mono text-xs">
                    {status.last_analysis_completed}/{status.last_analysis_total} 品种
                  </span>
                ) : (
                  <span className="text-muted-foreground">—</span>
                )
              }
            />
            {status?.last_data_fetch && (
              <SettingRow
                label="上次数据拉取"
                value={
                  <span className="text-xs">
                    日历 {status.last_data_fetch.calendar_events} · 资讯{" "}
                    {status.last_data_fetch.news_items} · K线{" "}
                    {status.last_data_fetch.klines_symbols} 品种
                  </span>
                }
              />
            )}
            {status?.last_error && (
              <p className="mt-2 text-xs text-down">{status.last_error}</p>
            )}
            {status?.cycle_in_progress && (
              <p className="text-xs text-primary">定时周期执行中，请稍候…</p>
            )}
          </div>
        </div>

        <div className="border-t border-border pt-4">
          <p className="mb-2 text-xs font-medium text-muted-foreground">手动触发</p>
          <div className="flex flex-wrap gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={acting || !!status?.cycle_in_progress}
              onClick={() =>
                runAction(async () => {
                  const r = await api.triggerDataFetch();
                  setActionMsg(
                    `数据拉取完成：日历 ${r.calendar_events} 条，资讯 ${r.news_items} 条，K线 ${r.klines_symbols} 品种`
                  );
                })
              }
            >
              立即拉取数据
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={acting || !!status?.cycle_in_progress}
              onClick={() =>
                runAction(async () => {
                  const r = await api.triggerComprehensiveAnalysis();
                  setActionMsg(`全面分析已启动（${r.total} 个品种，含数据拉取）`);
                })
              }
            >
              全面分析全部品种
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={acting || !!status?.cycle_in_progress}
              onClick={() =>
                runAction(async () => {
                  const r = await api.triggerBatchAnalysis({ trigger: "manual" });
                  setActionMsg(`仅分析任务已启动（${r.total} 个品种，不含数据拉取）`);
                })
              }
            >
              仅分析（不拉数据）
            </Button>
          </div>
          {actionMsg && <p className="mt-2 text-xs text-muted-foreground">{actionMsg}</p>}
        </div>
      </CardContent>
    </Card>
  );
}

function DimensionFactsDebug() {
  const currentSymbol = useAppStore((s) => s.currentSymbol);
  const [symbol, setSymbol] = useState(currentSymbol);

  const { data: facts, isLoading, refetch } = useQuery({
    queryKey: ["dimension-facts", symbol],
    queryFn: () => api.listDimensionFacts({ symbol, limit: 50 }),
  });

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">调试 · dimension_facts</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Input
            value={symbol}
            onChange={(e) => setSymbol(e.target.value)}
            placeholder="品种 symbol，如 RB0"
            className="max-w-[160px] font-mono text-sm"
          />
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            查询
          </Button>
          <span className="text-xs text-muted-foreground">
            共 {facts?.length ?? 0} 条有效事实
          </span>
        </div>
        <ScrollArea className="h-[240px] rounded-md border border-border">
          {isLoading ? (
            <div className="p-3">
              <PanelSkeleton rows={5} />
            </div>
          ) : facts && facts.length > 0 ? (
            <div className="divide-y divide-border">
              {facts.map((f) => (
                <div key={f.id} className="space-y-1 px-3 py-2.5">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="secondary" className="text-[10px]">
                      {dimensionLabel(f.dimension_code)}
                    </Badge>
                    <span className="font-mono text-[10px] text-muted-foreground">{f.symbol}</span>
                    <span className="text-[10px] text-muted-foreground">
                      {new Date(f.created_at).toLocaleString("zh-CN")}
                    </span>
                  </div>
                  <p className="text-sm text-foreground">{f.fact}</p>
                  {f.source_report_id && (
                    <p className="font-mono text-[10px] text-muted-foreground">
                      report: {f.source_report_id.slice(0, 8)}…
                    </p>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <p className="p-3 text-sm text-muted-foreground">暂无 dimension_facts 记录</p>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

function UnclassifiedNewsPanel() {
  const queryClient = useQueryClient();
  const { data: items, isLoading, refetch } = useQuery({
    queryKey: ["unclassified-news"],
    queryFn: () => api.listUnclassifiedNews(40),
    refetchInterval: 120_000,
  });

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-semibold">未分类资讯队列</CardTitle>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" className="h-7 text-xs" onClick={() => refetch()}>
            刷新
          </Button>
          <Button
            variant="outline"
            size="sm"
            className="h-7 text-xs"
            disabled={!items?.length}
            onClick={async () => {
              if (!items?.length) return;
              await api.reclassifyNews({ news_ids: items.map((n) => n.id) });
              refetch();
              queryClient.invalidateQueries({ queryKey: ["unclassified-news"] });
            }}
          >
            LLM 重分类
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[220px] rounded-md border border-border">
          {isLoading ? (
            <div className="p-3">
              <PanelSkeleton rows={4} />
            </div>
          ) : items && items.length > 0 ? (
            <div className="divide-y divide-border">
              {items.map((n: NewsRecord) => (
                <div key={n.id} className="space-y-0.5 px-3 py-2.5">
                  <p className="text-sm font-medium text-foreground">{n.title}</p>
                  <p className="line-clamp-2 text-xs text-muted-foreground">{n.summary}</p>
                  <p className="text-[10px] text-muted-foreground">
                    {new Date(n.display_time).toLocaleString("zh-CN")} · {n.source}
                  </p>
                </div>
              ))}
            </div>
          ) : (
            <p className="p-3 text-sm text-muted-foreground">暂无未分类资讯</p>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}

function RuntimeStatusPanel() {
  const { data: runtime, refetch } = useQuery({
    queryKey: ["runtime-status"],
    queryFn: () => api.getRuntimeStatus(),
    refetchInterval: 15_000,
  });

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-semibold">运行时 · 行情与回填</CardTitle>
        <Button variant="outline" size="sm" className="h-7 text-xs" onClick={() => refetch()}>
          刷新
        </Button>
      </CardHeader>
      <CardContent>
        <SettingRow
          label="行情源"
          value={
            <span className="font-mono text-xs">{runtime?.feed_source ?? "—"}</span>
          }
        />
        <SettingRow
          label="订阅品种"
          value={
            <span className="font-mono text-xs">
              {runtime?.poll?.symbol_count ?? 0} 个 · {runtime?.poll?.interval ?? "—"}s
            </span>
          }
        />
        <SettingRow
          label="历史回填"
          value={
            runtime?.backfill?.running ? (
              <span className="text-xs">
                {runtime.backfill.completed}/{runtime.backfill.total}
                {runtime.backfill.current_symbol && ` · ${runtime.backfill.current_symbol}`}
              </span>
            ) : (
              <span className="text-xs text-muted-foreground">
                完成 {runtime?.backfill?.completed ?? 0}/{runtime?.backfill?.total ?? 0}
              </span>
            )
          }
        />
        {runtime?.backfill?.last_error && (
          <p className="mt-2 text-xs text-down">{runtime.backfill.last_error}</p>
        )}
      </CardContent>
    </Card>
  );
}

export function SettingsPage() {
  const queryClient = useQueryClient();
  const [searchParams] = useSearchParams();
  const [ollamaOk, setOllamaOk] = useState<boolean | null>(null);
  const [exportMsg, setExportMsg] = useState("");
  const {
    akshareOnline,
    jinshiOnline,
    jinshiCalendarReady,
    jinshiCalendarFetchedAt,
    jinshiCalendarEventCount,
    realtimeOnline,
    realtimeSource,
    statusMessage,
  } = useAppStore();

  const rawSection = searchParams.get("section");

  const { data: settings, isLoading } = useQuery({
    queryKey: ["app-settings"],
    queryFn: () => api.getSettings(),
  });

  if (!rawSection) {
    return <Navigate to="/settings?section=schedule" replace />;
  }

  const section = parseSettingsSection(rawSection);

  const renderSection = (id: SettingsSectionId) => {
    if (isLoading) {
      return <Skeleton className="h-48 rounded-lg" />;
    }

    switch (id) {
      case "appearance":
        return <AppearanceSettingsPanel />;
      case "schedule":
        return <ScheduleTaskPanel />;
      case "preferences":
        return <AppPreferencesEditor />;
      case "data":
        return (
          <>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">连接状态</CardTitle>
              </CardHeader>
              <CardContent>
                <SettingRow
                  label="AKShare K 线"
                  value={
                    akshareOnline ? (
                      <span className="text-up">已启用</span>
                    ) : (
                      <span className="text-down">不可用</span>
                    )
                  }
                />
                <SettingRow
                  label="K 线轮询"
                  value={
                    realtimeOnline ? (
                      <span className="font-mono">
                        {settings?.realtime_poll_interval ?? 5}s · {realtimeSource}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">未启动</span>
                    )
                  }
                />
                <SettingRow
                  label="金十资讯"
                  value={
                    jinshiOnline ? (
                      <span className="text-up">已连接</span>
                    ) : (
                      <span className="text-down">离线</span>
                    )
                  }
                />
                <SettingRow
                  label="金十财经日历"
                  value={
                    jinshiCalendarReady ? (
                      <span className="text-up">可用 · {jinshiCalendarEventCount} 条缓存</span>
                    ) : (
                      <span className="text-down">未拉取</span>
                    )
                  }
                />
                <SettingRow
                  label="日历上次更新"
                  value={
                    jinshiCalendarFetchedAt ? (
                      <span className="font-mono text-xs">
                        {new Date(jinshiCalendarFetchedAt).toLocaleString("zh-CN")}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )
                  }
                />
                <SettingRow
                  label="核心品种数"
                  value={
                    <span className="font-mono text-xs">{settings?.core_product_count ?? "—"}</span>
                  }
                />
                {statusMessage && (
                  <p className="mt-2 text-xs leading-relaxed text-muted-foreground">{statusMessage}</p>
                )}
              </CardContent>
            </Card>
            <RuntimeStatusPanel />
          </>
        );
      case "llm":
        return (
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-semibold">模型与凭据</CardTitle>
            </CardHeader>
            <CardContent>
              <SettingRow
                label="默认提供商"
                value={<span className="font-mono">{settings?.default_llm_provider ?? "—"}</span>}
              />
              <SettingRow
                label="已配置"
                value={
                  settings?.llm_providers.length ? (
                    <span className="font-mono">{settings.llm_providers.join(", ")}</span>
                  ) : (
                    <span className="text-down">未配置 API Key</span>
                  )
                }
              />
              <SettingRow
                label="资讯 LLM 分类"
                value={
                  <span className="font-mono">
                    {settings?.news_classify_enabled ? "开启" : "关闭"}
                    {settings?.news_classify_enabled ? ` · batch ${settings.news_classify_batch}` : ""}
                  </span>
                }
              />
              {settings?.llm_keys_masked?.map(([name, masked]) => (
                <SettingRow
                  key={name}
                  label={`Key · ${name}`}
                  value={<span className="font-mono text-xs">{masked}</span>}
                />
              ))}
              {settings?.ollama_configured && (
                <SettingRow
                  label="Ollama"
                  value={
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs"
                      onClick={async () => {
                        try {
                          setOllamaOk(await api.probeOllama());
                        } catch {
                          setOllamaOk(false);
                        }
                      }}
                    >
                      {ollamaOk === null ? "检测连通" : ollamaOk ? "在线" : "离线"}
                    </Button>
                  }
                />
              )}
              {settings?.llm_last_errors &&
                Object.entries(settings.llm_last_errors).map(([name, err]) => (
                  <p key={name} className="text-xs text-down">
                    {name}: {err.slice(0, 100)}
                  </p>
                ))}
              <div className="pt-3">
                <Button variant="outline" size="sm" className="h-8 text-xs" asChild>
                  <Link to="/setup">配置 LLM API Key</Link>
                </Button>
              </div>
              <SettingRow
                label="凭据加密"
                value={
                  settings?.encryption_configured ? (
                    <span className="text-up">ENCRYPTION_KEY 已配置</span>
                  ) : (
                    <span className="text-muted-foreground">未配置</span>
                  )
                }
              />
            </CardContent>
          </Card>
        );
      case "storage":
        return (
          <>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">数据库</CardTitle>
              </CardHeader>
              <CardContent>
                <SettingRow
                  label="路径"
                  value={
                    <span className="max-w-[240px] truncate font-mono text-xs">
                      {settings?.database_path}
                    </span>
                  }
                />
                <SettingRow
                  label="基础配置"
                  value={
                    <span className="max-w-[240px] truncate font-mono text-xs">
                      {settings?.preferences_path ?? "data/user_preferences.json"}
                    </span>
                  }
                />
                <SettingRow
                  label="后端"
                  value={
                    <span className="font-mono text-xs">{settings?.database_backend ?? "sqlite"}</span>
                  }
                />
                <SettingRow
                  label="行情源"
                  value={
                    <span className="font-mono text-xs">{settings?.market_feed ?? "akshare_poll"}</span>
                  }
                />
                <SettingRow
                  label="异动检测"
                  value={
                    settings?.anomaly_enabled ? (
                      <span className="font-mono text-xs">
                        开启 · {settings.anomaly_price_pct}% / {settings.anomaly_window_secs}s
                      </span>
                    ) : (
                      <span className="text-muted-foreground">关闭</span>
                    )
                  }
                />
                <SettingRow
                  label="回填范围"
                  value={
                    <span className="font-mono text-xs">
                      日K {settings?.backfill_days_daily ?? 120}d · 1m{" "}
                      {settings?.backfill_days_minute ?? 5}d
                    </span>
                  }
                />
              </CardContent>
            </Card>
            <div className="flex flex-wrap gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  const s = await api.reloadConfig();
                  queryClient.setQueryData(["app-settings"], s);
                }}
              >
                重载 .env（金十等）
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  const sym = useAppStore.getState().currentSymbol.toLowerCase();
                  const csv = await api.exportKlinesCsv({ symbol: sym, interval: "1d", limit: 500 });
                  const blob = new Blob([csv], { type: "text/csv" });
                  const a = document.createElement("a");
                  a.href = URL.createObjectURL(blob);
                  a.download = `${sym}-1d.csv`;
                  a.click();
                  setExportMsg(`已导出 ${sym} 日K`);
                }}
              >
                导出 K 线 CSV
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  const csv = await api.exportReportsCsv({ limit: 100 });
                  const blob = new Blob([csv], { type: "text/csv" });
                  const a = document.createElement("a");
                  a.href = URL.createObjectURL(blob);
                  a.download = "reports.csv";
                  a.click();
                  setExportMsg("已导出报告 CSV");
                }}
              >
                导出报告 CSV
              </Button>
            </div>
            {exportMsg && <p className="text-xs text-muted-foreground">{exportMsg}</p>}
          </>
        );
      case "simulation":
        return <SimulationRulesPanel />;
      case "debug":
        return (
          <>
            <UnclassifiedNewsPanel />
            <DimensionFactsDebug />
          </>
        );
      default:
        return null;
    }
  };

  return (
    <SettingsSectionShell>{renderSection(section)}</SettingsSectionShell>
  );
}
