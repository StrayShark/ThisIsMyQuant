import { useEffect, useState, useMemo } from "react";
import type React from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Send, Calendar, TrendingUp, AlertCircle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { formatPrice, formatPercent, formatAmount } from "@/features/markets/market-utils";
import type { MarketAsset, MarketType } from "@/types";

interface AssetSidebarProps {
  symbol: string;
  market: MarketType;
  asset: MarketAsset;
  className?: string;
}

function KeyMetricsCard({ asset, market }: { asset: MarketAsset; market: MarketType }) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm">
          <TrendingUp className="h-4 w-4 text-primary" />
          关键指标
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <MetricRow label="最新价" value={formatPrice(asset.price)} />
        <MetricRow label="涨跌幅" value={formatPercent(asset.change_pct)} />
        <MetricRow label="涨跌额" value={formatPrice(asset.change_amount)} />
        <MetricRow label="成交额" value={formatAmount(asset.turnover)} />
        {market === "futures" && <MetricRow label="成交量" value={asset.volume?.toLocaleString("zh-CN") ?? "--"} />}
        <MetricRow label="市场" value={market === "futures" ? "期货" : "A股"} />
        <MetricRow label="板块" value={asset.sector ?? "--"} />
      </CardContent>
    </Card>
  );
}

function MetricRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between text-sm">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium tabular-nums text-foreground">{value}</span>
    </div>
  );
}

function WatchlistNotes({ symbol, market, name }: { symbol: string; market: MarketType; name: string }) {
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

  const [notes, setNotes] = useState(item?.notes ?? "");
  const [alertPrice, setAlertPrice] = useState("");
  const [alertPct, setAlertPct] = useState("");

  useEffect(() => {
    setNotes(item?.notes ?? "");
    setAlertPrice(item?.alert_price === null || item?.alert_price === undefined ? "" : String(item.alert_price));
    setAlertPct(item?.alert_pct === null || item?.alert_pct === undefined ? "" : String(item.alert_pct));
  }, [item?.id, item?.notes, item?.alert_price, item?.alert_pct]);

  const updateMutation = useMutation({
    mutationFn: () => {
      const parsedAlertPrice = alertPrice.trim() === "" ? undefined : Number(alertPrice);
      const parsedAlertPct = alertPct.trim() === "" ? undefined : Number(alertPct);
      if (!item) {
        return api.addWatchlistItem({
          group_id: defaultGroupId,
          asset_type: market,
          symbol,
          name,
          notes,
          alert_price: parsedAlertPrice,
          alert_pct: parsedAlertPct,
        });
      }
      return api.updateWatchlistItem({
        id: item.id,
        group_id: item.group_id,
        asset_type: item.asset_type,
        symbol: item.symbol,
        name: item.name,
        notes,
        alert_price: parsedAlertPrice,
        alert_pct: parsedAlertPct,
        sort_order: item.sort_order,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["watchlist-items", defaultGroupId] });
      showToast("自选备注已保存");
    },
    onError: (err: Error) => showToast(err.message || "保存失败"),
  });

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm">自选备注</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <Textarea
          placeholder="记录你的交易逻辑、风险提示或观察要点..."
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          className="min-h-[100px] resize-none"
        />
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="mb-1 block text-xs text-muted-foreground">触发价</label>
            <Input
              type="number"
              value={alertPrice}
              onChange={(e) => setAlertPrice(e.target.value)}
              placeholder="价格提醒"
              className="h-9"
            />
          </div>
          <div>
            <label className="mb-1 block text-xs text-muted-foreground">涨跌幅%</label>
            <Input
              type="number"
              value={alertPct}
              onChange={(e) => setAlertPct(e.target.value)}
              placeholder="幅度提醒"
              className="h-9"
            />
          </div>
        </div>
        <Button
          type="button"
          size="sm"
          className="w-full"
          onClick={() => updateMutation.mutate()}
          disabled={updateMutation.isPending}
        >
          {updateMutation.isPending ? "保存中..." : "保存备注"}
        </Button>
      </CardContent>
    </Card>
  );
}

function RelatedEvents({ symbol }: { symbol: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ["asset-sidebar-events", symbol],
    queryFn: () => api.listCalendarEvents({ keyword: symbol }),
    staleTime: 60_000,
  });

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm">
          <Calendar className="h-4 w-4 text-primary" />
          相关事件
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {isLoading ? (
          <Skeleton className="h-20 w-full" />
        ) : !data || data.length === 0 ? (
          <p className="text-sm text-muted-foreground">暂无相关事件</p>
        ) : (
          data.map((event) => (
            <div key={event.id} className="border-b border-border pb-2 last:border-0 last:pb-0">
              <p className="text-sm font-medium text-foreground">{event.name}</p>
              <p className="text-xs text-muted-foreground">
                {event.country} · {event.pub_time}
              </p>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  );
}

function AiQuickAsk() {
  const [question, setQuestion] = useState("");

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-sm">
          <AlertCircle className="h-4 w-4 text-primary" />
          AI 快问
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <Textarea
          placeholder="例如：最近有哪些影响该品种的宏观事件？"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          className="min-h-[80px] resize-none"
        />
        <Button type="button" size="sm" className="w-full gap-1.5" disabled={!question.trim()}>
          <Send className="h-3.5 w-3.5" />
          提问
        </Button>
      </CardContent>
    </Card>
  );
}

export function AssetSidebar({ symbol, market, asset, className }: AssetSidebarProps) {
  return (
    <div className={`flex flex-col gap-4 ${className ?? ""}`}>
      <KeyMetricsCard asset={asset} market={market} />
      <WatchlistNotes symbol={symbol} market={market} name={asset.name} />
      <RelatedEvents symbol={symbol} />
      <AiQuickAsk />
    </div>
  );
}
