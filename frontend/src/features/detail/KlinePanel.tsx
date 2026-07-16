import { useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  CandlestickSeries,
  HistogramSeries,
  createSeriesMarkers,
  createChart,
  type IChartApi,
  type ISeriesApi,
  type ISeriesMarkersPluginApi,
  type SeriesMarker,
  type Time,
} from "lightweight-charts";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import { getChartTheme, volumeColor } from "@/lib/chart-theme";
import { api } from "@/api/client";
import type { Interval, KLine, MarketType, StockBar } from "@/types";

const INTERVALS: { value: Interval; label: string }[] = [
  { value: "1d", label: "1d" },
  { value: "1h", label: "1h" },
  { value: "30m", label: "30m" },
  { value: "15m", label: "15m" },
  { value: "5m", label: "5m" },
  { value: "1m", label: "1m" },
];

interface KlinePanelProps {
  symbol: string;
  market: MarketType;
  height?: number;
  className?: string;
}

function toFuturesCandle(k: KLine) {
  return {
    time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
    open: k.open,
    high: k.high,
    low: k.low,
    close: k.close,
  };
}

function toFuturesVolume(k: KLine, theme: ReturnType<typeof getChartTheme>) {
  return {
    time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
    value: k.volume,
    color: volumeColor(k.close, k.open, theme),
  };
}

function formatTradeDate(d: string): string {
  if (d.length === 8) {
    return `${d.slice(0, 4)}-${d.slice(4, 6)}-${d.slice(6, 8)}`;
  }
  return d;
}

function toStockCandle(bar: StockBar) {
  return {
    time: formatTradeDate(bar.trade_date) as Time,
    open: bar.open ?? 0,
    high: bar.high ?? 0,
    low: bar.low ?? 0,
    close: bar.close ?? 0,
  };
}

function toStockVolume(bar: StockBar, theme: ReturnType<typeof getChartTheme>) {
  return {
    time: formatTradeDate(bar.trade_date) as Time,
    value: bar.volume ?? 0,
    color: volumeColor(bar.close ?? 0, bar.open ?? 0, theme),
  };
}

