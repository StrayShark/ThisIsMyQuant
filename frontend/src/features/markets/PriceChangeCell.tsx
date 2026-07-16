import { cn } from "@/lib/utils";
import { formatPercent } from "./market-utils";

interface PriceChangeCellProps {
  value?: number | null;
  className?: string;
}

export function PriceChangeCell({ value, className }: PriceChangeCellProps) {
  const pct = value ?? 0;
  const positive = pct > 0;
  const negative = pct < 0;

  return (
    <span
      className={cn(
        "inline-flex items-center font-mono text-sm tabular-nums",
        positive && "text-[var(--color-up)]",
        negative && "text-[var(--color-down)]",
        !positive && !negative && "text-muted-foreground",
        className
      )}
    >
      {formatPercent(pct)}
    </span>
  );
}
