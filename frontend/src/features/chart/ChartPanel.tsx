import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  CandlestickSeries,
  HistogramSeries,
  createChart,
  type IChartApi,
  type ISeriesApi,
  type Time,
} from "lightweight-charts";
import { Settings2 } from "lucide-react";
import { api } from "@/api/client";
import { wsClient } from "@/ws/socket";
import { useAppStore } from "@/app/store";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { getFuturesProduct } from "@/data/futures";
import { ensureMarketSubscription } from "@/lib/market-subscribe";
import { getChartTheme } from "@/lib/chart-theme";
import type { Interval, KLine, WsMessage } from "@/types";
import {
  defaultChartConfigFromTheme,
  loadChartConfig,
  saveChartConfig,
  type ChartUserConfig,
} from "./chart-config";
import {
  buildCandleOptions,
  buildChartOptions,
  buildVolumeOptions,
  volumeBarColor,
} from "./chart-options";
import { ChartSettingsPanel } from "./ChartSettingsPanel";
import {
  applyIndicatorPaneLayout,
  createIndicatorSeries,
  syncIndicatorData,
  type IndicatorSeriesBundle,
} from "./indicators/apply-indicators";
import { IndicatorToolbar } from "./indicators/IndicatorToolbar";
import {
  loadIndicatorToggles,
  saveIndicatorToggles,
  type IndicatorToggles,
} from "./indicators/types";

const INTERVALS: { value: Interval; label: string }[] = [
  { value: "1d", label: "1d" },
  { value: "1h", label: "1h" },
  { value: "30m", label: "30m" },
  { value: "15m", label: "15m" },
  { value: "5m", label: "5m" },
  { value: "1m", label: "1m" },
];

interface OhlcLegend {
  time: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  change: number;
  changePct: number;
}

function toCandle(k: KLine) {
  return {
    time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
    open: k.open,
    high: k.high,
    low: k.low,
    close: k.close,
  };
}

function toVolume(k: KLine, config: ChartUserConfig) {
  return {
    time: Math.floor(new Date(k.start_time).getTime() / 1000) as Time,
    value: k.volume,
    color: volumeBarColor(k.close, k.open, config),
  };
}

