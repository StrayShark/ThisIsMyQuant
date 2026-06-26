import { useEffect, useRef } from "react";
import { CandlestickSeries, createChart, type IChartApi, type Time } from "lightweight-charts";
import { api } from "@/api/client";
import { getChartTheme } from "@/lib/chart-theme";
import type { Interval, KLine } from "@/types";

interface MiniChartProps {
  symbol: string;
  interval: Interval;
  label: string;
}

export function MiniChart({ symbol, interval, label }: MiniChartProps) {
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);

  useEffect(() => {
    if (!ref.current) return;
    const theme = getChartTheme();
    const chart = createChart(ref.current, {
      width: ref.current.clientWidth,
      height: 100,
      layout: { background: { color: theme.background }, textColor: theme.textColor },
      grid: { vertLines: { visible: false }, horzLines: { visible: false } },
      rightPriceScale: { visible: false },
      timeScale: { visible: false },
      crosshair: { vertLine: { visible: false }, horzLine: { visible: false } },
    });
    const series = chart.addSeries(CandlestickSeries, {
      upColor: theme.upColor,
      downColor: theme.downColor,
      wickVisible: false,
      borderVisible: false,
    });
    chartRef.current = chart;

    void api
      .getKlines({ symbol, interval, limit: interval === "1d" ? 60 : 120 })
      .then((klines: KLine[]) => {
        series.setData(
          klines.map((k) => ({
            time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
          }))
        );
        chart.timeScale().fitContent();
      })
      .catch(() => {});

    const ro = new ResizeObserver(() => {
      if (ref.current) chart.applyOptions({ width: ref.current.clientWidth });
    });
    ro.observe(ref.current);

    return () => {
      ro.disconnect();
      chart.remove();
      chartRef.current = null;
    };
  }, [symbol, interval]);

  return (
    <div className="rounded-md border border-border bg-card p-2">
      <p className="mb-1 font-mono text-[10px] text-muted-foreground">{label}</p>
      <div ref={ref} className="h-[100px] w-full" />
    </div>
  );
}
