import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/api/client";
import type { LeaderboardCategory, MarketType } from "@/types";
import { AssetIdentityCell } from "./AssetIdentityCell";
import { PriceChangeCell } from "./PriceChangeCell";
import { formatAmount } from "./market-utils";

interface MarketLeaderboardProps {
  market?: MarketType | "all";
  className?: string;
}

const CATEGORIES: { value: LeaderboardCategory; label: string }[] = [
  { value: "gainers", label: "涨跌榜" },
  { value: "turnover", label: "成交额榜" },
  { value: "volume_spike", label: "放量榜" },
  { value: "watchlist_moves", label: "自选异动" },
];

export function MarketLeaderboard({ market, className }: MarketLeaderboardProps) {
  const [category, setCategory] = useState<LeaderboardCategory>("gainers");

  const { data: leaderboard, isLoading } = useQuery({
    queryKey: ["market-leaderboard", category, market],
    queryFn: () =>
      api.getMarketLeaderboard({
        category,
        market: market === "all" ? undefined : market,
        limit: 10,
      }),
  });

  return (
    <Card className={cn("h-fit", className)}>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">发现</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <Tabs
          value={category}
          onValueChange={(v) => setCategory(v as LeaderboardCategory)}
          className="w-full"
        >
          <TabsList className="grid h-8 w-full grid-cols-4">
            {CATEGORIES.map((c) => (
              <TabsTrigger key={c.value} value={c.value} className="text-[11px]">
                {c.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        {isLoading && (
          <div className="space-y-2">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-8 animate-pulse rounded bg-muted" />
            ))}
          </div>
        )}

        {!isLoading && leaderboard && (
          <div className="space-y-1">
            {leaderboard.assets.length === 0 && (
              <div className="py-6 text-center text-xs text-muted-foreground">暂无数据</div>
            )}
            {leaderboard.assets.map((asset, index) => (
              <div
                key={`${asset.market}:${asset.symbol}`}
                className="grid grid-cols-[24px_1fr_auto] items-center gap-2 rounded-lg px-1 py-1.5 hover:bg-muted/40"
              >
                <span className="text-center text-xs text-muted-foreground">{index + 1}</span>
                <AssetIdentityCell asset={asset} />
                <div className="text-right">
                  <PriceChangeCell value={asset.change_pct} />
                  <div className="text-[10px] text-muted-foreground">{formatAmount(asset.turnover)}</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
