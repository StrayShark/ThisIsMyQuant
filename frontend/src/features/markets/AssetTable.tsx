import { useNavigate } from "react-router-dom";
import { ArrowUpDown, ArrowUp, ArrowDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent } from "@/components/ui/card";
import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import type { MarketAsset } from "@/types";
import { AssetIdentityCell } from "./AssetIdentityCell";
import { PriceChangeCell } from "./PriceChangeCell";
import { MiniSparkline } from "./MiniSparkline";
import { WatchButton } from "./WatchButton";
import { formatAmount, formatPrice } from "./market-utils";

interface AssetTableProps {
  assets: MarketAsset[];
  isLoading?: boolean;
  sortBy: string;
  sortDesc: boolean;
  onSort: (field: string) => void;
}

type SortField = "price" | "change_pct" | "turnover" | "volume" | "updated_at";

interface ColumnDef {
  key: SortField | "identity" | "sparkline" | "quality";
  label: string;
  align?: "left" | "right" | "center";
  sortable?: boolean;
  width?: string;
}

const COLUMNS: ColumnDef[] = [
  { key: "identity", label: "# / 标的", align: "left", width: "260px" },
  { key: "price", label: "最新价", align: "right", sortable: true },
  { key: "change_pct", label: "24h / 今日", align: "right", sortable: true },
  { key: "turnover", label: "成交额", align: "right", sortable: true },
  { key: "volume", label: "成交量", align: "right", sortable: true },
  { key: "identity", label: "板块 / 行业", align: "left" },
  { key: "sparkline", label: "走势", align: "center" },
  { key: "quality", label: "状态", align: "center" },
];

export function AssetTable({
  assets,
  isLoading,
  sortBy,
  sortDesc,
  onSort,
}: AssetTableProps) {
  const navigate = useNavigate();

  const handleRowClick = (asset: MarketAsset) => {
    const path = asset.market === "futures" ? `/markets/futures/${asset.symbol}` : `/markets/stocks/${asset.symbol}`;
    navigate(path);
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="p-0">
          <div className="flex h-64 items-center justify-center text-sm text-muted-foreground">
            加载市场数据中…
          </div>
        </CardContent>
      </Card>
    );
  }

  if (assets.length === 0) {
    return (
      <Card>
        <CardContent className="p-0">
          <div className="flex h-64 items-center justify-center text-sm text-muted-foreground">
            暂无标的
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="overflow-hidden">
      <CardContent className="p-0">
        <div className="overflow-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-border bg-muted/40 text-muted-foreground">
                {COLUMNS.map((col) => (
                  <th
                    key={col.label}
                    className={cn(
                      "py-3 px-4 font-medium",
                      col.align === "right" && "text-right",
                      col.align === "center" && "text-center",
                      col.sortable && "cursor-pointer select-none hover:text-foreground"
                    )}
                    style={{ width: col.width }}
                    onClick={() => col.sortable && onSort(col.key)}
                  >
                    <span className="inline-flex items-center gap-1">
                      {col.label}
                      {col.sortable && <SortIcon field={col.key} current={sortBy} desc={sortDesc} />}
                    </span>
                  </th>
                ))}
                <th className="w-12 px-2 py-3 text-center font-medium">自选</th>
              </tr>
            </thead>
            <tbody>
              {assets.map((asset, index) => (
                <tr
                  key={`${asset.market}:${asset.symbol}`}
                  className="border-b border-border/50 transition-colors hover:bg-muted/30 cursor-pointer"
                  onClick={() => handleRowClick(asset)}
                >
                  <td className="px-4 py-3">
                    <AssetIdentityCell asset={asset} rank={index + 1} />
                  </td>
                  <td className="px-4 py-3 text-right">
                    <span className="font-mono tabular-nums text-foreground">
                      {formatPrice(asset.price)}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <PriceChangeCell value={asset.change_pct} />
                  </td>
                  <td className="px-4 py-3 text-right">
                    <span className="font-mono tabular-nums text-muted-foreground">
                      {formatAmount(asset.turnover)}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <span className="font-mono tabular-nums text-muted-foreground">
                      {formatAmount(asset.volume)}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-muted-foreground">
                    {asset.sector ?? asset.industry ?? "--"}
                  </td>
                  <td className="px-4 py-3 text-center">
                    <MiniSparkline data={asset.sparkline} />
                  </td>
                  <td className="px-4 py-3 text-center">
                    <DataQualityBadge status={asset.quality} />
                  </td>
                  <td className="px-2 py-3 text-center" onClick={(e) => e.stopPropagation()}>
                    <WatchButton
                      symbol={asset.symbol}
                      name={asset.name}
                      market={asset.market}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}

function SortIcon({ field, current, desc }: { field: string; current: string; desc: boolean }) {
  if (field !== current) {
    return <ArrowUpDown className="h-3 w-3 opacity-40" />;
  }
  return desc ? <ArrowDown className="h-3 w-3" /> : <ArrowUp className="h-3 w-3" />;
}
