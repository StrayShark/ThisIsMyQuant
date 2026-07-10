import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, BarChart3, BellRing, Bookmark, Sparkles } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AiPanel } from "@/features/analysis/AiPanel";
import { DimensionSummary } from "@/features/analysis/DimensionSummary";
import { MiniChart } from "@/features/symbol/MiniChart";
import { ReportTimeline } from "@/features/symbol/ReportTimeline";
import { ensureMarketSubscription } from "@/lib/market-subscribe";
import type { Interval } from "@/types";

const PERIODS: { interval: Interval; label: string }[] = [
  { interval: "1m", label: "1m" },
  { interval: "5m", label: "5m" },
  { interval: "15m", label: "15m" },
  { interval: "1h", label: "1h" },
  { interval: "1d", label: "1d" },
];

export function SymbolDetailPage() {
  const { symbol: routeSymbol } = useParams<{ symbol: string }>();
  const navigate = useNavigate();
  const setCurrentSymbol = useAppStore((s) => s.setCurrentSymbol);
  const symbol = (routeSymbol ?? "RB0").toUpperCase();
  const [analyzing, setAnalyzing] = useState(false);

  const { data: ctx } = useQuery({
    queryKey: ["symbol-context", symbol],
    queryFn: () => api.getSymbolContext(symbol),
  });

  const { data: reports } = useQuery({
    queryKey: ["reports", symbol],
    queryFn: () => api.listReports({ symbol: symbol.toLowerCase(), limit: 5 }),
  });

  useEffect(() => {
    setCurrentSymbol(symbol);
    void ensureMarketSubscription(symbol);
  }, [symbol, setCurrentSymbol]);

  const related = (ctx?.related_products as Array<{ symbol: string; name: string }>) ?? [];

  return (
    <div className="page-scroll">
      <div className="page-inner">
        {/* 1. 详情 Header */}
        <div className="mb-4 flex flex-wrap items-center justify-between gap-3 rounded-lg border p-4">
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="icon" onClick={() => navigate("/symbols")} aria-label="返回">
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-xl font-semibold">{(ctx?.product_name as string) ?? symbol}</h1>
                <Badge variant="outline" className="font-mono text-xs">
                  {symbol}
                </Badge>
                <Badge variant="secondary" className="text-xs">
                  {(ctx?.name as string) ?? "品种详情"}
                </Badge>
              </div>
              <p className="mt-0.5 text-xs text-muted-foreground">
                数据状态：<span className="text-green-500">live</span> · 主力连续
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" aria-label="收藏">
              <Bookmark className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm" aria-label="预警">
              <BellRing className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                setCurrentSymbol(symbol);
                navigate("/workspace");
              }}
            >
              <BarChart3 className="mr-1 h-4 w-4" />
              行情页
            </Button>
            <Button
              variant="default"
              size="sm"
              disabled={analyzing}
              onClick={async () => {
                setAnalyzing(true);
                try {
                  const r = await api.triggerAnalysis({ symbol, trigger: "manual" });
                  navigate(`/reports/${r.report_id}`);
                } finally {
                  setAnalyzing(false);
                }
              }}
            >
              <Sparkles className="mr-1 h-4 w-4" />
              {analyzing ? "分析中…" : "生成分析"}
            </Button>
          </div>
        </div>

        {/* 2. 上区：多周期图表 + 驱动解释 */}
        <div className="mb-4 grid gap-4 lg:grid-cols-3">
          <div className="space-y-4 lg:col-span-2">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">多周期价格行为</CardTitle>
              </CardHeader>
              <CardContent className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                {PERIODS.map((p) => (
                  <MiniChart key={p.interval} symbol={symbol} interval={p.interval} label={p.label} />
                ))}
              </CardContent>
            </Card>
          </div>

          <div className="space-y-4">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">驱动解释</CardTitle>
              </CardHeader>
              <CardContent>
                {reports && reports.length > 0 ? (
                  <DimensionSummary summary={reports[0].dimension_summary} compact />
                ) : (
                  <div className="text-sm text-muted-foreground">暂无报告驱动解释，点击右上角生成分析。</div>
                )}
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">板块驱动</CardTitle>
              </CardHeader>
              <CardContent className="flex flex-wrap gap-1.5">
                {(((ctx?.drivers as string[]) ?? []).filter(Boolean)).map((d) => (
                  <Badge key={d} variant="outline" className="text-[10px]">
                    {d}
                  </Badge>
                ))}
              </CardContent>
            </Card>
          </div>
        </div>

        {/* 3. 下区：关联品种 + 报告时间线 + 追问记录 */}
        <Tabs defaultValue="reports" className="mb-4">
          <TabsList>
            <TabsTrigger value="reports">报告时间线</TabsTrigger>
            <TabsTrigger value="related">关联品种</TabsTrigger>
            <TabsTrigger value="copilot">追问记录</TabsTrigger>
          </TabsList>
          <TabsContent value="reports">
            <ReportTimeline symbol={symbol} />
          </TabsContent>
          <TabsContent value="related">
            <RelatedProducts related={related} />
          </TabsContent>
          <TabsContent value="copilot">
            <AiPanel />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

function RelatedProducts({ related }: { related: Array<{ symbol: string; name: string }> }) {
  if (related.length === 0) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-sm text-muted-foreground">暂无关联品种数据</CardContent>
      </Card>
    );
  }
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">同板块关联品种</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-wrap gap-2">
        {related.map((p) => (
          <Link
            key={p.symbol}
            to={`/symbols/${p.symbol}`}
            className="rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted/30"
          >
            {p.name}
            <span className="ml-1 font-mono text-xs text-muted-foreground">{p.symbol}</span>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
