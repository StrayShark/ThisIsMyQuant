import { useState } from "react";
import { PageShell } from "@/components/layout/PageShell";
import { PageHeader } from "@/components/layout/PageHeader";
import { AiTaskPanel } from "@/features/ai/AiTaskPanel";
import { AiReportCard } from "@/features/ai/AiReportCard";
import { cn } from "@/lib/utils";
import type { AiReportSummary } from "@/types";

export function AiPage() {
  const [report, setReport] = useState<AiReportSummary | null>(null);

  return (
    <PageShell>
      <PageHeader
        title="AI 分析"
        description="基于本地数据生成可解释的研究摘要"
      />

      <div
        className={cn(
          "grid gap-5",
          "grid-cols-1 lg:grid-cols-[380px_minmax(0,1fr)]"
        )}
      >
        <AiTaskPanel onReport={setReport} />
        <AiReportCard report={report} />
      </div>
    </PageShell>
  );
}
