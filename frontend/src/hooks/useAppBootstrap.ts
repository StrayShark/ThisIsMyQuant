import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";

/** 启动时预拉财经日历。 */
export function useAppBootstrap() {
  const queryClient = useQueryClient();

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
