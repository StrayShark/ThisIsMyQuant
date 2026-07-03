import { useCallback, useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import type { UserPreferences } from "@/types";

/** 修改用户偏好并自动持久化（可选防抖，用于数字/文本输入）。 */
export function useUserPreferences() {
  const queryClient = useQueryClient();
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  const query = useQuery({
    queryKey: ["user-preferences"],
    queryFn: () => api.getUserPreferences(),
  });

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const persist = useCallback(
    async (next: UserPreferences) => {
      const saved = await api.saveUserPreferences(next);
      queryClient.setQueryData(["user-preferences"], saved);
      queryClient.invalidateQueries({ queryKey: ["app-settings"] });
      queryClient.invalidateQueries({ queryKey: ["runtime-status"] });
      queryClient.invalidateQueries({ queryKey: ["schedule-status"] });
      return saved;
    },
    [queryClient]
  );

  const update = useCallback(
    (partial: Partial<UserPreferences>, opts?: { debounceMs?: number }) => {
      const current = queryClient.getQueryData<UserPreferences>(["user-preferences"]);
      if (!current) return;
      const next = { ...current, ...partial };
      queryClient.setQueryData(["user-preferences"], next);

      const ms = opts?.debounceMs ?? 0;
      if (timerRef.current) clearTimeout(timerRef.current);
      if (ms > 0) {
        timerRef.current = setTimeout(() => {
          void persist(next);
        }, ms);
      } else {
        void persist(next);
      }
    },
    [queryClient, persist]
  );

  return { ...query, prefs: query.data, update, persist };
}
