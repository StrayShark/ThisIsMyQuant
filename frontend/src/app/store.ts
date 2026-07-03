import { create } from "zustand";
import type { Interval } from "@/types";

export interface MarketStatus {
  akshareOnline: boolean;
  jinshiOnline: boolean;
  jinshiCalendarReady: boolean;
  jinshiCalendarFetchedAt: string | null;
  jinshiCalendarEventCount: number;
  realtimeOnline: boolean;
  realtimeSource: string | null;
  statusMessage: string;
}

export interface NewsFocus {
  dimension: string | null;
  keyword: string | null;
  eventId: string | null;
  eventName: string | null;
}

interface AppState extends MarketStatus {
  currentSymbol: string;
  setCurrentSymbol: (s: string) => void;
  currentInterval: Interval;
  setCurrentInterval: (i: Interval) => void;
  newsFocus: NewsFocus | null;
  setNewsFocus: (focus: NewsFocus | null) => void;
  toastMessage: string | null;
  showToast: (message: string) => void;
  clearToast: () => void;
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

  newsFocus: null,
  setNewsFocus: (focus) => set({ newsFocus: focus }),

  akshareOnline: false,
  jinshiOnline: false,
  jinshiCalendarReady: false,
  jinshiCalendarFetchedAt: null,
  jinshiCalendarEventCount: 0,
  realtimeOnline: false,
  realtimeSource: null,
  statusMessage: "",
  toastMessage: null,
  showToast: (message) => set({ toastMessage: message }),
  clearToast: () => set({ toastMessage: null }),
  setMarketStatus: (status) => set((st) => ({ ...st, ...status })),
}));
