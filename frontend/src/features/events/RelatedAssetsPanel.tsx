import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { TrendingUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import { AssetIdentityCell } from "@/features/markets/AssetIdentityCell";
import { STATIC_FUTURES_CATALOG } from "@/data/futures";
import type { MarketAsset, MarketEvent } from "@/types";

interface RelatedAssetsPanelProps {
  events: MarketEvent[];
  maxItems?: number;
  className?: string;
}

const FUTURES_SYMBOL_MAP = new Map(
  STATIC_FUTURES_CATALOG.flatMap((sector) =>
    sector.products.map((p) => [p.symbol, { name: p.name, exchange: p.exchange, category: p.code, sector: sector.name }] as const)
  )
);

function isStockSymbol(symbol: string): boolean {
  return /^\d/.test(symbol) || /\.(SH|SZ|BJ)$/.test(symbol);
}

function buildAsset(symbol: string): MarketAsset {
  const futuresMeta = FUTURES_SYMBOL_MAP.get(symbol);
  const market = isStockSymbol(symbol) ? "stock" : "futures";

  return {
    symbol,
    name: futuresMeta?.name ?? symbol,
    market,
    sector: futuresMeta?.sector ?? null,
    industry: market === "stock" ? null : null,
    category: futuresMeta?.category ?? null,
    exchange: futuresMeta?.exchange ?? null,
    quality: "reference",
    source: "event",
    updated_at: new Date().toISOString(),
  };
}

export function RelatedAssetsPanel({
  events,
  maxItems = 8,
  className,
}: RelatedAssetsPanelProps) {
  const navigate = useNavigate();

  const assets = useMemo(() => {
    const counts = new Map<string, number>();
    for (const event of events) {
      for (const symbol of event.affected_symbols) {
        counts.set(symbol, (counts.get(symbol) ?? 0) + 1);
      }
    }
    return Array.from(counts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, maxItems)
      .map(([symbol]) => buildAsset(symbol));
  }, [events, maxItems]);

  return (
    <Card className={cn("h-fit", className)}>
      <CardHeader className="p-4 pb-2">
        <CardTitle className="flex items-center gap-2 text-sm font-semibold">
          <TrendingUp className="h-4 w-4 text-primary" />
          相关标的
          <DataQualityBadge status="reference" label="事件关联" className="ml-auto" />
        </CardTitle>
      </CardHeader>
      <CardContent className="p-4 pt-0">
        {assets.length === 0 ? (
          <p className="py-4 text-center text-xs text-muted-foreground">
            暂无相关标的
          </p>
        ) : (
          <ul className="divide-y divide-border">
            {assets.map((asset) => (
              <li key={asset.symbol}>
                <button
                  type="button"
                  onClick={() =>
                    navigate(
                      asset.market === "stock"
                        ? `/markets/stocks/${encodeURIComponent(asset.symbol)}`
                        : `/markets/futures/${encodeURIComponent(asset.symbol)}`
                    )
                  }
                  className="w-full py-2.5 text-left transition-opacity hover:opacity-80"
                >
                  <AssetIdentityCell asset={asset} />
                </button>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
