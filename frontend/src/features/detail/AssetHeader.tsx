import { useNavigate } from "react-router-dom";
import { ArrowRightLeft, Sparkles } from "lucide-react";
import { AssetIdentityCell } from "@/features/markets/AssetIdentityCell";
import { PriceChangeCell } from "@/features/markets/PriceChangeCell";
import { WatchButton } from "@/features/markets/WatchButton";
import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import { Button } from "@/components/ui/button";
import { AiSummaryModal } from "@/features/ai/AiSummaryModal";
import { useAiSummary } from "@/features/ai/useAiSummary";
import { formatPrice, formatTimeAgo } from "@/features/markets/market-utils";
import type { MarketAsset, MarketType } from "@/types";

interface AssetHeaderProps {
  asset: MarketAsset;
  market: MarketType;
  className?: string;
}

export function AssetHeader({ asset, market, className }: AssetHeaderProps) {
  const navigate = useNavigate();
  const aiSummary = useAiSummary();

  return (
    <div
      className={`flex flex-col gap-4 rounded-2xl border border-border bg-card p-5 sm:flex-row sm:items-center sm:justify-between ${className ?? ""}`}
    >
      <div className="flex min-w-0 items-center gap-4">
        <AssetIdentityCell asset={asset} className="min-w-0" />
        <div className="hidden h-8 w-px bg-border sm:block" />
        <div className="flex flex-col">
          <div className="flex items-baseline gap-3">
            <span className="text-2xl font-semibold tabular-nums text-foreground">
              {formatPrice(asset.price)}
            </span>
            <PriceChangeCell value={asset.change_pct} className="text-base" />
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span>更新 {formatTimeAgo(asset.updated_at)}</span>
            <DataQualityBadge status={asset.quality} />
            <span className="font-mono text-[11px] opacity-70">{asset.source}</span>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <WatchButton
          symbol={asset.symbol}
          name={asset.name}
          market={market}
          className="shrink-0"
        />
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="gap-1.5"
          onClick={() =>
            aiSummary.generate({
              task_type: "asset_brief",
              target_symbol: asset.symbol,
            })
          }
        >
          <Sparkles className="h-3.5 w-3.5" />
          生成 AI 速览
        </Button>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="gap-1.5"
          onClick={() => navigate(`/simulation?symbol=${encodeURIComponent(asset.symbol)}`)}
        >
          <ArrowRightLeft className="h-3.5 w-3.5" />
          模拟下单
        </Button>
      </div>

      <AiSummaryModal
        isOpen={aiSummary.isOpen}
        onClose={aiSummary.close}
        title="AI 标的速览"
        report={aiSummary.report}
        isLoading={aiSummary.isLoading}
        error={aiSummary.error}
      />
    </div>
  );
}
