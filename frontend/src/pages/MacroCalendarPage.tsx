import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { CalendarClock } from "lucide-react";
import { api } from "@/api/client";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FilterPill } from "@/components/ui/filter-pill";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { dimensionForCalendarCountry, uniqueCountries } from "@/data/calendar";
import { dimensionLabel } from "@/data/dimensions";

function todayOffset(days: number) {
  const d = new Date();
  d.setDate(d.getDate() + days);
  return d.toISOString().slice(0, 10);
}

function stars(n: number) {
  return "★".repeat(Math.min(5, Math.max(1, n)));
}

export function MacroCalendarPage() {
  const [minStar, setMinStar] = useState(3);
  const [country, setCountry] = useState<string | null>(null);
  const [keyword, setKeyword] = useState("");
  const [start, setStart] = useState(todayOffset(0));
  const [end, setEnd] = useState(todayOffset(14));

  const { data: events, isLoading } = useQuery({
    queryKey: ["macro-calendar-page", start, end, minStar, country, keyword],
    queryFn: () =>
      api.listCalendarEvents({
        start,
        end,
        min_star: minStar,
        country: country ?? undefined,
        keyword: keyword.trim() || undefined,
      }),
    refetchInterval: 300_000,
  });

  const countries = useMemo(() => uniqueCountries(events ?? []), [events]);

  return (
    <div className="page-scroll">
      <div className="page-inner space-y-4">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-normal">日历与宏观</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              高星宏观事件按品种影响维度组织，自动进入盘前和收盘报告上下文。
            </p>
          </div>
          <div className="flex flex-wrap items-end gap-2">
            <Input type="date" value={start} onChange={(e) => setStart(e.target.value)} />
            <Input type="date" value={end} onChange={(e) => setEnd(e.target.value)} />
            <Input
              value={keyword}
              onChange={(e) => setKeyword(e.target.value)}
              placeholder="CPI / PMI / 非农"
              className="w-[170px]"
            />
          </div>
        </div>

        <div className="flex flex-wrap gap-1.5">
          {[3, 4, 5].map((star) => (
            <FilterPill key={star} active={minStar === star} onClick={() => setMinStar(star)}>
              ★{star}+
            </FilterPill>
          ))}
          <FilterPill active={country === null} onClick={() => setCountry(null)}>
            全部国家
          </FilterPill>
          {countries.map((c) => (
            <FilterPill key={c} active={country === c} onClick={() => setCountry(c)}>
              {c}
            </FilterPill>
          ))}
        </div>

        {isLoading ? (
          <div className="space-y-3">
            {[1, 2, 3, 4].map((i) => (
              <Skeleton key={i} className="h-28 rounded-lg" />
            ))}
          </div>
        ) : events && events.length > 0 ? (
          <div className="grid gap-4 xl:grid-cols-2">
            {events.map((event) => {
              const dimension = dimensionForCalendarCountry(event.country);
              return (
                <Card key={event.id}>
                  <CardHeader className="pb-2">
                    <div className="flex items-start justify-between gap-3">
                      <CardTitle className="line-clamp-2 text-base">{event.name}</CardTitle>
                      <Badge variant={event.status === "released" ? "up" : "secondary"}>
                        {event.status === "released" ? "已公布" : "待公布"}
                      </Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3 text-sm">
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="outline">{event.country}</Badge>
                      <Badge variant="secondary">{stars(event.star)}</Badge>
                      <Badge variant="secondary">{dimensionLabel(dimension)}</Badge>
                      <Badge variant="secondary">{event.pub_time}</Badge>
                    </div>
                    <div className="grid gap-2 text-muted-foreground sm:grid-cols-3">
                      <p>前值：{event.previous ?? "—"}</p>
                      <p>预期：{event.consensus ?? "—"}</p>
                      <p>实际：{event.actual ?? "—"}</p>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      影响路径：{event.country.includes("美") ? "海外金融环境 → 贵金属/有色/能源" : "国内宏观 → 黑色建材/农产品需求"}
                    </p>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        ) : (
          <EmptyState
            icon={CalendarClock}
            title="暂无宏观日历"
            description="当前筛选条件下没有日历事件，或数据源暂不可用。"
          />
        )}
      </div>
    </div>
  );
}
