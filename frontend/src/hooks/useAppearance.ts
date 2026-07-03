import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import {
  applyAppearance,
  bindSystemThemeListener,
  DEFAULT_APPEARANCE,
  normalizeAppearance,
} from "@/lib/appearance";

/** 启动时加载并应用外观偏好。 */
export function useAppearance() {
  const { data: prefs } = useQuery({
    queryKey: ["user-preferences"],
    queryFn: () => api.getUserPreferences(),
    staleTime: 60_000,
  });

  useEffect(() => {
    const appearance = prefs ? normalizeAppearance(prefs) : DEFAULT_APPEARANCE;
    applyAppearance(appearance);
    bindSystemThemeListener(appearance.theme, () => applyAppearance(appearance));
    return () => bindSystemThemeListener("dark", () => {});
  }, [prefs?.quote_color_scheme, prefs?.theme, prefs]);
}
