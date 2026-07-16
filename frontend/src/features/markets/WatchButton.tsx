import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Star } from "lucide-react";
import { Button } from "@/components/ui/button";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import type { MarketType } from "@/types";
import { cn } from "@/lib/utils";

interface WatchButtonProps {
  symbol: string;
  name: string;
  market: MarketType;
  className?: string;
}

export function WatchButton({ symbol, name, market, className }: WatchButtonProps) {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);

  const { data: groups } = useQuery({
    queryKey: ["watchlist-groups"],
    queryFn: () => api.listWatchlistGroups(),
    staleTime: 60_000,
  });

  const defaultGroupId = useMemo(() => groups?.[0]?.id ?? "default", [groups]);

  const { data: items } = useQuery({
    queryKey: ["watchlist-items", defaultGroupId],
    queryFn: () => api.listWatchlistItems(defaultGroupId),
    enabled: !!defaultGroupId,
    staleTime: 30_000,
  });

  const item = useMemo(
    () => items?.find((i) => i.symbol === symbol && i.asset_type === market),
    [items, symbol, market]
  );
  const isWatched = !!item;

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["watchlist-items", defaultGroupId] });
    queryClient.invalidateQueries({ queryKey: ["market-assets"] });
    queryClient.invalidateQueries({ queryKey: ["watchlist-summary"] });
  };

  const addMutation = useMutation({
    mutationFn: () =>
      api.addWatchlistItem({
        group_id: defaultGroupId,
        asset_type: market,
        symbol,
        name,
      }),
    onSuccess: () => {
      showToast(`已将 ${name} 加入自选`);
      invalidate();
    },
    onError: (err: Error) => showToast(err.message || "加入自选失败"),
  });

  const removeMutation = useMutation({
    mutationFn: () => api.removeWatchlistItem(item!.id),
    onSuccess: () => {
      showToast(`已将 ${name} 移出自选`);
      invalidate();
    },
    onError: (err: Error) => showToast(err.message || "移出自选失败"),
  });

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isWatched) {
      removeMutation.mutate();
    } else {
      addMutation.mutate();
    }
  };

  const busy = addMutation.isPending || removeMutation.isPending;

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      className={cn("h-8 w-8", className)}
      onClick={handleClick}
      disabled={busy}
      aria-label={isWatched ? "移出自选" : "加入自选"}
    >
      <Star
        className={cn(
          "h-4 w-4 transition-colors",
          isWatched ? "fill-amber-400 text-amber-400" : "text-muted-foreground"
        )}
      />
    </Button>
  );
}
