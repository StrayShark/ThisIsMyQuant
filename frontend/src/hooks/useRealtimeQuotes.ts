import { useEffect, useState } from "react";
import { api } from "@/api/client";
import { ensureMarketSubscription } from "@/lib/market-subscribe";
import { wsClient } from "@/ws/socket";
import type { RealtimeQuote } from "@/types";

/** 订阅品种实时行情（轮询 + quote-update 推送）。 */
export function useRealtimeQuotes(symbols: string[]) {
  const [quotes, setQuotes] = useState<Map<string, RealtimeQuote>>(new Map());
  const key = symbols.map((s) => s.toLowerCase()).sort().join(",");

  useEffect(() => {
    if (!key) return;
    void ensureMarketSubscription(symbols);
    void api
      .getRealtimeQuotes(symbols)
      .then((list) => {
        setQuotes(new Map(list.map((q) => [q.symbol.toLowerCase(), q])));
      })
      .catch(() => {});
  }, [key, symbols]);

  useEffect(() => {
    wsClient.connect();
    const off = wsClient.on((msg) => {
      if (msg.type !== "quote") return;
      const sym = msg.symbol.toLowerCase();
      setQuotes((prev) => {
        const next = new Map(prev);
        next.set(sym, {
          symbol: sym,
          last_price: msg.last_price,
          prev_close: msg.prev_close,
          change_pct: msg.change_pct,
          timestamp: msg.timestamp,
        });
        return next;
      });
    });
    return () => off();
  }, []);

  return quotes;
}
