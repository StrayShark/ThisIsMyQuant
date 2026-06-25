import { cn } from "@/lib/utils";
import { INDICATOR_REGISTRY, type IndicatorToggles } from "./types";

interface IndicatorToolbarProps {
  toggles: IndicatorToggles;
  onToggle: (id: keyof IndicatorToggles) => void;
}

export function IndicatorToolbar({ toggles, onToggle }: IndicatorToolbarProps) {
  return (
    <div className="flex flex-wrap items-center gap-1 border-b border-border/60 px-3 py-1.5">
      <span className="mr-1 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
        指标
      </span>
      {INDICATOR_REGISTRY.map((item) => {
        const active = toggles[item.id];
        return (
          <button
            key={item.id}
            type="button"
            onClick={() => onToggle(item.id)}
            className={cn(
              "rounded-full px-2 py-0.5 font-mono text-[10px] transition-colors",
              active
                ? "bg-primary/15 text-primary ring-1 ring-primary/40"
                : "bg-muted/40 text-muted-foreground hover:bg-muted/60"
            )}
            title={item.kind === "overlay" ? "主图叠加" : "独立副图"}
          >
            {item.label}
          </button>
        );
      })}
    </div>
  );
}
