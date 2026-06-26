import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";

/** 启动时同步后端 watchlist，并预拉财经日历。 */
export function useAppBootstrap() {
  const setWatchlist = useAppStore((s) => s.setWatchlist);
  const queryClient = useQueryClient();

  const { data: settings } = useQuery({
    queryKey: ["app-settings-bootstrap"],
    queryFn: () => api.getSettings(),
    staleTime: 120_000,
  });

  useEffect(() => {
    if (settings?.watchlist?.length) {
      setWatchlist(settings.watchlist);
    }
  }, [settings?.watchlist, setWatchlist]);

  useEffect(() => {
    void queryClient
      .prefetchQuery({
        queryKey: ["calendar", "macro", 3, null],
        queryFn: () => api.listCalendarEvents({ min_star: 3 }),
        staleTime: 300_000,
      })
      .then(() => queryClient.invalidateQueries({ queryKey: ["health"] }));
  }, [queryClient]);
}
