import { api } from "@/api/client";

/** 将品种加入 Rust 端轮询订阅列表。 */
export async function ensureMarketSubscription(symbols: string | string[]) {
  const list = (Array.isArray(symbols) ? symbols : [symbols])
    .map((s) => s.trim().toLowerCase())
    .filter(Boolean);
  if (list.length === 0) return;
  try {
    await api.marketSubscribe(list);
  } catch {
    /* 非 Tauri 或 poll 未启动时忽略 */
  }
}
