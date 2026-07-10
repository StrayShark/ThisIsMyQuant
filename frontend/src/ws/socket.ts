import type { WsMessage } from "@/types";

type Handler = (msg: WsMessage) => void;

class EventClient {
  private handlers = new Set<Handler>();
  private channels = new Set<string>();
  private started = false;

  resetUrl() {
    /* no-op */
  }

  async connect() {
    if (this.started) return;
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;
    this.started = true;
    const { listen } = await import("@tauri-apps/api/event");
    await listen<WsMessage>("kline-update", (e) => {
      const msg = e.payload;
      if (msg.type !== "kline") return;
      const channel = `kline:${msg.symbol}:${msg.interval}`;
      if (this.channels.size > 0 && !this.channels.has(channel)) return;
      this.handlers.forEach((h) => h(msg));
    });
    await listen<WsMessage>("quote-update", (e) => {
      const msg = e.payload;
      if (msg.type !== "quote") return;
      this.handlers.forEach((h) => h(msg));
    });
    await listen<WsMessage>("notification", (e) => {
      this.handlers.forEach((h) => h(e.payload));
    });
    await listen<WsMessage>("sim-order-update", (e) => {
      this.handlers.forEach((h) => h(e.payload));
    });
    await listen<WsMessage>("sim-account-update", (e) => {
      this.handlers.forEach((h) => h(e.payload));
    });
  }

  subscribe(channels: string[]) {
    channels.forEach((c) => this.channels.add(c));
    this.connect();
  }

  on(handler: Handler): () => void {
    this.handlers.add(handler);
    this.connect();
    return () => this.handlers.delete(handler);
  }
}

export const wsClient = new EventClient();
