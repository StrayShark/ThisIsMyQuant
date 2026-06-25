import { create } from "zustand";
import type { Interval } from "@/types";

export interface MarketStatus {
  akshareOnline: boolean;
  jinshiOnline: boolean;
  realtimeOnline: boolean;
  realtimeSource: string | null;
  statusMessage: string;
}

interface AppState extends MarketStatus {
  currentSymbol: string;
  setCurrentSymbol: (s: string) => void;
  currentInterval: Interval;
  setCurrentInterval: (i: Interval) => void;
  watchlist: string[];
  toggleWatch: (s: string) => void;
  setMarketStatus: (s: MarketStatus) => void;
}

export const useAppStore = create<AppState>((set) => ({
  currentSymbol: "RB0",
  setCurrentSymbol: (s) =>
    set((st) =>
      st.currentSymbol === s ? st : { currentSymbol: s, currentInterval: "1d" }
    ),
  currentInterval: "1d",
  setCurrentInterval: (i) => set({ currentInterval: i }),
  watchlist: ["rb2510", "au2512", "IF2512"],
  toggleWatch: (s) =>
    set((st) => ({
      watchlist: st.watchlist.includes(s)
        ? st.watchlist.filter((x) => x !== s)
        : [...st.watchlist, s],
    })),

  akshareOnline: false,
  jinshiOnline: false,
  realtimeOnline: false,
  realtimeSource: null,
  statusMessage: "",
  setMarketStatus: (status) => set(status),
}));
