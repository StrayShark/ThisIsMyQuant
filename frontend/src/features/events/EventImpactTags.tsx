import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import type { MarketEvent } from "@/types";

interface EventImpactTagsProps {
  event: MarketEvent;
  onSymbolClick?: (symbol: string) => void;
  onSectorClick?: (sector: string) => void;
  className?: string;
}

export function EventImpactTags({
  event,
  onSymbolClick,
  onSectorClick,
  className,
}: EventImpactTagsProps) {
  return (
    <div className={cn("flex flex-wrap items-center gap-1.5", className)}>
      {event.affected_symbols.map((symbol) => (
        <Badge
          key={`sym-${symbol}`}
          variant="outline"
          className="cursor-pointer text-[11px] hover:bg-muted"
          onClick={(e) => {
            e.stopPropagation();
            onSymbolClick?.(symbol);
          }}
        >
          {symbol}
        </Badge>
      ))}
      {event.affected_sectors.map((sector) => (
        <Badge
          key={`sec-${sector}`}
          variant="secondary"
          className="cursor-pointer text-[11px] hover:bg-muted"
          onClick={(e) => {
            e.stopPropagation();
            onSectorClick?.(sector);
          }}
        >
          {sector}
        </Badge>
      ))}
    </div>
  );
}
