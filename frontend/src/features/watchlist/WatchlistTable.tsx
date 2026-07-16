import React, { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation, useQuery, useQueryClient, useQueries } from "@tanstack/react-query";
import { Trash2, Edit3, TrendingUp, Bell } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import type { WatchlistItem, MarketAsset, DataQualityStatus } from "@/types";
import { AssetIdentityCell } from "@/features/markets/AssetIdentityCell";
import { PriceChangeCell } from "@/features/markets/PriceChangeCell";
import { MiniSparkline } from "@/features/markets/MiniSparkline";
import { formatPrice } from "@/features/markets/market-utils";
import { WatchlistNoteEditor } from "./WatchlistNoteEditor";

interface WatchlistTableProps {
  groupId: string;
}

interface EnrichedItem {
  item: WatchlistItem;
  asset: MarketAsset;
  price?: number | null;
  changePct?: number | null;
  quality: DataQualityStatus;
  sparkline?: number[] | null;
}

const COLUMNS = [
  { key: "identity", label: "标的", align: "left" as const, width: "220px" },
  { key: "price", label: "价格", align: "right" as const },
  { key: "change", label: "涨跌", align: "right" as const },
  { key: "sparkline", label: "走势", align: "center" as const, width: "110px" },
  { key: "notes", label: "备注", align: "left" as const },
  { key: "alert", label: "提醒", align: "left" as const, width: "140px" },
  { key: "quality", label: "状态", align: "center" as const, width: "80px" },
  { key: "action", label: "操作", align: "center" as const, width: "64px" },
];

