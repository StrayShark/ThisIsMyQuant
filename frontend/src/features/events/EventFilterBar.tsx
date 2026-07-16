import { Search } from "lucide-react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { FilterPill } from "@/components/ui/filter-pill";
import type { EventImportance, EventSource } from "@/types";

export type EventSourceFilter = EventSource | "all";
export type EventImportanceFilter = EventImportance | "all";
export type EventTimeRange = "today" | "7d" | "30d";

export interface EventFilters {
  source: EventSourceFilter;
  importance: EventImportanceFilter;
  range: EventTimeRange;
  query: string;
}

const SOURCE_ITEMS: { value: EventSourceFilter; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "jin10", label: "金十" },
  { value: "calendar", label: "日历" },
  { value: "announcement", label: "公告" },
  { value: "earnings", label: "财报" },
  { value: "industry", label: "产业" },
];

const IMPORTANCE_ITEMS: { value: EventImportance; label: string }[] = [
  { value: "high", label: "高" },
  { value: "medium", label: "中" },
  { value: "low", label: "低" },
];

interface EventFilterBarProps {
  filters: EventFilters;
  onChange: (filters: EventFilters) => void;
  className?: string;
}

export function EventFilterBar({ filters, onChange, className }: EventFilterBarProps) {
  const setSource = (source: EventSourceFilter) => onChange({ ...filters, source });
  const setImportance = (importance: EventImportanceFilter) =>
    onChange({ ...filters, importance });
  const setRange = (range: EventTimeRange) => onChange({ ...filters, range });
  const setQuery = (query: string) => onChange({ ...filters, query });

  return (
    <div className={className}>
      <div className="mb-3 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <Tabs
          value={filters.source}
          onValueChange={(v) => setSource(v as EventSourceFilter)}
        >
          <TabsList className="h-9">
            {SOURCE_ITEMS.map((item) => (
              <TabsTrigger key={item.value} value={item.value} className="text-xs">
                {item.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        <div className="flex items-center gap-2">
          <NativeSelect
            value={filters.range}
            onChange={(e) => setRange(e.target.value as EventTimeRange)}
            className="h-8 text-xs"
          >
            <option value="today">今日</option>
            <option value="7d">近 7 日</option>
            <option value="30d">近 30 日</option>
          </NativeSelect>

          <div className="relative">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="标的 / 板块"
              value={filters.query}
              onChange={(e) => setQuery(e.target.value)}
              className="h-8 w-40 pl-8 text-xs lg:w-56"
            />
          </div>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <span className="text-xs text-muted-foreground">重要性</span>
        {IMPORTANCE_ITEMS.map((item) => (
          <FilterPill
            key={item.value}
            active={filters.importance === item.value}
            onClick={() =>
              setImportance(filters.importance === item.value ? "all" : item.value)
            }
          >
            {item.label}
          </FilterPill>
        ))}
      </div>
    </div>
  );
}
