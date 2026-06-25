import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { PageHeader } from "@/components/PageHeader";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useAppStore } from "@/app/store";
import { api } from "@/api/client";
import { dimensionLabel } from "@/data/dimensions";

function SettingRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-center justify-between border-b border-border py-3 last:border-0">
      <span className="text-sm text-foreground">{label}</span>
      <span className="text-sm text-muted-foreground">{value}</span>
    </div>
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
    <Card className="md:col-span-2 xl:col-span-3">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">调试 · dimension_facts</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Input
            value={symbol}
            onChange={(e) => setSymbol(e.target.value)}
            placeholder="品种 symbol，如 rb0"
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
            <p className="p-3 text-sm text-muted-foreground">加载中…</p>
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

export function SettingsPage() {
  const { akshareOnline, jinshiOnline, realtimeOnline, realtimeSource, statusMessage } =
    useAppStore();

  const { data: settings, isLoading, refetch } = useQuery({
    queryKey: ["app-settings"],
    queryFn: () => api.getSettings(),
  });

  const cronParts = settings?.daily_analysis_cron.split(" ") ?? ["0", "17"];
  const dailyTime = `${cronParts[1] ?? "17"}:${(cronParts[0] ?? "0").padStart(2, "0")}`;

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <PageHeader
          title="设置"
          description="读取 Rust 核心与 .env 配置。修改 .env 后需重启应用生效。"
        />

        {isLoading ? (
          <Skeleton className="h-48 rounded-lg" />
        ) : (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">数据源</CardTitle>
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
                  label="关注列表"
                  value={
                    <span className="max-w-[180px] truncate font-mono text-xs">
                      {(settings?.watchlist ?? []).join(", ")}
                    </span>
                  }
                />
                {statusMessage && (
                  <p className="mt-2 text-xs leading-relaxed text-muted-foreground">
                    {statusMessage}
                  </p>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">大模型</CardTitle>
              </CardHeader>
              <CardContent>
                <SettingRow
                  label="默认提供商"
                  value={
                    <span className="font-mono">{settings?.default_llm_provider ?? "—"}</span>
                  }
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
                      {settings?.news_classify_enabled
                        ? ` · batch ${settings.news_classify_batch}`
                        : ""}
                    </span>
                  }
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">分析调度</CardTitle>
              </CardHeader>
              <CardContent>
                <SettingRow
                  label="每日报告"
                  value={
                    <span className="font-mono">
                      {dailyTime}
                      {settings?.scheduler_daily_running ? " · 运行中" : ""}
                    </span>
                  }
                />
                <SettingRow
                  label="实时间隔"
                  value={
                    <span className="font-mono">
                      {settings?.realtime_analysis_interval ?? 300}s
                      {settings?.scheduler_realtime_running ? " · 运行中" : ""}
                    </span>
                  }
                />
                <SettingRow
                  label="数据库"
                  value={
                    <span className="max-w-[180px] truncate font-mono text-xs">
                      {settings?.database_path}
                    </span>
                  }
                />
              </CardContent>
            </Card>

            <DimensionFactsDebug />
          </div>
        )}

        <Button variant="outline" className="mt-8" onClick={() => refetch()}>
          刷新配置
        </Button>
      </div>
    </div>
  );
}
