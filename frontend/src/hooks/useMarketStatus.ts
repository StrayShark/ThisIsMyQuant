import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { useAppStore, type MarketStatus } from "@/app/store";

function parseHealth(data: {
  feeds?: Record<string, boolean>;
  realtime?: { available?: boolean; source?: string | null };
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
    statusMessage = `后端轮询 ${pollInterval}s · AKShare + 金十`;
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

export function useMarketStatus() {
  const setMarketStatus = useAppStore((s) => s.setMarketStatus);

  const { data } = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const res = await api.health();
      return parseHealth(res.data);
    },
    refetchInterval: 30_000,
    retry: 2,
  });

  useEffect(() => {
    if (data) setMarketStatus(data);
  }, [data, setMarketStatus]);
}
