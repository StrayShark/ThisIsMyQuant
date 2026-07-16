import { useState, useCallback } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Sparkles, Loader2, RefreshCw, Clock, AlertCircle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { cn } from "@/lib/utils";
import type { AiReportSummary, AiSummaryRequest, AiTaskType } from "@/types";

const TASK_OPTIONS: { value: AiTaskType; label: string; needsSymbol: boolean }[] = [
  { value: "market_summary", label: "市场摘要", needsSymbol: false },
  { value: "leaderboard_explain", label: "排行榜解释", needsSymbol: false },
  { value: "asset_brief", label: "标的速览", needsSymbol: true },
  { value: "watchlist_summary", label: "自选摘要", needsSymbol: false },
  { value: "position_risk", label: "持仓风险", needsSymbol: false },
  { value: "event_impact", label: "事件影响", needsSymbol: true },
  { value: "custom", label: "自定义", needsSymbol: false },
];

interface AiTaskPanelProps {
  onReport?: (report: AiReportSummary) => void;
  className?: string;
}

export function AiTaskPanel({ onReport, className }: AiTaskPanelProps) {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const [taskType, setTaskType] = useState<AiTaskType>("market_summary");
  const [targetSymbol, setTargetSymbol] = useState("");
  const [customPrompt, setCustomPrompt] = useState("");

  const { data: tasksResult, isLoading: tasksLoading } = useQuery({
    queryKey: ["ai-tasks"],
    queryFn: () => api.listAiTasks(),
  });

  const generateMutation = useMutation({
    mutationFn: (payload: AiSummaryRequest) => api.generateAiSummary(payload),
    onSuccess: (report) => {
      showToast("AI 摘要已生成");
      onReport?.(report);
      void queryClient.invalidateQueries({ queryKey: ["ai-tasks"] });
    },
    onError: (err: Error) => showToast(err.message || "生成失败"),
  });

  const handleGenerate = useCallback(() => {
    const payload: AiSummaryRequest = {
      task_type: taskType,
      target_symbol: taskType === "custom" ? null : targetSymbol || null,
      prompt: taskType === "custom" ? customPrompt || null : null,
    };
    generateMutation.mutate(payload);
  }, [taskType, targetSymbol, customPrompt, generateMutation]);

  const currentTask = TASK_OPTIONS.find((t) => t.value === taskType);
  const showSymbolInput = currentTask?.needsSymbol ?? false;
  const showPromptInput = taskType === "custom";

  return (
    <div className={className}>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Sparkles className="h-4 w-4 text-primary" />
            新建任务
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
            {TASK_OPTIONS.map((option) => (
              <Button
                key={option.value}
                type="button"
                variant={taskType === option.value ? "default" : "outline"}
                size="sm"
                className="h-8 text-xs"
                onClick={() => setTaskType(option.value)}
              >
                {option.label}
              </Button>
            ))}
          </div>

          {showSymbolInput && (
            <div>
              <label className="mb-1.5 block text-xs font-medium text-muted-foreground">
                目标标的
              </label>
              <Input
                placeholder="例如 RB0 或 600000.SH"
                value={targetSymbol}
                onChange={(e) => setTargetSymbol(e.target.value)}
                className="h-9"
              />
            </div>
          )}

          {showPromptInput && (
            <div>
              <label className="mb-1.5 block text-xs font-medium text-muted-foreground">
                自定义提示
              </label>
              <Textarea
                placeholder="输入你希望我分析的问题..."
                value={customPrompt}
                onChange={(e) => setCustomPrompt(e.target.value)}
                className="min-h-[80px] resize-none"
              />
            </div>
          )}

          <Button
            type="button"
            className="w-full"
            disabled={generateMutation.isPending}
            onClick={handleGenerate}
          >
            {generateMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                生成中...
              </>
            ) : (
              <>
                <Sparkles className="h-4 w-4" />
                生成摘要
              </>
            )}
          </Button>

          <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 p-2.5">
            <div className="flex items-start gap-2">
              <AlertCircle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
              <p className="text-[11px] leading-relaxed text-amber-600">
                仅供研究与复盘，不构成投资建议
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="mt-4">
        <CardHeader className="flex-row items-center justify-between pb-2">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Clock className="h-4 w-4 text-primary" />
            历史任务
          </CardTitle>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-7 gap-1 text-xs"
            onClick={() => queryClient.invalidateQueries({ queryKey: ["ai-tasks"] })}
            disabled={tasksLoading}
          >
            <RefreshCw className={cn("h-3 w-3", tasksLoading && "animate-spin")} />
            刷新
          </Button>
        </CardHeader>
        <CardContent>
          {tasksLoading ? (
            <p className="text-sm text-muted-foreground">加载中...</p>
          ) : !tasksResult?.tasks.length ? (
            <p className="text-sm text-muted-foreground">暂无历史任务</p>
          ) : (
            <div className="flex flex-col gap-2">
              {tasksResult.tasks.map((task) => {
                const label = TASK_OPTIONS.find((t) => t.value === task.task_type)?.label ?? task.task_type;
                return (
                  <div
                    key={task.id}
                    className="rounded-xl border border-border bg-muted/20 p-2.5 text-sm"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="font-medium text-foreground">{label}</span>
                      <TaskStatusBadge status={task.status} />
                    </div>
                    {task.target_symbol && (
                      <p className="mt-0.5 font-mono text-xs text-muted-foreground">
                        {task.target_symbol}
                      </p>
                    )}
                    <p className="mt-1 text-[11px] text-muted-foreground">
                      {new Date(task.created_at).toLocaleString("zh-CN")}
                    </p>
                    {task.error && (
                      <p className="mt-1 text-[11px] text-destructive">{task.error}</p>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function TaskStatusBadge({ status }: { status: string }) {
  const config: Record<string, { label: string; className: string }> = {
    pending: { label: "待处理", className: "bg-muted text-muted-foreground" },
    running: { label: "运行中", className: "bg-primary/20 text-primary" },
    done: { label: "完成", className: "bg-[var(--color-up-bg)] text-[var(--color-up)]" },
    error: { label: "失败", className: "bg-destructive/20 text-destructive" },
  };
  const { label, className } = config[status] ?? config.pending;
  return (
    <span className={`rounded-full px-2 py-0.5 text-[10px] font-medium ${className}`}>{label}</span>
  );
}
