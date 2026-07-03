import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, BarChart3, Sparkles } from "lucide-react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { AnalysisTimeline, defaultTimelineSteps } from "@/components/AnalysisTimeline";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AiPanel } from "@/features/analysis/AiPanel";
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

  useEffect(() => {
    setCurrentSymbol(symbol);
    void ensureMarketSubscription(symbol);
  }, [symbol, setCurrentSymbol]);

  const related = (ctx?.related_products as Array<{ symbol: string; name: string }>) ?? [];

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <div className="mb-4 flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={() => navigate("/symbols")}>
            <ArrowLeft className="mr-1 h-4 w-4" />
            品种列表
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
            打开行情页
          </Button>
          <Button
            variant="outline"
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

        <div className="grid gap-4 lg:grid-cols-3">
          <div className="space-y-4 lg:col-span-2">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">多周期缩略图</CardTitle>
              </CardHeader>
              <CardContent className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                {PERIODS.map((p) => (
                  <MiniChart key={p.interval} symbol={symbol} interval={p.interval} label={p.label} />
                ))}
              </CardContent>
            </Card>

            <ReportTimeline symbol={symbol} />

            {related.length > 0 && (
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
            )}
          </div>

          <div className="space-y-4">
            <AnalysisTimeline steps={defaultTimelineSteps("done")} />
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold">板块驱动</CardTitle>
              </CardHeader>
              <CardContent className="flex flex-wrap gap-1.5">
                {((ctx?.drivers as string[]) ?? []).map((d) => (
                  <Badge key={d} variant="outline" className="text-[10px]">
                    {d}
                  </Badge>
                ))}
              </CardContent>
            </Card>
            <AiPanel />
          </div>
        </div>
      </div>
    </div>
  );
}