export function KlinePanel({ symbol, market, height = 360, className }: KlinePanelProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const markersRef = useRef<ISeriesMarkersPluginApi<Time> | null>(null);
  const [interval, setInterval] = useState<Interval>("1d");

  const isStock = market === "stock";
  const activeInterval = isStock ? "1d" : interval;

  const futuresQuery = useQuery({
    queryKey: ["futures-klines", symbol, activeInterval],
    queryFn: () => api.getKlines({ symbol, interval: activeInterval, limit: 250 }),
    enabled: !isStock,
    staleTime: 60_000,
  });

  const stockQuery = useQuery({
    queryKey: ["stock-klines-detail", symbol],
    queryFn: () => api.getStockKlines({ ts_code: symbol, adjustment: "none", limit: 250 }),
    enabled: isStock,
    staleTime: 60_000,
  });

  const tradesQuery = useQuery({
    queryKey: ["sim-trade-markers", symbol],
    queryFn: () => api.listSimTrades({ symbol, limit: 50 }),
    enabled: !isStock,
    staleTime: 30_000,
  });

  const isLoading = isStock ? stockQuery.isLoading : futuresQuery.isLoading;
  const error = isStock ? stockQuery.error : futuresQuery.error;
  const klines: KLine[] | undefined = !isStock ? futuresQuery.data : undefined;
  const bars: StockBar[] | undefined = isStock ? stockQuery.data : undefined;
  const updatedAt = klines?.[klines.length - 1]?.start_time ?? bars?.[bars.length - 1]?.updated_at;

  useEffect(() => {
    if (!containerRef.current) return;
    const theme = getChartTheme();
    const chart = createChart(containerRef.current, {
      width: containerRef.current.clientWidth,
      height,
      layout: {
        background: { color: theme.background },
        textColor: theme.textColor,
        fontFamily: theme.fontFamily,
      },
      grid: {
        vertLines: { color: theme.gridColor },
        horzLines: { color: theme.gridColor },
      },
      crosshair: { mode: 1 },
      rightPriceScale: { borderColor: theme.borderColor },
      timeScale: { borderColor: theme.borderColor, timeVisible: activeInterval !== "1d" },
    });

    const candle = chart.addSeries(CandlestickSeries, {
      upColor: theme.upColor,
      downColor: theme.downColor,
      borderUpColor: theme.upColor,
      borderDownColor: theme.downColor,
      wickUpColor: theme.upColor,
      wickDownColor: theme.downColor,
    });

    const volume = chart.addSeries(HistogramSeries, {
      priceFormat: { type: "volume" },
      priceScaleId: "",
    });
    volume.priceScale().applyOptions({
      scaleMargins: { top: 0.8, bottom: 0 },
    });

    chartRef.current = chart;
    candleRef.current = candle;
    volumeRef.current = volume;
    markersRef.current = createSeriesMarkers(candle, []);

    const ro = new ResizeObserver(() => {
      if (containerRef.current) {
        chart.applyOptions({ width: containerRef.current.clientWidth });
      }
    });
    ro.observe(containerRef.current);

    return () => {
      ro.disconnect();
      chart.remove();
      chartRef.current = null;
      candleRef.current = null;
      volumeRef.current = null;
      markersRef.current = null;
    };
  }, [height, activeInterval]);

  useEffect(() => {
    if (!candleRef.current || !volumeRef.current) return;
    const theme = getChartTheme();

    if (!isStock && klines && klines.length > 0) {
      candleRef.current.setData(klines.map(toFuturesCandle));
      volumeRef.current.setData(klines.map((k) => toFuturesVolume(k, theme)));
      chartRef.current?.timeScale().fitContent();
    } else if (isStock && bars && bars.length > 0) {
      candleRef.current.setData(bars.map(toStockCandle));
      volumeRef.current.setData(bars.map((b) => toStockVolume(b, theme)));
      chartRef.current?.timeScale().fitContent();
    }
  }, [klines, bars, isStock]);

  useEffect(() => {
    if (!markersRef.current || isStock) return;
    const markers: SeriesMarker<Time>[] = (tradesQuery.data ?? [])
      .map((trade) => ({
        time: Math.floor(new Date(trade.traded_at).getTime() / 1000) as Time,
        position: trade.side === "buy" ? ("belowBar" as const) : ("aboveBar" as const),
        color: trade.side === "buy" ? "#16a34a" : "#dc2626",
        shape: trade.side === "buy" ? ("arrowUp" as const) : ("arrowDown" as const),
        text: `${trade.side === "buy" ? "买" : "卖"}${trade.offset === "open" ? "开" : "平"} ${trade.quantity}`,
      }))
      .sort((a, b) => Number(a.time) - Number(b.time));
    markersRef.current.setMarkers(markers);
  }, [tradesQuery.data, isStock]);

  const qualityStatus = error ? "error" : isLoading ? "pending" : klines || bars ? "live" : "pending";

  return (
    <div className={`flex flex-col gap-3 ${className ?? ""}`}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        {!isStock ? (
          <Tabs value={interval} onValueChange={(v) => setInterval(v as Interval)}>
            <TabsList className="h-8 bg-muted/40 p-0.5">
              {INTERVALS.map((it) => (
                <TabsTrigger key={it.value} value={it.value} className="h-7 px-2.5 font-mono text-xs">
                  {it.label}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        ) : (
          <span className="inline-flex h-8 items-center rounded-lg bg-muted/40 px-3 font-mono text-xs text-muted-foreground">
            日K
          </span>
        )}
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          {updatedAt && <span>更新: {new Date(updatedAt).toLocaleString("zh-CN")}</span>}
        </div>
      </div>

      <div className="relative rounded-xl border border-border bg-card">
        <div ref={containerRef} className="w-full" style={{ height }} />
        {isLoading && (
          <div className="absolute inset-0 flex items-center justify-center rounded-xl bg-background/60 backdrop-blur-[1px]">
            <Skeleton className="h-8 w-32" />
          </div>
        )}
      </div>

      <DataQualityBadge status={qualityStatus as import("@/types").DataQualityStatus} label={error ? "加载失败" : undefined} />
      {!isStock && tradesQuery.data && tradesQuery.data.length > 0 && (
        <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
          {tradesQuery.data.slice(0, 4).map((trade) => (
            <span key={trade.id} className="rounded-full bg-muted px-2 py-1">
              {new Date(trade.traded_at).toLocaleDateString("zh-CN")} · {trade.side === "buy" ? "买" : "卖"}
              {trade.offset === "open" ? "开" : "平"} {trade.quantity} @ {trade.price.toFixed(2)}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
