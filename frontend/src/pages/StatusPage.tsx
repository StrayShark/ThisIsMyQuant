import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import type { StatusDashboard } from "@/types";
import { Button } from "@/components/ui/button";

function BoolBadge({ ok, label }: { ok: boolean; label: string }) {
  return (
    <Badge variant={ok ? "up" : "secondary"} className="text-[10px]">
      {label} · {ok ? "正常" : "离线"}
    </Badge>
  );
}

export function StatusPage() {
  const { data, isLoading, isError, error, refetch } = useQuery<StatusDashboard>({
    queryKey: ["status-dashboard"],
    queryFn: () => api.getStatusDashboard(),
    refetchInterval: 15_000,
  });

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <div className="mb-4 flex justify-end">
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            刷新
          </Button>
        </div>

        {isLoading ? (
          <Skeleton className="h-64 rounded-lg" />
        ) : isError ? (
          <Card>
            <CardContent className="space-y-3 pt-4">
              <p className="text-sm text-down">
                加载失败：{error instanceof Error ? error.message : "未知错误"}
              </p>
              <Button variant="outline" size="sm" onClick={() => refetch()}>
                重试
              </Button>
            </CardContent>
          </Card>
        ) : data ? (
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            <Card>
              <CardHeader>
                <CardTitle>Prompt · 行情</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2 text-sm">
                <p>
                  分析版本：<span className="font-mono">{data.prompt_version}</span>
                </p>
                <p>
                  行情源：
                  <span className="font-mono">{data.runtime.feed_source ?? "—"}</span>
                </p>
                <p>
                  报价缓存：
                  <span className="font-mono">
                    {data.quote_status.quote_count} 个
                    {data.quote_status.stale_count > 0
                      ? ` · ${data.quote_status.stale_count} 个过期`
                      : " · 新鲜"}
                  </span>
                </p>
                <p>
                  最大延迟：
                  <span className="font-mono">
                    {data.quote_status.max_age_secs != null
                      ? `${data.quote_status.max_age_secs}s`
                      : "—"}
                  </span>
                </p>
                <p>
                  定时：每 {data.runtime.schedule.interval_hours}h
                  {data.runtime.schedule.last_cycle_at &&
                    ` · 上次 ${new Date(data.runtime.schedule.last_cycle_at).toLocaleString("zh-CN")}`}
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>大模型</CardTitle>
              </CardHeader>
              <CardContent className="flex flex-wrap gap-2">
                {Object.entries(data.llm_health).map(([name, ok]) => (
                  <BoolBadge key={name} ok={ok} label={name} />
                ))}
                {Object.entries(data.llm_last_errors).map(([name, err]) => (
                  <p key={name} className="w-full text-xs text-down">
                    {name}: {err.slice(0, 120)}
                  </p>
                ))}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>扩展数据源</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2 text-sm">
                {data.questdb_configured ? (
                  <BoolBadge ok={data.questdb_online} label="QuestDB" />
                ) : (
                  <p className="text-xs text-muted-foreground">QuestDB 未配置（可选）</p>
                )}
                <p className="text-xs text-muted-foreground">
                  {(data.overseas.message as string) ?? "海外期货参考源"}
                </p>
                {Array.isArray(data.overseas.symbols) && data.overseas.symbols.length > 0 && (
                  <p className="font-mono text-xs text-muted-foreground">
                    {data.overseas.symbols
                      .slice(0, 6)
                      .map((s) => (typeof s === "object" && s ? (s as { symbol?: string }).symbol : null))
                      .filter(Boolean)
                      .join(" / ")}
                  </p>
                )}
              </CardContent>
            </Card>

            <Card className="md:col-span-2">
              <CardHeader>
                <CardTitle>批量分析任务</CardTitle>
              </CardHeader>
              <CardContent className="text-sm">
                {data.batch_job.running ? (
                  <p>
                    进行中 {data.batch_job.completed}/{data.batch_job.total}
                    {data.batch_job.current_symbol && ` · ${data.batch_job.current_symbol}`}
                  </p>
                ) : (
                  <p className="text-muted-foreground">空闲</p>
                )}
                {data.batch_job.errors.length > 0 && (
                  <ul className="mt-2 list-disc pl-4 text-xs text-down">
                    {data.batch_job.errors.map((e) => (
                      <li key={e}>{e}</li>
                    ))}
                  </ul>
                )}
              </CardContent>
            </Card>
          </div>
        ) : (
          <Card>
            <CardContent className="pt-4">
              <p className="text-sm text-muted-foreground">暂无状态数据</p>
              <Button variant="outline" size="sm" className="mt-3" onClick={() => refetch()}>
                刷新
              </Button>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
