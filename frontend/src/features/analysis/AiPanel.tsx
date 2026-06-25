import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowUp, Sparkles } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { getFuturesProduct } from "@/data/futures";
import {
  AnalysisTimeline,
  defaultTimelineSteps,
  type TimelineStage,
} from "@/components/AnalysisTimeline";
import { DimensionSummary, type DimensionSummaryData } from "./DimensionSummary";

function stripJsonFence(text: string): string {
  const lower = text.toLowerCase();
  const start = lower.indexOf("```json");
  if (start === -1) return text;
  const rest = text.slice(start + 7);
  const endRel = rest.indexOf("```");
  if (endRel === -1) return text;
  const before = text.slice(0, start).trim();
  const after = rest.slice(endRel + 3).trim();
  return [before, after].filter(Boolean).join("\n\n");
}

async function readStream(
  reader: ReadableStreamDefaultReader<Uint8Array>,
  onData: (obj: Record<string, unknown>) => void
) {
  const decoder = new TextDecoder();
  let buffer = "";
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";
    for (const line of lines) {
      if (line.startsWith("data:")) {
        try {
          onData(JSON.parse(line.slice(5).trim()));
        } catch {
          /* ignore */
        }
      }
    }
  }
}

export function AiPanel() {
  const currentSymbol = useAppStore((s) => s.currentSymbol);
  const currentProduct = getFuturesProduct(currentSymbol);
  const displayName = currentProduct?.name || currentSymbol;
  const queryClient = useQueryClient();
  const [streaming, setStreaming] = useState(false);
  const [followupStreaming, setFollowupStreaming] = useState(false);
  const [streamText, setStreamText] = useState("");
  const [followupText, setFollowupText] = useState("");
  const [pendingQuestion, setPendingQuestion] = useState("");
  const [streamDimensionSummary, setStreamDimensionSummary] = useState<DimensionSummaryData | null>(
    null
  );
  const [stage, setStage] = useState<TimelineStage | undefined>();
  const [prompt, setPrompt] = useState("");

  const { data: reports } = useQuery({
    queryKey: ["reports", currentSymbol],
    queryFn: () => api.listReports({ symbol: currentSymbol, limit: 5 }),
  });

  const todayReport = reports?.[0];

  const { data: followups } = useQuery({
    queryKey: ["followups", todayReport?.id],
    queryFn: () => api.listFollowups({ report_id: todayReport!.id, limit: 30 }),
    enabled: Boolean(todayReport?.id),
  });

  async function handleFullAnalysis() {
    setStreaming(true);
    setStreamText("");
    setStreamDimensionSummary(null);
    setStage("thinking");
    try {
      const reader = await api.streamAnalysis(currentSymbol, "manual");
      await readStream(reader, (obj) => {
        if (obj.text) {
          setStreamText((t) => t + String(obj.text));
          setStage("edit");
        }
        if (obj.status === "ok") {
          setStage("done");
          if (obj.dimension_summary) {
            setStreamDimensionSummary(obj.dimension_summary as DimensionSummaryData);
          }
          void queryClient.invalidateQueries({ queryKey: ["reports", currentSymbol] });
        }
      });
    } catch {
      /* ignore */
    } finally {
      setStreaming(false);
    }
  }

  async function handleFollowup() {
    if (!todayReport || !prompt.trim()) return;
    const question = prompt.trim();
    setFollowupStreaming(true);
    setFollowupText("");
    setPendingQuestion(question);
    setPrompt("");
    try {
      const reader = await api.streamFollowup(todayReport.id, question);
      await readStream(reader, (obj) => {
        if (obj.text) {
          setFollowupText((t) => t + String(obj.text));
        }
        if (obj.status === "ok") {
          void queryClient.invalidateQueries({ queryKey: ["followups", todayReport.id] });
          setFollowupText("");
          setPendingQuestion("");
        }
      });
    } catch {
      setPendingQuestion("");
      /* ignore */
    } finally {
      setFollowupStreaming(false);
    }
  }

  async function handleSubmit() {
    const question = prompt.trim();
    if (question && todayReport) {
      await handleFollowup();
      return;
    }
    await handleFullAnalysis();
  }

  const busy = streaming || followupStreaming;
  const dimensionSummary =
    streaming ? streamDimensionSummary : todayReport?.dimension_summary ?? streamDimensionSummary;

  const rawText = streaming ? streamText : todayReport?.content;
  const displayText = rawText ? stripJsonFence(rawText) : undefined;

  const submitLabel = prompt.trim() && todayReport ? "追问" : streaming ? "分析中…" : "生成报告";

  return (
    <div className="flex flex-col gap-3">
      {/* Copilot 输入区 */}
      <Card>
        <CardHeader className="flex-row items-center gap-2 space-y-0 pb-0 pt-4">
          <Sparkles className="h-4 w-4 text-primary" />
          <CardTitle className="text-sm font-semibold">Copilot</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 pt-3">
          <div className="rounded-md border border-border bg-muted/20">
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder={
                todayReport
                  ? `基于最新报告追问，例如：${displayName} 库存变化对价格有何影响？`
                  : `分析 ${displayName} 主力的当前趋势与关键支撑阻力…`
              }
              rows={3}
              className="w-full resize-none bg-transparent px-3 py-2.5 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none"
              disabled={busy}
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                  e.preventDefault();
                  void handleSubmit();
                }
              }}
            />
            <div className="flex items-center justify-between border-t border-border px-2 py-1.5">
              <p className="text-[11px] text-muted-foreground">
                {todayReport
                  ? "有报告时可追问；空输入将重新生成完整分析。"
                  : "AI 可能出错，请自行核实分析结论。"}
              </p>
              <Button
                size="sm"
                className="h-7 gap-1 px-2.5"
                onClick={() => void handleSubmit()}
                disabled={busy}
              >
                {busy ? "处理中…" : <ArrowUp className="h-3.5 w-3.5" />}
                {!busy && <span className="sr-only">{submitLabel}</span>}
              </Button>
            </div>
          </div>
          {(followups && followups.length > 0) || followupText || pendingQuestion ? (
            <ScrollArea className="max-h-[200px] rounded-md border border-border bg-background px-3 py-2">
              <div className="space-y-3">
                {followups?.map((f) => (
                  <div key={f.id} className="space-y-1">
                    <p className="text-xs font-medium text-foreground">问：{f.question}</p>
                    <p className="whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground">
                      {f.answer}
                    </p>
                  </div>
                ))}
                {(pendingQuestion || followupText) && (
                  <div className="space-y-1">
                    {pendingQuestion && (
                      <p className="text-xs font-medium text-foreground">问：{pendingQuestion}</p>
                    )}
                    <p className="whitespace-pre-wrap text-xs leading-relaxed text-foreground">
                      {followupText || "思考中…"}
                    </p>
                  </div>
                )}
              </div>
            </ScrollArea>
          ) : null}
        </CardContent>
      </Card>

      {/* 今日报告 */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-semibold">今日报告</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {todayReport && !streaming && (
            <div className="flex items-center gap-2">
              <Badge variant="secondary">{todayReport.trigger}</Badge>
              <span className="text-xs text-muted-foreground">
                {new Date(todayReport.created_at).toLocaleString("zh-CN")}
              </span>
            </div>
          )}
          {dimensionSummary && (
            <DimensionSummary summary={dimensionSummary} compact />
          )}
          {displayText ? (
            <p className="whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground">
              {displayText.slice(0, 600)}
              {displayText.length > 600 ? "…" : ""}
            </p>
          ) : streaming ? (
            <p className="min-h-[80px] text-sm text-muted-foreground">
              {streamText ? stripJsonFence(streamText) : "等待大模型响应…"}
            </p>
          ) : (
            <p className="text-sm text-muted-foreground">暂无报告，使用 Copilot 生成分析。</p>
          )}
        </CardContent>
      </Card>

      {(streaming || stage) && <AnalysisTimeline steps={defaultTimelineSteps(stage)} />}

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-semibold">历史报告</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <ScrollArea className="h-[180px] px-6 pb-4">
            {reports && reports.length > 0 ? (
              <div className="space-y-3">
                {reports.map((r) => (
                  <div key={r.id} className="border-b border-border pb-3 last:border-0">
                    <div className="flex items-center justify-between">
                      <span className="text-sm">{getFuturesProduct(r.symbol)?.name || r.symbol}</span>
                      <Badge variant="outline">{r.trigger}</Badge>
                    </div>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {new Date(r.created_at).toLocaleString("zh-CN")} · {r.provider}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">暂无历史报告</p>
            )}
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  );
}
