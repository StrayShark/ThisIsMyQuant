import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { PageShell } from "@/components/layout/PageShell";
import { PageHeader } from "@/components/layout/PageHeader";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/api/client";
import type { MarketFilters as MarketFiltersType, MarketType } from "@/types";
import { AssetTable } from "@/features/markets/AssetTable";
import { AssetSearch } from "@/features/markets/AssetSearch";
import { MarketFilters } from "@/features/markets/MarketFilters";
import { MarketLeaderboard } from "@/features/markets/MarketLeaderboard";
import { AiSummaryModal } from "@/features/ai/AiSummaryModal";
import { useAiSummary } from "@/features/ai/useAiSummary";
import { cn } from "@/lib/utils";
import { Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";

type MarketTab = MarketType | "all" | "watched";

const TAB_ITEMS: { value: MarketTab; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "futures", label: "期货" },
  { value: "stock", label: "A股" },
  { value: "watched", label: "自选" },
];

export function MarketsPage() {
  const [tab, setTab] = useState<MarketTab>("all");
  const [filters, setFilters] = useState<MarketFiltersType>({ market: "all" });
  const aiSummary = useAiSummary();
  const [sortBy, setSortBy] = useState<string>("turnover");
  const [sortDesc, setSortDesc] = useState<boolean>(true);

  const handleTabChange = (value: MarketTab) => {
    setTab(value);
    if (value === "watched") {
      setFilters((prev) => ({ ...prev, market: "all", watched: true }));
    } else {
      setFilters((prev) => ({ ...prev, market: value, watched: null }));
    }
  };

  const handleSort = (field: string) => {
    if (sortBy === field) {
      setSortDesc((prev) => !prev);
    } else {
      setSortBy(field);
      setSortDesc(true);
    }
  };

  const { data: result, isLoading } = useQuery({
    queryKey: ["market-assets", filters, sortBy, sortDesc],
    queryFn: () =>
      api.listMarketAssets({
        ...filters,
        sort_by: sortBy,
        sort_desc: sortDesc,
        limit: 100,
      }),
  });

  const assets = useMemo(() => result?.assets ?? [], [result]);

  const sectorOptions = useMemo(
    () => Array.from(new Set(assets.map((a) => a.sector).filter(Boolean) as string[])),
    [assets]
  );
  const industryOptions = useMemo(
    () => Array.from(new Set(assets.map((a) => a.industry).filter(Boolean) as string[])),
    [assets]
  );

  return (
    <PageShell>
      <PageHeader
        title="市场"
        description="统一期货与 A 股的发现入口"
      >
        <AssetSearch />
        <Button
          size="sm"
          variant="outline"
          className="h-8 gap-1.5 rounded-full text-xs"
          onClick={() =>
            aiSummary.generate({
              task_type: "market_summary",
            })
          }
        >
          <Sparkles className="h-3.5 w-3.5" />
          AI 市场摘要
        </Button>
      </PageHeader>

      <Tabs value={tab} onValueChange={(v) => handleTabChange(v as MarketTab)} className="mb-5">
        <TabsList className="h-9">
          {TAB_ITEMS.map((item) => (
            <TabsTrigger key={item.value} value={item.value} className="text-xs">
              {item.label}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>

      <MarketFilters
        filters={filters}
        onChange={setFilters}
        sectors={sectorOptions}
        industries={industryOptions}
        className="mb-4"
      />

      <div className={cn("grid gap-5", "grid-cols-1 lg:grid-cols-[1fr_360px]")}>
        <AssetTable
          assets={assets}
          isLoading={isLoading}
          sortBy={sortBy}
          sortDesc={sortDesc}
          onSort={handleSort}
        />
        <MarketLeaderboard market={tab === "watched" ? "all" : (filters.market ?? "all")} />
      </div>

      <AiSummaryModal
        isOpen={aiSummary.isOpen}
        onClose={aiSummary.close}
        title="AI 市场摘要"
        report={aiSummary.report}
        isLoading={aiSummary.isLoading}
        error={aiSummary.error}
      />
    </PageShell>
  );
}