export function WatchlistTable({ groupId }: WatchlistTableProps) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const [editingItemId, setEditingItemId] = useState<string | null>(null);

  const { data: items = [], isLoading: itemsLoading } = useQuery({
    queryKey: ["watchlist-items", groupId],
    queryFn: () => api.listWatchlistItems(groupId === "all" ? undefined : groupId),
  });

  const symbols = useMemo(() => items.map((i) => i.symbol), [items]);

  const { data: quotes = [] } = useQuery({
    queryKey: ["realtime-quotes", symbols],
    queryFn: () => api.getRealtimeQuotes(symbols),
    enabled: symbols.length > 0,
    refetchInterval: 10_000,
  });

  const quoteMap = useMemo(() => {
    const map = new Map<string, { price: number; changePct: number; quality: DataQualityStatus }>();
    for (const q of quotes) {
      const ts = new Date(q.timestamp).getTime();
      const ageMs = Date.now() - ts;
      const quality: DataQualityStatus = ageMs > 120_000 ? "stale" : "live";
      map.set(q.symbol, { price: q.last_price, changePct: q.change_pct, quality });
    }
    return map;
  }, [quotes]);

  const sparklineQueries = useQueries({
    queries: items.map((item) => ({
      queryKey: ["asset-sparkline", item.symbol, item.asset_type],
      queryFn: () => api.getAssetSparkline({ symbol: item.symbol, market: item.asset_type, points: 24 }),
      enabled: items.length > 0,
      staleTime: 60_000,
    })),
  });

  const enriched: EnrichedItem[] = useMemo(() => {
    return items.map((item, index) => {
      const quote = quoteMap.get(item.symbol);
      const sparkline = sparklineQueries[index]?.data ?? null;
      const asset: MarketAsset = {
        symbol: item.symbol,
        name: item.name,
        market: item.asset_type,
        quality: quote?.quality ?? "pending",
        source: "watchlist",
        updated_at: new Date().toISOString(),
      };
      return {
        item,
        asset,
        price: quote?.price ?? null,
        changePct: quote?.changePct ?? null,
        quality: quote?.quality ?? "pending",
        sparkline,
      };
    });
  }, [items, quoteMap, sparklineQueries]);

  const updateMutation = useMutation({
    mutationFn: (payload: {
      id: string;
      group_id: string;
      asset_type: "futures" | "stock";
      symbol: string;
      name: string;
      notes?: string;
      alert_price?: number;
      alert_pct?: number;
    }) => api.updateWatchlistItem(payload),
    onSuccess: () => {
      showToast("备注已保存");
      queryClient.invalidateQueries({ queryKey: ["watchlist-items"] });
      setEditingItemId(null);
    },
    onError: (err: Error) => showToast(err.message || "保存失败"),
  });

  const removeMutation = useMutation({
    mutationFn: (id: string) => api.removeWatchlistItem(id),
    onSuccess: () => {
      showToast("已移出自选");
      queryClient.invalidateQueries({ queryKey: ["watchlist-items"] });
      queryClient.invalidateQueries({ queryKey: ["watchlist-summary"] });
    },
    onError: (err: Error) => showToast(err.message || "删除失败"),
  });

  const handleRowClick = (item: WatchlistItem) => {
    if (editingItemId === item.id) return;
    const path = item.asset_type === "futures" ? `/markets/futures/${item.symbol}` : `/markets/stocks/${item.symbol}`;
    navigate(path);
  };

  const handleSave = (
    item: WatchlistItem,
    payload: { notes: string; alert_price: number | null; alert_pct: number | null }
  ) => {
    updateMutation.mutate({
      id: item.id,
      group_id: item.group_id,
      asset_type: item.asset_type,
      symbol: item.symbol,
      name: item.name,
      notes: payload.notes,
      alert_price: payload.alert_price ?? undefined,
      alert_pct: payload.alert_pct ?? undefined,
    });
  };

  if (itemsLoading) {
    return (
      <Card>
        <CardContent className="p-0">
          <div className="flex h-64 items-center justify-center text-sm text-muted-foreground">
            加载自选数据中…
          </div>
        </CardContent>
      </Card>
    );
  }

  if (items.length === 0) {
    return (
      <Card>
        <CardContent className="p-0">
          <div className="flex h-64 flex-col items-center justify-center gap-3 text-sm text-muted-foreground">
            <span>还没有自选标的，去市场添加</span>
            <Button variant="outline" size="sm" onClick={() => navigate("/markets")}>
              去市场
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="overflow-hidden">
      <CardContent className="p-0">
        <div className="overflow-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-border bg-muted/40 text-muted-foreground">
                {COLUMNS.map((col) => (
                  <th
                    key={col.key}
                    className={cn(
                      "py-3 px-4 font-medium",
                      col.align === "right" && "text-right",
                      col.align === "center" && "text-center"
                    )}
                    style={{ width: col.width }}
                  >
                    {col.label}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {enriched.map(({ item, asset, price, changePct, quality, sparkline }) => {
                const isEditing = editingItemId === item.id;
                return (
                  <React.Fragment key={item.id}>
                    <tr
                      className="group border-b border-border/50 transition-colors hover:bg-muted/30 cursor-pointer"
                      onClick={() => handleRowClick(item)}
                    >
                      <td className="px-4 py-3">
                        <AssetIdentityCell asset={asset} />
                      </td>
                      <td className="px-4 py-3 text-right">
                        <span className="font-mono tabular-nums text-foreground">
                          {formatPrice(price)}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-right">
                        <PriceChangeCell value={changePct} />
                      </td>
                      <td className="px-4 py-3 text-center">
                        <MiniSparkline data={sparkline} width={90} height={28} />
                      </td>
                      <td
                        className="px-4 py-3 text-muted-foreground"
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditingItemId(item.id);
                        }}
                      >
                        <div className="flex items-center gap-1">
                          <span className={cn("truncate max-w-[160px]", !item.notes && "italic opacity-60")}>
                            {item.notes || "点击添加备注"}
                          </span>
                          <Edit3 className="h-3 w-3 opacity-0 transition-opacity group-hover:opacity-100" />
                        </div>
                      </td>
                      <td
                        className="px-4 py-3"
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditingItemId(item.id);
                        }}
                      >
                        <div className="flex flex-col gap-0.5 text-muted-foreground">
                          {item.alert_price !== null && item.alert_price !== undefined && (
                            <span className="inline-flex items-center gap-1 text-[11px]">
                              <Bell className="h-3 w-3" />
                              价 {formatPrice(item.alert_price)}
                            </span>
                          )}
                          {item.alert_pct !== null && item.alert_pct !== undefined && (
                            <span className="inline-flex items-center gap-1 text-[11px]">
                              <TrendingUp className="h-3 w-3" />
                              幅 {item.alert_pct}%
                            </span>
                          )}
                          {item.alert_price === null && item.alert_pct === null && (
                            <span className="italic opacity-60">点击设置提醒</span>
                          )}
                        </div>
                      </td>
                      <td className="px-4 py-3 text-center">
                        <DataQualityBadge status={quality} />
                      </td>
                      <td className="px-4 py-3 text-center" onClick={(e) => e.stopPropagation()}>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-muted-foreground hover:text-destructive"
                          onClick={() => removeMutation.mutate(item.id)}
                          disabled={removeMutation.isPending}
                          aria-label="删除自选"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </td>
                    </tr>
                    {isEditing && (
                      <tr className="border-b border-border/50 bg-muted/20" key={`${item.id}-edit`}>
                        <td colSpan={COLUMNS.length} className="px-4 py-3">
                          <WatchlistNoteEditor
                            item={item}
                            onSave={(payload) => handleSave(item, payload)}
                            onCancel={() => setEditingItemId(null)}
                            className="max-w-xl"
                          />
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                );
              })}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}
