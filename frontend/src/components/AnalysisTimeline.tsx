import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

export type TimelineStage = "thinking" | "grep" | "read" | "edit" | "done";

interface TimelineStep {
  stage: TimelineStage;
  label: string;
  detail: string;
  active?: boolean;
}

const STAGE_LABEL: Record<TimelineStage, string> = {
  thinking: "思考",
  grep: "检索",
  read: "读取",
  edit: "分析",
  done: "完成",
};

export function AnalysisTimeline({ steps }: { steps: TimelineStep[] }) {
  return (
    <Card className="border-border bg-card">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">分析时间线</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {steps.map((step, idx) => (
          <div key={idx} className="flex items-start gap-3">
            <span
              className={cn(
                "pill-" + step.stage,
                "inline-flex min-w-[56px] justify-center rounded-full px-2.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
                step.active && "animate-pulse"
              )}
            >
              {STAGE_LABEL[step.stage]}
            </span>
            <div className="min-w-0 flex-1">
              <div className="text-sm font-medium">{step.label}</div>
              <div className="text-xs text-muted-foreground">{step.detail}</div>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

export function defaultTimelineSteps(activeStage?: TimelineStage): TimelineStep[] {
  const stages: TimelineStage[] = ["thinking", "grep", "read", "edit", "done"];
  const details: Record<TimelineStage, string> = {
    thinking: "正在汇总近 60 日 K 线与指标",
    grep: "拉取金十财经日历与持仓数据",
    read: "解析资金流向与基差",
    edit: "调用大模型生成走势研判",
    done: "报告已生成",
  };
  const activeIdx = activeStage ? stages.indexOf(activeStage) : -1;
  return stages.map((s, i) => ({
    stage: s,
    label: STAGE_LABEL[s],
    detail: details[s],
    active: i === activeIdx,
  }));
}
