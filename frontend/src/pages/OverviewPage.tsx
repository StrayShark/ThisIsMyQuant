import { SectorHeatmap } from "@/features/overview/SectorHeatmap";
import { LlmBiasColumn } from "@/features/overview/LlmBiasColumn";
import { MacroTimeline } from "@/features/overview/MacroTimeline";
import { ProductNewsFeed } from "@/features/overview/ProductNewsFeed";
import { ProfessionalWorkbench } from "@/features/overview/ProfessionalWorkbench";

export function OverviewPage() {
  return (
    <div className="page-scroll h-full">
      <div className="page-inner space-y-4 pb-6">
        <ProfessionalWorkbench />
        <SectorHeatmap />
        <LlmBiasColumn />
        <div className="grid gap-4 lg:grid-cols-2">
          <MacroTimeline />
          <ProductNewsFeed />
        </div>
      </div>
    </div>
  );
}