function fmt(n: number, digits = 2) {
  return n.toLocaleString("zh-CN", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
}

export function ChartPanel() {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const klinesRef = useRef<KLine[]>([]);
  const configRef = useRef<ChartUserConfig | null>(null);
  const indicatorBundleRef = useRef<IndicatorSeriesBundle | null>(null);

  const { currentSymbol, currentInterval, setCurrentInterval } = useAppStore();
  const currentProduct = getFuturesProduct(currentSymbol);
  const [loading, setLoading] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [legend, setLegend] = useState<OhlcLegend | null>(null);
  const [indicators, setIndicators] = useState<IndicatorToggles>(() => loadIndicatorToggles());
  const indicatorsRef = useRef(indicators);

  const themeDefaults = useMemo(() => defaultChartConfigFromTheme(getChartTheme()), []);
  const [config, setConfig] = useState<ChartUserConfig>(() =>
    loadChartConfig(themeDefaults)
  );

  const patchConfig = useCallback(
    (patch: Partial<ChartUserConfig>) => {
      setConfig((prev) => {
        const next = { ...prev, ...patch };
        saveChartConfig(next);
        return next;
      });
    },
    []
  );

  const resetConfig = useCallback(() => {
    const next = defaultChartConfigFromTheme(getChartTheme());
    setConfig(next);
    saveChartConfig(next);
  }, []);

  const toggleIndicator = useCallback((id: keyof IndicatorToggles) => {
    setIndicators((prev) => {
      const next = { ...prev, [id]: !prev[id] };
      saveIndicatorToggles(next);
      indicatorsRef.current = next;
      return next;
    });
  }, []);

  const refreshIndicators = useCallback(() => {
    const chart = chartRef.current;
    const bundle = indicatorBundleRef.current;
    if (!chart || !bundle || klinesRef.current.length === 0) return;
    syncIndicatorData(
      chart,
      bundle,
      klinesRef.current,
      indicatorsRef.current,
      configRef.current ?? config
    );
  }, [config]);

  const applyPaneLayout = useCallback(
    (cfg: ChartUserConfig, ind: IndicatorToggles) => {
      const chart = chartRef.current;
      if (!chart) return;
      applyIndicatorPaneLayout(chart, cfg, ind);
    },
    []
  );

  const applyConfigToChart = useCallback(
    (cfg: ChartUserConfig) => {
      const chart = chartRef.current;
      if (!chart) return;
      chart.applyOptions(buildChartOptions(cfg));
      candleRef.current?.applyOptions(buildCandleOptions(cfg));
      volumeRef.current?.applyOptions(buildVolumeOptions(cfg));
      applyPaneLayout(cfg, indicatorsRef.current);
      configRef.current = cfg;
      refreshIndicators();
    },
    [applyPaneLayout, refreshIndicators]
  );

  useEffect(() => {
    if (!containerRef.current) return;

    const cfg = configRef.current ?? config;
    const chart = createChart(containerRef.current, buildChartOptions(cfg));
    chartRef.current = chart;

    const candle = chart.addSeries(CandlestickSeries, buildCandleOptions(cfg), 0);
    candleRef.current = candle;

    chart.addPane();
    const volume = chart.addSeries(HistogramSeries, buildVolumeOptions(cfg), 1);
    volumeRef.current = volume;
    chart.addPane(); // MACD
    chart.addPane(); // RSI
    indicatorBundleRef.current = createIndicatorSeries(chart);
    applyPaneLayout(cfg, indicatorsRef.current);
    configRef.current = cfg;

    chart.subscribeCrosshairMove((param) => {
      if (!param.time || !param.point) {
        setLegend(null);
        return;
      }
      const candleData = param.seriesData.get(candle) as
        | { open: number; high: number; low: number; close: number }
        | undefined;
      const volData = param.seriesData.get(volume) as { value: number } | undefined;
      if (!candleData) {
        setLegend(null);
        return;
      }
      const ts =
        typeof param.time === "number"
          ? new Date(param.time * 1000).toLocaleString("zh-CN")
          : String(param.time);
      const change = candleData.close - candleData.open;
      const changePct = candleData.open ? (change / candleData.open) * 100 : 0;
      setLegend({
        time: ts,
        open: candleData.open,
        high: candleData.high,
        low: candleData.low,
        close: candleData.close,
        volume: volData?.value ?? 0,
        change,
        changePct,
      });
    });

    return () => {
      chart.remove();
      chartRef.current = null;
      candleRef.current = null;
      volumeRef.current = null;
      indicatorBundleRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chart 实例仅创建一次
  }, []);

  useEffect(() => {
    configRef.current = config;
    applyConfigToChart(config);
  }, [config, applyConfigToChart]);

  useEffect(() => {
    if (!candleRef.current || !volumeRef.current) return;
    setLoading(true);
    api
      .getKlines({ symbol: currentSymbol, interval: currentInterval, limit: 500 })
      .then((klines) => {
        klinesRef.current = klines;
        const cfg = configRef.current ?? config;
        candleRef.current?.setData(klines.map(toCandle));
        if (cfg.showVolume) {
          volumeRef.current?.setData(klines.map((k) => toVolume(k, cfg)));
        } else {
          volumeRef.current?.setData([]);
        }
        chartRef.current?.timeScale().fitContent();
        refreshIndicators();
        if (klines.length > 0) {
          const last = klines[klines.length - 1];
          const change = last.close - last.open;
          setLegend({
            time: new Date(last.start_time).toLocaleString("zh-CN"),
            open: last.open,
            high: last.high,
            low: last.low,
            close: last.close,
            volume: last.volume,
            change,
            changePct: last.open ? (change / last.open) * 100 : 0,
          });
        }
      })
      .catch(() => {})
      .finally(() => setLoading(false));

    void ensureMarketSubscription(currentSymbol);
    const channel = `kline:${currentSymbol.toLowerCase()}:${currentInterval}`;
    wsClient.connect();
    wsClient.subscribe([channel]);
  }, [currentSymbol, currentInterval, refreshIndicators]);

  useEffect(() => {
    indicatorsRef.current = indicators;
    refreshIndicators();
  }, [indicators, refreshIndicators]);

  useEffect(() => {
    const off = wsClient.on((msg: WsMessage) => {
      if (msg.type !== "kline") return;
      if (
        msg.symbol.toLowerCase() !== currentSymbol.toLowerCase() ||
        msg.interval !== currentInterval
      )
        return;
      const cfg = configRef.current ?? config;
      const d = msg.data;
      candleRef.current?.update({
        time: Math.floor(d.t / 1000) as Time,
        open: d.o,
        high: d.h,
        low: d.l,
        close: d.c,
      });
      if (cfg.showVolume) {
        volumeRef.current?.update({
          time: Math.floor(d.t / 1000) as Time,
          value: d.v,
          color: volumeBarColor(d.c, d.o, cfg),
        });
      }
    });
    return () => off();
  }, [currentSymbol, currentInterval]);

  useEffect(() => {
    const cfg = configRef.current ?? config;
    const klines = klinesRef.current;
    if (!volumeRef.current || !candleRef.current || klines.length === 0) return;
    candleRef.current.applyOptions(buildCandleOptions(cfg));
    if (cfg.showVolume) {
      volumeRef.current.setData(klines.map((k) => toVolume(k, cfg)));
    } else {
      volumeRef.current.setData([]);
    }
    applyPaneLayout(cfg, indicatorsRef.current);
    refreshIndicators();
  }, [config, applyPaneLayout, refreshIndicators]);

  const changeClass =
    legend && legend.change >= 0 ? "text-[var(--color-up)]" : "text-[var(--color-down)]";

  return (
    <div className="panel flex h-full flex-col">
      <div className="panel-header">
        <div className="flex min-w-0 items-center gap-2">
          <span className="truncate text-sm font-semibold">
            {currentProduct?.name || currentSymbol}
          </span>
          <span className="shrink-0 font-mono text-xs text-muted-foreground">
            {currentSymbol} · {currentInterval}
          </span>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="ml-1 h-7 w-7 shrink-0 p-0"
            aria-label="图表配置"
            aria-pressed={settingsOpen}
            onClick={() => setSettingsOpen((o) => !o)}
          >
            <Settings2 className="h-3.5 w-3.5" />
          </Button>
        </div>
        <Tabs value={currentInterval} onValueChange={(v) => setCurrentInterval(v as Interval)}>
          <TabsList className="h-8 bg-muted/40 p-0.5">
            {INTERVALS.map((it) => (
              <TabsTrigger key={it.value} value={it.value} className="h-7 px-2.5 font-mono text-xs">
                {it.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </div>

      {settingsOpen && (
        <ChartSettingsPanel
          config={config}
          onChange={patchConfig}
          onReset={resetConfig}
          onClose={() => setSettingsOpen(false)}
        />
      )}

      <IndicatorToolbar toggles={indicators} onToggle={toggleIndicator} />

      <div className="relative min-h-0 flex-1 rounded-b-lg">
        {legend && (
          <div className="pointer-events-none absolute left-2 top-2 z-10 rounded-md border border-border/80 bg-background/90 px-2.5 py-1.5 font-mono text-[11px] leading-relaxed backdrop-blur-sm">
            <div className="mb-0.5 text-muted-foreground">{legend.time}</div>
            <div className="flex flex-wrap gap-x-3 gap-y-0.5">
              <span>O {fmt(legend.open)}</span>
              <span>H {fmt(legend.high)}</span>
              <span>L {fmt(legend.low)}</span>
              <span className={changeClass}>C {fmt(legend.close)}</span>
              {config.showVolume && <span>V {legend.volume.toLocaleString("zh-CN")}</span>}
              <span className={changeClass}>
                {legend.change >= 0 ? "+" : ""}
                {fmt(legend.change)} ({legend.changePct >= 0 ? "+" : ""}
                {fmt(legend.changePct)}%)
              </span>
            </div>
          </div>
        )}
        <div ref={containerRef} className="absolute inset-0" />
        {loading && (
          <div className="absolute inset-0 flex items-center justify-center bg-background/60 backdrop-blur-[1px]">
            <Skeleton className="h-8 w-32" />
          </div>
        )}
      </div>
    </div>
  );
}
