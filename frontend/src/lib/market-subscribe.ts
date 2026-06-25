import { api } from "@/api/client";

/** 将品种加入 Rust 端轮询订阅列表。 */
export async function ensureMarketSubscription(symbol: string) {
  const sym = symbol.trim().toLowerCase();
  if (!sym) return;
  try {
    await api.marketSubscribe([sym]);
  } catch {
    /* 非 Tauri 或 poll 未启动时忽略 */
  }
}
