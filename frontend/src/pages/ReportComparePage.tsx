import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "@/api/client";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { Skeleton } from "@/components/ui/skeleton";
import { DimensionSummary } from "@/features/analysis/DimensionSummary";
import { getFuturesProduct } from "@/data/futures";
import { triggerLabel } from "@/data/calendar";
import type { AnalysisReport } from "@/types";

function reportTitle(r: AnalysisReport) {
  const name = getFuturesProduct(r.symbol)?.name;
  return `${r.symbol.toUpperCase()}${name ? ` · ${name}` : ""}`;
}

function diffDimensions(a?: Record<string, string[]>, b?: Record<string, string[]>) {
  const keys = new Set([...Object.keys(a ?? {}), ...Object.keys(b ?? {})]);
  const rows: Array<{ key: string; left: string[]; right: string[]; changed: boolean }> = [];
  keys.forEach((key) => {
    const left = a?.[key] ?? [];
    const right = b?.[key] ?? [];
    const changed = JSON.stringify(left) !== JSON.stringify(right);
    rows.push({ key, left, right, changed });
  });
  return rows.sort((x, y) => x.key.localeCompare(y.key));
}

export function ReportComparePage() {
  const [symbolFilter, setSymbolFilter] = useState("");
  const [leftId, setLeftId] = useState("");
  const [rightId, setRightId] = useState("");

  const {
    data: reports,
    isLoading: reportsLoading,
    isError: reportsError,
    error: reportsErr,
    refetch: refetchReports,
  } = useQuery({
    queryKey: ["reports-compare"],
    queryFn: () => api.listReports({ limit: 100 }),
  });

  const filtered = useMemo(() => {
    if (!reports) return [];
    const sym = symbolFilter.trim().toLowerCase();
    if (!sym) return reports;
    return reports.filter((r) => r.symbol.toLowerCase().includes(sym));
  }, [reports, symbolFilter]);

  const { data: left, isLoading: leftLoading } = useQuery({
    queryKey: ["report", leftId],
    queryFn: () => api.getReport(leftId),
    enabled: !!leftId,
  });

  const { data: right, isLoading: rightLoading } = useQuery({
    queryKey: ["report", rightId],
    queryFn: () => api.getReport(rightId),
    enabled: !!rightId,
  });

  const dimDiff = useMemo(
    () => diffDimensions(left?.dimension_summary ?? undefined, right?.dimension_summary ?? undefined),
    [left, right]
  );

  const compareLoading = (!!leftId && leftLoading) || (!!rightId && rightLoading);

  return (
    <div className="page-scroll">
      <div className="page-inner space-y-4">
        <div className="flex justify-end">
          <Button variant="outline" size="sm" asChild>
            <Link to="/reports">返回列表</Link>
          </Button>
        </div>

        {reportsLoading ? (
          <Skeleton className="h-10 w-full max-w-2xl rounded-md" />
        ) : reportsError ? (
          <Card>
            <CardContent className="space-y-3 pt-4">
              <p className="text-sm text-down">
                报告列表加载失败：
                {reportsErr instanceof Error ? reportsErr.message : "未知错误"}
              </p>
              <Button variant="outline" size="sm" onClick={() => refetchReports()}>
                重试
              </Button>
            </CardContent>
          </Card>
        ) : (
          <div className="flex flex-wrap gap-3">
            <Input
              placeholder="筛选品种…"
              value={symbolFilter}
              onChange={(e) => setSymbolFilter(e.target.value)}
              className="max-w-[180px]"
            />
            <NativeSelect value={leftId} onChange={(e) => setLeftId(e.target.value)}>
              <option value="">报告 A</option>
              {filtered.map((r) => (
                <option key={r.id} value={r.id}>
                  {reportTitle(r)} · {triggerLabel(r.trigger)}
                </option>
              ))}
            </NativeSelect>
            <NativeSelect value={rightId} onChange={(e) => setRightId(e.target.value)}>
              <option value="">报告 B</option>
              {filtered.map((r) => (
                <option key={r.id} value={r.id}>
                  {reportTitle(r)} · {triggerLabel(r.trigger)}
                </option>
              ))}
            </NativeSelect>
          </div>
        )}

        {compareLoading && (
          <div className="grid gap-4 lg:grid-cols-2">
            <Skeleton className="h-48 rounded-lg" />
            <Skeleton className="h-48 rounded-lg" />
          </div>
        )}

        {!compareLoading && left && right && (
          <>
            <div className="grid gap-4 lg:grid-cols-2">
              {[left, right].map((r) => (
                <Card key={r.id}>
                  <CardHeader>
                    <CardTitle>
                      {reportTitle(r)} · {triggerLabel(r.trigger)}
                    </CardTitle>
                    <p className="text-xs text-muted-foreground">
                      {new Date(r.created_at).toLocaleString("zh-CN")} · {r.prompt_version}
                    </p>
                  </CardHeader>
                  <CardContent>
                    <DimensionSummary summary={r.dimension_summary ?? undefined} />
                  </CardContent>
                </Card>
              ))}
            </div>

            <Card>
              <CardHeader>
                <CardTitle>维度差异</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {dimDiff.map(({ key, left: l, right: r, changed }) => (
                  <div
                    key={key}
                    className={`rounded-md border p-3 ${changed ? "border-primary/40 bg-primary/5" : "border-border"}`}
                  >
                    <p className="mb-2 text-xs font-medium uppercase text-muted-foreground">{key}</p>
                    <div className="grid gap-2 text-sm lg:grid-cols-2">
                      <ul className="list-disc pl-4">
                        {l.map((item) => (
                          <li key={item}>{item}</li>
                        ))}
                      </ul>
                      <ul className="list-disc pl-4">
                        {r.map((item) => (
                          <li key={item}>{item}</li>
                        ))}
                      </ul>
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>
          </>
        )}
      </div>
    </div>
  );
}
