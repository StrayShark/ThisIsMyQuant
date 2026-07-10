import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Sparkles } from "lucide-react";
import { useAppStore } from "@/app/store";
import { api } from "@/api/client";
import { AiPanel } from "@/features/analysis/AiPanel";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FilterPill } from "@/components/ui/filter-pill";
import { Badge } from "@/components/ui/badge";

export function CopilotPage() {
  const currentSymbol = useAppStore((s) => s.currentSymbol);
  const setCurrentSymbol = useAppStore((s) => s.setCurrentSymbol);
  const { data: sectors } = useQuery({
    queryKey: ["products", "copilot"],
    queryFn: () => api.listProducts({ tier: "core" }),
  });
  const { data: dashboard } = useQuery({
    queryKey: ["professional-dashboard", "copilot"],
    queryFn: () => api.getProfessionalDashboard(),
    refetchInterval: 120_000,
  });

  const products = useMemo(
    () => (sectors ?? []).flatMap((s) => s.products.map((p) => ({ ...p, sector: s.name }))),
    [sectors]
  );
  const active = products.find((p) => p.symbol.toUpperCase() === currentSymbol.toUpperCase());
  const relatedNews = dashboard?.decision_flow.filter((n) => n.symbol === currentSymbol) ?? [];
  const factor = dashboard?.factors.find((f) => f.symbol === currentSymbol);

  return (
    <div className="page-scroll">
      <div className="page-inner space-y-4">
        <div>
          <h1 className="flex items-center gap-2 text-xl font-semibold tracking-normal">
            <Sparkles className="h-5 w-5 text-primary" />
            Copilot 研究助手
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            绑定当前品种、报告、新闻与因子上下文，生成分析或继续追问证据链。
          </p>
        </div>

        <div className="flex flex-wrap gap-1.5">
          {products.slice(0, 18).map((p) => (
            <FilterPill
              key={p.symbol}
              active={currentSymbol.toUpperCase() === p.symbol.toUpperCase()}
              onClick={() => setCurrentSymbol(p.symbol)}
            >
              {p.name}
            </FilterPill>
          ))}
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_380px]">
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>当前上下文</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex flex-wrap gap-2">
                  <Badge variant="outline">{active?.sector ?? "未分类"}</Badge>
                  <Badge variant="secondary">{active?.name ?? currentSymbol}</Badge>
                  <Badge variant="secondary" className="font-mono">
                    {currentSymbol}
                  </Badge>
                  <Badge variant={factor?.quality === "pending" ? "outline" : "secondary"}>
                    因子 {factor?.quality ?? "待回填"}
                  </Badge>
                </div>
                <div className="grid gap-3 md:grid-cols-3">
                  <div className="rounded-md border border-border p-3">
                    <p className="text-xs text-muted-foreground">快捷问题</p>
                    <p className="mt-2 text-sm">为什么涨跌？关键风险在哪？明天看什么？</p>
                  </div>
                  <div className="rounded-md border border-border p-3">
                    <p className="text-xs text-muted-foreground">关联资讯</p>
                    <p className="mt-2 text-sm">{relatedNews.length} 条已归因资讯</p>
                  </div>
                  <div className="rounded-md border border-border p-3">
                    <p className="text-xs text-muted-foreground">因子快照</p>
                    <p className="mt-2 text-sm">
                      {factor?.signals.map((s) => s.label).join(" / ") || "等待行情与因子回填"}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>证据链</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {relatedNews.slice(0, 3).map((item) => (
                  <article key={item.id} className="border-b border-border pb-3 last:border-0">
                    <div className="flex flex-wrap gap-2">
                      <Badge variant={item.impact === "bullish" ? "up" : item.impact === "bearish" ? "down" : "secondary"}>
                        {item.impact}
                      </Badge>
                      <Badge variant="secondary">{item.dimension_label ?? "未分类"}</Badge>
                    </div>
                    <p className="mt-2 text-sm font-medium">{item.title}</p>
                    <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{item.summary}</p>
                  </article>
                ))}
                {relatedNews.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    暂无当前品种的已归因资讯，Copilot 将主要使用行情、报告和板块上下文。
                  </p>
                )}
              </CardContent>
            </Card>
          </div>

          <AiPanel />
        </div>
      </div>
    </div>
  );
}
