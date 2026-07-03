import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { useAppStore, type MarketStatus } from "@/app/store";
import { isTauriRuntime } from "@/lib/platform";

function parseHealth(data: {
  feeds?: Record<string, boolean>;
  realtime?: { available?: boolean; source?: string | null };
  realtime_enabled?: boolean;
  akshare?: { history?: boolean };
  jinshi?: {
    enabled?: boolean;
    connected?: boolean;
    calendar_ready?: boolean;
    calendar_fetched_at?: string | null;
    calendar_cached_events?: number;
  };
  poll?: { running?: boolean; interval?: number; symbol_count?: number };
}): MarketStatus {
  const akshareHistory = data.feeds?.akshare === true || data.akshare?.history === true;
  const jinshiOnline = data.jinshi?.connected === true;
  const jinshiCalendarReady = data.jinshi?.calendar_ready === true;
  const jinshiCalendarFetchedAt = data.jinshi?.calendar_fetched_at ?? null;
  const jinshiCalendarEventCount = data.jinshi?.calendar_cached_events ?? 0;
  const realtimeOnline = Boolean(data.realtime?.available || data.poll?.running);
  const pollInterval = data.poll?.interval;

  let statusMessage = "";
  if (!akshareHistory) statusMessage = "AKShare 历史 K 线不可用";
  else if (realtimeOnline && pollInterval)
    statusMessage = `后端轮询 ${pollInterval}s · ${data.poll?.symbol_count ?? 0} 品种 · AKShare`;
  else if (data.realtime_enabled && akshareHistory)
    statusMessage = "实时行情未启动，请检查设置或网络";
  else if (jinshiOnline && jinshiCalendarReady)
    statusMessage = "AKShare K 线 + 金十资讯/日历";
  else if (jinshiOnline) statusMessage = "AKShare K 线 + 金十资讯";
  else statusMessage = "AKShare K 线（金十资讯离线）";

  return {
    akshareOnline: akshareHistory,
    jinshiOnline,
    jinshiCalendarReady,
    jinshiCalendarFetchedAt,
    jinshiCalendarEventCount,
    realtimeOnline,
    realtimeSource: data.realtime?.source ?? (realtimeOnline ? "market_poll" : null),
    statusMessage,
  };
}

const REALTIME_TOAST_COOLDOWN_MS = 60_000;

export function useMarketStatus() {
  const setMarketStatus = useAppStore((s) => s.setMarketStatus);
  const showToast = useAppStore((s) => s.showToast);
  const lastToastAt = useRef(0);

  const { data } = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const res = await api.health();
      return { parsed: parseHealth(res.data), raw: res.data };
    },
    refetchInterval: 30_000,
    retry: 2,
  });

  useEffect(() => {
    if (data?.parsed) setMarketStatus(data.parsed);
  }, [data, setMarketStatus]);

  useEffect(() => {
    if (!isTauriRuntime() || !data?.raw) return;
    const raw = data.raw as {
      realtime_enabled?: boolean;
      realtime?: { available?: boolean };
      feeds?: Record<string, boolean>;
      akshare?: { history?: boolean };
    };
    const akshareOk = raw.feeds?.akshare === true || raw.akshare?.history === true;
    const realtimeExpected = raw.realtime_enabled === true && akshareOk;
    const realtimeUp = raw.realtime?.available === true;
    if (!realtimeExpected || realtimeUp) return;

    const now = Date.now();
    if (now - lastToastAt.current < REALTIME_TOAST_COOLDOWN_MS) return;
    lastToastAt.current = now;
    showToast("实时行情不可用 — 无法从数据源获取报价，请检查网络或 AKShare 设置");
  }, [data, showToast]);
}
