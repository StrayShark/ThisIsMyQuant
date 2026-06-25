import { useQuery } from "@tanstack/react-query";
import { getFuturesCatalog, loadFuturesCatalog } from "@/data/futures";
import type { LiquidityTier } from "@/types";

export function useFuturesCatalog(tier: LiquidityTier | "all" = "core") {
  return useQuery({
    queryKey: ["futures-catalog", tier],
    queryFn: async () => {
      await loadFuturesCatalog({ tier, includeWatch: tier === "core" });
      return getFuturesCatalog(tier === "all" ? "all" : "core");
    },
    staleTime: 5 * 60_000,
  });
}
