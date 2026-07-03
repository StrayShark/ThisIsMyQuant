import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { cn } from "@/lib/utils";
import { formatPct } from "./sector-heat";
import { layoutNestedTreemap, type TreemapRect } from "./treemap-layout";
import { treemapHeatStyle } from "./treemap-colors";
import type { SectorHeat } from "./sector-heat";

function tileTypography(w: number, h: number) {
  const m = Math.min(w, h);
  const area = w * h;
  // 极小格：仅 tooltip
  if (area < 96 || m < 12) {
    return { labelPx: 0, pctPx: 0, showLabel: false, showPct: false };
  }
  // 窄格：优先显示涨跌幅
  if (m < 20 || area < 240) {
    return { labelPx: 0, pctPx: 8, showLabel: false, showPct: w >= 22 && h >= 16 };
  }
  if (m < 28) {
    return { labelPx: 8, pctPx: 8, showLabel: w >= 32, showPct: h >= 18 };
  }
  if (m < 38) {
    return { labelPx: 9, pctPx: 9, showLabel: w >= 34, showPct: h >= 22 };
  }
  return { labelPx: 11, pctPx: 11, showLabel: true, showPct: true };
}

function TreemapTile({ rect }: { rect: TreemapRect }) {
  const w = rect.x1 - rect.x0;
  const h = rect.y1 - rect.y0;
  const style = treemapHeatStyle(rect.changePct);
  const { labelPx, pctPx, showLabel, showPct } = tileTypography(w, h);

  const inner = (
    <div
      className="flex h-full w-full flex-col items-center justify-center gap-0.5 overflow-hidden px-1 text-center leading-tight transition-opacity hover:brightness-110"
      style={style}
      title={`${rect.label} ${formatPct(rect.changePct)}`}
    >
      {showLabel && (
        <span
          className="max-w-full truncate font-medium"
          style={{ fontSize: labelPx }}
        >
          {rect.label}
        </span>
      )}
      {showPct && (
        <span
          className="font-mono font-semibold tabular-nums"
          style={{ fontSize: pctPx }}
        >
          {formatPct(rect.changePct)}
        </span>
      )}
    </div>
  );

  const tileClass =
    "absolute box-border overflow-hidden border border-[var(--heat-neutral,#0a0a0a)]";

  if (rect.symbol) {
    return (
      <Link
        to={`/workspace?symbol=${rect.symbol}`}
        className={tileClass}
        style={{ left: rect.x0, top: rect.y0, width: w, height: h }}
      >
        {inner}
      </Link>
    );
  }

  return (
    <div
      className={tileClass}
      style={{ left: rect.x0, top: rect.y0, width: w, height: h }}
    >
      {inner}
    </div>
  );
}

export function TreemapHeatView({
  sectorHeat,
  klineBySymbol,
}: {
  sectorHeat: SectorHeat[];
  klineBySymbol: Map<string, import("@/types").KLine[]>;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect;
      setSize({ width: Math.floor(width), height: Math.floor(height) });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const groups = useMemo(
    () => layoutNestedTreemap(sectorHeat, size.width, size.height, klineBySymbol),
    [sectorHeat, size.width, size.height, klineBySymbol]
  );

  return (
    <div>
      <div
        ref={containerRef}
        className="relative w-full overflow-hidden rounded-md bg-[var(--heat-neutral,#0a0a0a)]"
        style={{ height: "min(52vh, 480px)", minHeight: 400 }}
      >
        {size.width > 0 &&
          groups.map((group) => {
            const w = group.x1 - group.x0;
            const h = group.y1 - group.y0;
            return (
              <div
                key={group.id}
                className="absolute flex flex-col overflow-hidden bg-[var(--heat-neutral,#0a0a0a)] outline outline-1 outline-border/50"
                style={{ left: group.x0, top: group.y0, width: w, height: h }}
              >
                {group.labelHeight > 0 && (
                  <div
                    className={cn(
                      "flex shrink-0 items-center border-b border-border/40 px-2",
                      "bg-background/90 text-sm leading-none"
                    )}
                    style={{ height: group.labelHeight }}
                  >
                    <span className="truncate font-medium text-foreground">{group.label}</span>
                  </div>
                )}
                <div className="relative min-h-0 flex-1">
                  {group.products.map((rect) => (
                    <TreemapTile key={rect.id} rect={rect} />
                  ))}
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
}
