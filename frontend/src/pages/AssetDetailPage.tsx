import { useMemo } from "react";
import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { PageShell } from "@/components/layout/PageShell";
import { PageHeader } from "@/components/layout/PageHeader";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/api/client";
import { AssetHeader } from "@/features/detail/AssetHeader";
import { AssetDetailTabs } from "@/features/detail/AssetDetailTabs";
import { AssetSidebar } from "@/features/detail/AssetSidebar";
import type { MarketAsset, MarketType } from "@/types";

function detectMarket(symbol: string): MarketType {
  if (/^\d{6}(\.SH|\.SZ)?$/i.test(symbol)) return "stock";
  if (/^\d+$/.test(symbol)) return "stock";
  return "futures";
}

function useAsset(symbol: string, market: MarketType) {
  return useQuery({
    queryKey: ["asset-detail", symbol, market],
    queryFn: async () => {
      const search = await api.searchAssets(symbol, 10);
      let found = search.assets.find(
        (a) => a.symbol.toLowerCase() === symbol.toLowerCase() && a.market === market
      );
      if (!found) {
        found = search.assets.find((a) => a.symbol.toLowerCase() === symbol.toLowerCase());
      }
      if (found) return found;

      const list = await api.listMarketAssets({ market, query: symbol, limit: 20 });
      found = list.assets.find(
        (a) => a.symbol.toLowerCase() === symbol.toLowerCase() && a.market === market
      );
      if (!found) {
        found = list.assets.find((a) => a.symbol.toLowerCase() === symbol.toLowerCase());
      }
      return found ?? null;
    },
    staleTime: 30_000,
  });
}

function buildFallbackAsset(symbol: string, market: MarketType): MarketAsset {
  return {
    symbol,
    name: symbol,
    market,
    quality: "pending",
    source: "--",
    updated_at: new Date().toISOString(),
  };
}

export function AssetDetailPage() {
  const { symbol } = useParams<{ symbol: string }>();
  const rawSymbol = symbol ?? "";
  const market = useMemo(() => detectMarket(rawSymbol), [rawSymbol]);

  const { data: asset, isLoading, error } = useAsset(rawSymbol, market);
  const displayAsset: MarketAsset = asset ?? buildFallbackAsset(rawSymbol, market);

  return (
    <PageShell>
      <PageHeader
        title={displayAsset.name}
        description={[
          displayAsset.symbol,
          displayAsset.category,
          displayAsset.exchange,
        ]
          .filter(Boolean)
          .join(" · ")}
      />

      {isLoading ? (
        <div className="space-y-6">
          <Skeleton className="h-28 w-full rounded-2xl" />
          <Skeleton className="h-96 w-full rounded-2xl" />
        </div>
      ) : error ? (
        <div className="rounded-2xl border border-destructive/30 bg-destructive/10 p-8 text-center text-sm text-destructive">
          标的加载失败：{(error as Error).message}
        </div>
      ) : (
        <div className="flex flex-col gap-6 lg:flex-row">
          <div className="flex min-w-0 flex-1 flex-col gap-6 lg:w-2/3">
            <AssetHeader asset={displayAsset} market={market} />
            <AssetDetailTabs symbol={rawSymbol} market={market} asset={displayAsset} />
          </div>
          <div className="w-full shrink-0 lg:w-80 xl:w-96">
            <AssetSidebar symbol={rawSymbol} market={market} asset={displayAsset} />
          </div>
        </div>
      )}
    </PageShell>
  );
}
