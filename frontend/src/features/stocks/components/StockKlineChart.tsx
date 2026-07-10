import { useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  CandlestickSeries,
  HistogramSeries,
  createChart,
  type IChartApi,
  type ISeriesApi,
  type Time,
} from "lightweight-charts";
import { api } from "@/api/client";
import { getChartTheme } from "@/lib/chart-theme";
import { Skeleton } from "@/components/ui/skeleton";
import { DataQualityBadge } from "./DataQualityBadge";
import type { StockBar } from "@/types";

interface StockKlineChartProps {
  tsCode: string;
  adjustment?: string;
  height?: number;
}

function formatTradeDate(d: string): string {
  // 20260102 -> 2026-01-02
  if (d.length === 8) {
    return `${d.slice(0, 4)}-${d.slice(4, 6)}-${d.slice(6, 8)}`;
  }
  return d;
}

function toCandle(bar: StockBar) {
  return {
    time: formatTradeDate(bar.trade_date) as Time,
    open: bar.open ?? 0,
    high: bar.high ?? 0,
    low: bar.low ?? 0,
    close: bar.close ?? 0,
  };
}

function toVolume(bar: StockBar, upColor: string, downColor: string) {
  const close = bar.close ?? 0;
  const open = bar.open ?? 0;
  return {
    time: formatTradeDate(bar.trade_date) as Time,
    value: bar.volume ?? 0,
    color: close >= open ? upColor : downColor,
  };
}

export function StockKlineChart({ tsCode, adjustment = "qfq", height = 280 }: StockKlineChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const [quality, setQuality] = useState<{ status: string; message?: string | null; lastSuccessAt?: string | null }>({
    status: "pending",
  });

  const { data: bars, isLoading, error } = useQuery({
    queryKey: ["stock-klines", tsCode, adjustment],
    queryFn: () => api.getStockKlines({ ts_code: tsCode, adjustment, limit: 250 }),
    staleTime: 60_000,
  });

  useEffect(() => {
    if (isLoading) {
      setQuality({ status: "pending" });
    } else if (error) {
      setQuality({ status: "error", message: (error as Error).message });
    } else if (!bars || bars.length === 0) {
      setQuality({ status: "pending", message: "暂无 K 线数据，请在总览页触发同步" });
    } else {
      setQuality({
        status: "available",
        lastSuccessAt: bars[bars.length - 1]?.trade_date,
      });
    }
  }, [bars, isLoading, error]);

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
      rightPriceScale: {
        borderColor: theme.borderColor,
      },
      timeScale: {
        borderColor: theme.borderColor,
        timeVisible: false,
      },
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
    };
  }, [height]);

  useEffect(() => {
    if (!bars || !candleRef.current || !volumeRef.current) return;
    const theme = getChartTheme();
    candleRef.current.setData(bars.map(toCandle));
    volumeRef.current.setData(bars.map((b) => toVolume(b, theme.upColor, theme.downColor)));
    chartRef.current?.timeScale().fitContent();
  }, [bars]);

  if (isLoading) {
    return <Skeleton className="h-[280px] w-full rounded-md" />;
  }

  return (
    <div className="flex flex-col gap-2">
      <div ref={containerRef} className="w-full rounded-md" style={{ height }} />
      <DataQualityBadge
        status={quality.status}
        message={quality.message}
        lastSuccessAt={quality.lastSuccessAt}
      />
    </div>
  );
}
