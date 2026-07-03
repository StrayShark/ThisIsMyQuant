import { useMemo } from "react";
import { useQueries } from "@tanstack/react-query";
import { api } from "@/api/client";
import { useFuturesCatalog } from "@/hooks/useFuturesCatalog";
import { useRealtimeQuotes } from "@/hooks/useRealtimeQuotes";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { buildSectorHeat } from "./sector-heat";
import { TreemapHeatView } from "./TreemapHeatView";

export function SectorHeatmap() {
  const { data: sectors = [], isLoading: catalogLoading } = useFuturesCatalog("core");

  const symbols = useMemo(
    () => sectors.flatMap((s) => s.products.map((p) => p.symbol)),
    [sectors]
  );

  const realtimeQuotes = useRealtimeQuotes(symbols);

  const end = useMemo(() => new Date().toISOString(), []);
  const start = useMemo(() => {
    const d = new Date();
    d.setDate(d.getDate() - 14);
    return d.toISOString();
  }, []);

  const klineQueries = useQueries({
    queries: symbols.map((symbol) => ({
      queryKey: ["overview-klines", symbol],
      queryFn: () => api.getKlines({ symbol, interval: "1d", start, end, limit: 5 }),
      staleTime: 60_000,
      refetchInterval: 60_000,
      enabled: symbols.length > 0,
    })),
  });

  const klineBySymbol = useMemo(() => {
    const map = new Map<string, import("@/types").KLine[]>();
    symbols.forEach((sym, i) => {
      const data = klineQueries[i]?.data;
      if (data?.length) map.set(sym.toLowerCase(), data);
    });
    return map;
  }, [symbols, klineQueries]);

  const sectorHeat = useMemo(
    () => buildSectorHeat(sectors, klineBySymbol, realtimeQuotes),
    [sectors, klineBySymbol, realtimeQuotes]
  );

  const loading =
    catalogLoading || (symbols.length > 0 && klineQueries.some((q) => q.isLoading));

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">热力图</CardTitle>
      </CardHeader>
      <CardContent>
        {loading ? (
          <Skeleton className="h-[min(52vh,480px)] min-h-[400px] w-full rounded-md" />
        ) : (
          <TreemapHeatView sectorHeat={sectorHeat} klineBySymbol={klineBySymbol} />
        )}
      </CardContent>
    </Card>
  );
}
