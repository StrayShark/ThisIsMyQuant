import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import type { DataQualityStatus, MarketFilters as MarketFiltersType } from "@/types";
import { cn } from "@/lib/utils";

interface MarketFiltersProps {
  filters: MarketFiltersType;
  onChange: (filters: MarketFiltersType) => void;
  sectors?: string[];
  industries?: string[];
  className?: string;
}

const QUALITY_OPTIONS: { value: DataQualityStatus; label: string }[] = [
  { value: "live", label: "实时" },
  { value: "history", label: "历史" },
  { value: "stale", label: "陈旧" },
  { value: "error", label: "错误" },
  { value: "pending", label: "待更新" },
  { value: "estimated", label: "估算" },
  { value: "reference", label: "参考" },
  { value: "local", label: "本地" },
];

export function MarketFilters({
  filters,
  onChange,
  sectors = [],
  industries = [],
  className,
}: MarketFiltersProps) {
  const update = (patch: Partial<MarketFiltersType>) => onChange({ ...filters, ...patch });

  const hasSector = sectors.length > 0;
  const hasIndustry = industries.length > 0;

  return (
    <div className={cn("flex flex-wrap items-center gap-3", className)}>
      {hasSector && (
        <NativeSelect
          value={filters.sector ?? ""}
          onChange={(e) => update({ sector: e.target.value || null })}
          className="h-9 min-w-[140px] rounded-lg text-xs"
        >
          <option value="">全部板块</option>
          {sectors.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </NativeSelect>
      )}

      {hasIndustry && (
        <NativeSelect
          value={filters.industry ?? ""}
          onChange={(e) => update({ industry: e.target.value || null })}
          className="h-9 min-w-[140px] rounded-lg text-xs"
        >
          <option value="">全部行业</option>
          {industries.map((i) => (
            <option key={i} value={i}>
              {i}
            </option>
          ))}
        </NativeSelect>
      )}

      <NativeSelect
        value={filters.quality ?? ""}
        onChange={(e) => update({ quality: (e.target.value as DataQualityStatus) || null })}
        className="h-9 min-w-[120px] rounded-lg text-xs"
      >
        <option value="">全部状态</option>
        {QUALITY_OPTIONS.map((q) => (
          <option key={q.value} value={q.value}>
            {q.label}
          </option>
        ))}
      </NativeSelect>

      <Input
        type="number"
        min={0}
        step={1000000}
        placeholder="最小成交额"
        value={filters.min_turnover ?? ""}
        onChange={(e) => {
          const value = e.target.value === "" ? null : Number(e.target.value);
          update({ min_turnover: value });
        }}
        className="h-9 w-32 rounded-lg text-xs"
      />

      <label className="inline-flex cursor-pointer items-center gap-2 text-xs text-muted-foreground">
        <input
          type="checkbox"
          className="h-4 w-4 rounded border-border"
          checked={!!filters.watched}
          onChange={(e) => update({ watched: e.target.checked || null })}
        />
        仅自选
      </label>
    </div>
  );
}
