import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import type { DatabaseTableStats } from "@/types";

export function LocalDatabasePage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const { data: summary, isLoading } = useQuery({
    queryKey: ["database-summary"],
    queryFn: () => api.getDatabaseSummary(),
  });

  const backup = useMutation({
    mutationFn: () => api.backupDatabase(),
    onSuccess: (path) => showToast(`已备份至 ${path}`),
    onError: (err: Error) => showToast(err.message),
  });

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-4xl px-6 py-6">
        <div className="mb-4">
          <h1 className="text-xl font-semibold">本地数据库</h1>
          <p className="text-sm text-muted-foreground">查看和管理本地 SQLite 数据资产</p>
        </div>

        <Card className="mb-4">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">存储概览</CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading || !summary ? (
              <div className="text-sm text-muted-foreground">加载中…</div>
            ) : (
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">数据库路径</span>
                  <span className="font-mono">{summary.path}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">总大小</span>
                  <span className="tabular-nums">{formatBytes(summary.total_size_bytes)}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">表数量</span>
                  <span className="tabular-nums">{summary.tables.length}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="mb-4">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">表空间统计</CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading || !summary ? (
              <div className="text-sm text-muted-foreground">加载中…</div>
            ) : (
              <div className="rounded-lg border">
                <table className="w-full text-sm">
                  <thead className="bg-muted/40">
                    <tr>
                      <th className="px-3 py-2 text-left">表名</th>
                      <th className="px-3 py-2 text-right">行数</th>
                      <th className="px-3 py-2 text-right">大小</th>
                      <th className="px-3 py-2 text-left">最近更新</th>
                    </tr>
                  </thead>
                  <tbody>
                    {summary.tables.map((t) => (
                      <TableRow key={t.name} table={t} />
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">维护操作</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex flex-wrap gap-2">
              <Button onClick={() => backup.mutate()} disabled={backup.isPending}>
                {backup.isPending ? "备份中…" : "立即备份"}
              </Button>
              <Button variant="outline" onClick={() => void queryClient.invalidateQueries({ queryKey: ["database-summary"] })}>
                刷新统计
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">备份文件保存在数据库同目录，可在设置中配置清理策略。</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function TableRow({ table }: { table: DatabaseTableStats }) {
  return (
    <tr className="border-t">
      <td className="px-3 py-2 font-mono text-xs">{table.name}</td>
      <td className="px-3 py-2 text-right tabular-nums">{table.row_count.toLocaleString()}</td>
      <td className="px-3 py-2 text-right tabular-nums">{formatBytes(table.size_bytes)}</td>
      <td className="px-3 py-2 text-xs text-muted-foreground">
        {table.last_updated ? new Date(table.last_updated).toLocaleString() : <Badge variant="outline">未知</Badge>}
      </td>
    </tr>
  );
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}
