import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { StockScreenerBuilder, type ScreenerCriteria } from "./components/StockScreenerBuilder";
import { ScreenResultTable } from "./components/ScreenResultTable";
import { DataQualityBadge } from "./components/DataQualityBadge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Sparkles, Trash2 } from "lucide-react";
import type { StockScreenerResultView, StockSymbolSnapshot, StockScreenTemplate } from "@/types";

interface StockScreenerProps {
  onSelectStock?: (row: StockSymbolSnapshot) => void;
}

export function StockScreener({ onSelectStock }: StockScreenerProps) {
  const queryClient = useQueryClient();
  const [result, setResult] = useState<StockScreenerResultView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<string | null>(null);

  const { data: templates } = useQuery({
    queryKey: ["stock-screen-templates"],
    queryFn: () => api.listStockScreenTemplates(),
  });

  const runMutation = useMutation({
    mutationFn: (params: { criteria: ScreenerCriteria; name: string }) =>
      api.runStockScreener({
        criteria_json: JSON.stringify(params.criteria),
        name: params.name || undefined,
      }),
    onSuccess: (data) => {
      setResult(data);
      setError(null);
      setSummary(null);
    },
    onError: (err: Error) => {
      setError(err.message);
    },
  });

  const saveMutation = useMutation({
    mutationFn: (params: { criteria: ScreenerCriteria; name: string }) =>
      api.saveStockScreen({
        criteria_json: JSON.stringify(params.criteria),
        name: params.name || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stock-screen-templates"] });
    },
  });

  const deleteTemplateMutation = useMutation({
    mutationFn: (id: string) => api.deleteStockScreenTemplate(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["stock-screen-templates"] }),
  });

  const summaryMutation = useMutation({
    mutationFn: () => {
      if (!result) throw new Error("no result");
      const rows = result.rows.slice(0, 20).map((r) => `${r.name}(${r.ts_code})`).join(", ");
      return api.summarizeStockScreen({
        criteria_json: result.criteria_json,
        result_summary: `共 ${result.count} 只，行情 ${result.trade_date ?? "--"}，财报 ${result.report_period ?? "--"}。样本：${rows}`,
      });
    },
    onSuccess: (report) => setSummary(report.content),
  });

  const handleRun = (criteria: ScreenerCriteria, name: string) => {
    runMutation.mutate({ criteria, name });
  };

  const handleSave = (criteria: ScreenerCriteria, name: string) => {
    saveMutation.mutate({ criteria, name });
  };

  const handleLoadTemplate = (template: StockScreenTemplate) => {
    try {
      const criteria = JSON.parse(template.criteria_json) as ScreenerCriteria;
      runMutation.mutate({ criteria, name: template.name });
    } catch {
      setError("模板条件解析失败");
    }
  };

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-12">
        <div className="flex flex-col gap-4 lg:col-span-4">
          {templates && templates.length > 0 && (
            <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
              <h3 className="mb-2 text-sm font-medium">已保存模板</h3>
              <div className="space-y-1">
                {templates.map((t) => (
                  <div
                    key={t.id}
                    className="flex items-center justify-between rounded-md px-2 py-1 hover:bg-muted/60"
                  >
                    <button
                      type="button"
                      onClick={() => handleLoadTemplate(t)}
                      className="flex-1 text-left text-xs text-primary hover:underline"
                    >
                      {t.name}
                    </button>
                    <button
                      type="button"
                      onClick={() => deleteTemplateMutation.mutate(t.id)}
                      className="text-muted-foreground hover:text-rose-500"
                    >
                      <Trash2 className="h-3 w-3" />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
            <h3 className="mb-3 text-sm font-medium">筛选条件</h3>
            <StockScreenerBuilder
              onRun={handleRun}
              onSave={handleSave}
              isRunning={runMutation.isPending || saveMutation.isPending}
            />
          </div>
        </div>

        <div className="flex flex-col gap-4 lg:col-span-8">
          <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
            <div className="mb-3 flex items-center justify-between">
              <h3 className="text-sm font-medium">筛选结果</h3>
              {result && (
                <div className="flex items-center gap-2">
                  <div className="text-xs text-muted-foreground">
                    共 {result.count} 条
                    {result.trade_date && ` · 行情 ${result.trade_date}`}
                    {result.report_period && ` · 财报 ${result.report_period}`}
                  </div>
                  <button
                    type="button"
                    onClick={() => summaryMutation.mutate()}
                    disabled={summaryMutation.isPending}
                    className="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-1 text-[10px] hover:bg-accent disabled:opacity-50"
                  >
                    <Sparkles className="h-3 w-3" />
                    {summaryMutation.isPending ? "生成中…" : "AI 总结"}
                  </button>
                </div>
              )}
            </div>
            {error && <div className="mb-2 text-xs text-rose-500">{error}</div>}
            <ScreenResultTable rows={result?.rows ?? []} onSelect={onSelectStock} />
          </div>

          {summary && (
            <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
              <div className="mb-2 flex items-center justify-between">
                <h3 className="text-sm font-medium">AI 筛选总结</h3>
                <button
                  type="button"
                  onClick={() => setSummary(null)}
                  className="text-xs text-muted-foreground hover:text-foreground"
                >
                  收起
                </button>
              </div>
              <ScrollArea className="h-40 w-full rounded-md bg-muted/40 p-3 text-xs leading-relaxed whitespace-pre-wrap">
                {summary}
              </ScrollArea>
            </div>
          )}

          {result && (
            <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
              <div className="mb-2 text-sm font-medium">数据质量</div>
              <DataQualityBadge
                status={result.count > 0 ? "available" : "pending"}
                message={result.count > 0 ? `命中 ${result.count} 只` : "无匹配结果"}
                lastSuccessAt={result.trade_date}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
