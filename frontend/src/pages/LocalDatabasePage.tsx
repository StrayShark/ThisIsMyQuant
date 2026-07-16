import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { RefreshCw, Download, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { PageShell } from "@/components/layout/PageShell";
import { PageHeader } from "@/components/layout/PageHeader";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { DataDomainGrid } from "@/features/database/DataDomainGrid";
import { DataDomainTable } from "@/features/database/DataDomainTable";
import { formatBytes, formatDateTime } from "@/features/database/utils";

export function LocalDatabasePage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const [restorePath, setRestorePath] = useState("");

  const { data: summary, isLoading } = useQuery({
    queryKey: ["database-domain-summary"],
    queryFn: () => api.getDatabaseDomainSummary(),
  });

  const backup = useMutation({
    mutationFn: () => api.backupDatabase(),
    onSuccess: (path) => showToast(`已备份至 ${path}`),
    onError: (err: Error) => showToast(err.message),
  });

  const restore = useMutation({
    mutationFn: (path: string) => api.prepareDatabaseRestore(path),
    onSuccess: (message) => showToast(message),
    onError: (err: Error) => showToast(err.message),
  });

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ["database-domain-summary"] });
  };

  const domains = summary?.domains ?? [];

  return (
    <PageShell>
      <PageHeader title="本地数据库" description="管理本地 SQLite 数据资产">
        <Button onClick={() => backup.mutate()} disabled={backup.isPending}>
          <Download className="size-4" aria-hidden="true" />
          {backup.isPending ? "备份中…" : "立即备份"}
        </Button>
        <Button variant="outline" onClick={refresh} disabled={isLoading}>
          <RefreshCw
            className={isLoading ? "size-4 animate-spin" : "size-4"}
            aria-hidden="true"
          />
          刷新
        </Button>
      </PageHeader>

      <Card className="mb-6">
        <CardContent className="space-y-4 py-4">
          {isLoading || !summary ? (
            <div className="text-sm text-muted-foreground">加载中…</div>
          ) : (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <div>
                <p className="text-xs text-muted-foreground">数据库路径</p>
                <p className="truncate font-mono text-sm">{summary.path}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">总大小</p>
                <p className="text-sm font-semibold tabular-nums">
                  {formatBytes(summary.total_size_bytes)}
                </p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">数据域数量</p>
                <p className="text-sm font-semibold tabular-nums">{domains.length}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">最后更新时间</p>
                <p className="text-sm tabular-nums">{formatDateTime(summary.updated_at)}</p>
              </div>
            </div>
          )}
          <div className="flex flex-col gap-2 border-t border-border pt-4 md:flex-row md:items-center">
            <Input
              value={restorePath}
              onChange={(event) => setRestorePath(event.target.value)}
              placeholder="粘贴 .bak 备份文件路径，校验后生成恢复候选"
              className="font-mono text-xs"
            />
            <Button
              variant="outline"
              onClick={() => restore.mutate(restorePath)}
              disabled={!restorePath.trim() || restore.isPending}
              className="shrink-0"
            >
              <RotateCcw className="size-4" aria-hidden="true" />
              {restore.isPending ? "校验中…" : "准备恢复"}
            </Button>
          </div>
        </CardContent>
      </Card>

      <section className="mb-8">
        <h2 className="mb-4 text-sm font-semibold text-muted-foreground">数据域概览</h2>
        {isLoading ? (
          <div className="text-sm text-muted-foreground">加载中…</div>
        ) : (
          <DataDomainGrid domains={domains} />
        )}
      </section>

      <section>
        <h2 className="mb-4 text-sm font-semibold text-muted-foreground">数据域明细</h2>
        {isLoading ? (
          <div className="text-sm text-muted-foreground">加载中…</div>
        ) : (
          <DataDomainTable domains={domains} />
        )}
      </section>
    </PageShell>
  );
}
