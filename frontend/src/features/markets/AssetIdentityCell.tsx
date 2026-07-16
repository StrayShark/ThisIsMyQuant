import { cn } from "@/lib/utils";
import type { MarketAsset } from "@/types";

interface AssetIdentityCellProps {
  asset: MarketAsset;
  rank?: number;
  className?: string;
}

export function AssetIdentityCell({ asset, rank, className }: AssetIdentityCellProps) {
  const label = asset.symbol.slice(0, 2).toUpperCase();
  const subtitle = [asset.symbol, asset.exchange, asset.category].filter(Boolean).join(" · ");

  return (
    <div className={cn("flex items-center gap-3", className)}>
      {rank !== undefined && (
        <span className="w-5 text-center text-xs text-muted-foreground">{rank}</span>
      )}
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-bold text-primary">
        {label}
      </div>
      <div className="min-w-0">
        <div className="truncate text-sm font-medium text-foreground">{asset.name}</div>
        <div className="truncate text-xs text-muted-foreground">{subtitle}</div>
      </div>
    </div>
  );
}
