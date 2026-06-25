import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { dimensionLabel } from "@/data/dimensions";
import { Badge } from "@/components/ui/badge";

export type DimensionSummaryData = Record<string, string[]>;

interface DimensionSummaryProps {
  summary: DimensionSummaryData | null | undefined;
  compact?: boolean;
}

function normalizeSummary(raw: unknown): DimensionSummaryData | null {
  if (!raw || typeof raw !== "object") return null;
  const out: DimensionSummaryData = {};
  for (const [key, val] of Object.entries(raw as Record<string, unknown>)) {
    if (Array.isArray(val)) {
      const points = val.filter((v): v is string => typeof v === "string" && v.trim().length > 0);
      if (points.length > 0) out[key] = points;
    } else if (typeof val === "string" && val.trim()) {
      out[key] = [val];
    }
  }
  return Object.keys(out).length > 0 ? out : null;
}

export function DimensionSummary({ summary, compact = false }: DimensionSummaryProps) {
  const data = normalizeSummary(summary);
  const [open, setOpen] = useState<Record<string, boolean>>({});

  if (!data) return null;

  const entries = Object.entries(data);

  if (compact) {
    return (
      <div className="space-y-2">
        {entries.map(([code, points]) => (
          <div key={code} className="rounded-md border border-border bg-muted/20 px-3 py-2">
            <div className="mb-1 flex items-center gap-2">
              <Badge variant="secondary" className="text-[10px]">
                {dimensionLabel(code)}
              </Badge>
            </div>
            <ul className="list-inside list-disc space-y-0.5 text-xs text-muted-foreground">
              {points.slice(0, 2).map((p, i) => (
                <li key={i}>{p}</li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {entries.map(([code, points]) => {
        const isOpen = open[code] ?? true;
        const label = dimensionLabel(code);
        return (
          <div key={code} className="rounded-md border border-border">
            <button
              type="button"
              className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm font-medium hover:bg-muted/30"
              onClick={() => setOpen((s) => ({ ...s, [code]: !isOpen }))}
            >
              {isOpen ? (
                <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
              )}
              <span>{label}</span>
              <Badge variant="outline" className="ml-auto text-[10px] font-normal">
                {points.length} 条
              </Badge>
            </button>
            {isOpen && (
              <ul className="border-t border-border px-3 py-2 pl-9 text-sm leading-relaxed text-muted-foreground">
                {points.map((p, i) => (
                  <li key={i} className="list-disc">
                    {p}
                  </li>
                ))}
              </ul>
            )}
          </div>
        );
      })}
    </div>
  );
}
