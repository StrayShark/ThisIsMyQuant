import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Newspaper, RefreshCw } from "lucide-react";
import { api } from "@/api/client";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FilterPill } from "@/components/ui/filter-pill";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";

const impactLabels: Record<string, string> = {
  bullish: "利多",
  bearish: "利空",
  neutral: "中性",
};

function impactVariant(impact: string): "up" | "down" | "secondary" {
  if (impact === "bullish") return "up";
  if (impact === "bearish") return "down";
  return "secondary";
}

export function NewsDecisionPage() {
  const [impact, setImpact] = useState("全部");
  const [keyword, setKeyword] = useState("");
  const queryClient = useQueryClient();
  const { data, isLoading } = useQuery({
    queryKey: ["news-decision"],
    queryFn: () => api.getProfessionalDashboard(),
    refetchInterval: 60_000,
  });

  const reclassify = useMutation({
    mutationFn: (id: string) => api.reclassifyNews({ news_ids: [id], use_llm: true }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["news-decision"] });
      void queryClient.invalidateQueries({ queryKey: ["professional-dashboard"] });
    },
  });

  const list = useMemo(() => {
    const kw = keyword.trim().toLowerCase();
    return (data?.decision_flow ?? []).filter((item) => {
      if (impact !== "全部" && item.impact !== impact) return false;
      if (!kw) return true;
      return (
        item.title.toLowerCase().includes(kw) ||
        item.summary.toLowerCase().includes(kw) ||
        (item.product_name ?? "").toLowerCase().includes(kw) ||
        (item.symbol ?? "").toLowerCase().includes(kw)
      );
    });
  }, [data?.decision_flow, impact, keyword]);

  return (
    <div className="page-scroll">
      <div className="page-inner space-y-4">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-normal">资讯决策中心</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              将快讯归因到品种、维度、利多利空和置信度，形成可追踪的研究线索。
            </p>
          </div>
          <Input
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            placeholder="搜索新闻、品种、代码"
            className="w-[220px]"
          />
        </div>

        <div className="flex flex-wrap gap-1.5">
          {["全部", "bullish", "bearish", "neutral"].map((v) => (
            <FilterPill key={v} active={impact === v} onClick={() => setImpact(v)}>
              {impactLabels[v] ?? v}
            </FilterPill>
          ))}
        </div>

        {isLoading ? (
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <Skeleton key={i} className="h-32 rounded-lg" />
            ))}
          </div>
        ) : list.length > 0 ? (
          <div className="grid gap-4 xl:grid-cols-2">
            {list.map((item) => (
              <Card key={item.id || item.title}>
                <CardHeader className="pb-2">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <CardTitle className="line-clamp-2 text-base">{item.title}</CardTitle>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {item.source} · {new Date(item.display_time).toLocaleString("zh-CN")}
                      </p>
                    </div>
                    <Badge variant={impactVariant(item.impact)}>
                      {impactLabels[item.impact] ?? item.impact}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  <p className="text-sm leading-relaxed text-muted-foreground">{item.summary}</p>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="outline">{item.sector ?? "全市场"}</Badge>
                    <Badge variant="secondary">
                      {item.product_name ?? item.symbol ?? "未归因"}
                    </Badge>
                    <Badge variant="secondary">{item.dimension_label ?? "未分类"}</Badge>
                    <Badge variant="secondary">
                      置信度 {(item.confidence * 100).toFixed(0)}%
                    </Badge>
                  </div>
                  {item.id && (
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={reclassify.isPending}
                      onClick={() => reclassify.mutate(item.id)}
                    >
                      <RefreshCw className="h-3.5 w-3.5" />
                      重新分类
                    </Button>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={Newspaper}
            title="暂无匹配资讯"
            description="请调整影响方向或关键词，或确认金十资讯轮询已开启。"
          />
        )}
      </div>
    </div>
  );
}
