import { useQuery } from "@tanstack/react-query";
import { Activity, BellRing, FileText, Link2, Newspaper, SlidersHorizontal } from "lucide-react";
import { api } from "@/api/client";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { ProfessionalDashboard } from "@/types";

const impactText: Record<string, string> = {
  bullish: "利多",
  bearish: "利空",
  neutral: "中性",
};

const signalText: Record<string, string> = {
  bullish: "偏多",
  bearish: "偏空",
  neutral: "中性",
  tracked: "跟踪",
  watch: "观察",
  pending: "待接",
};

function toneClass(tone: string) {
  if (tone === "bullish" || tone === "high") return "text-up";
  if (tone === "bearish") return "text-down";
  if (tone === "medium") return "text-amber-300";
  return "text-muted-foreground";
}

function SectionTitle({
  icon: Icon,
  title,
}: {
  icon: typeof Newspaper;
  title: string;
}) {
  return (
    <div className="mb-2 flex items-center gap-2 text-xs font-semibold text-muted-foreground">
      <Icon className="h-3.5 w-3.5" />
      <span>{title}</span>
    </div>
  );
}

function EmptyLine() {
  return <div className="text-xs text-muted-foreground">暂无数据，等待数据轮询或手动触发。</div>;
}

export function ProfessionalWorkbench() {
  const { data, isLoading } = useQuery({
    queryKey: ["professional-dashboard"],
    queryFn: () => api.getProfessionalDashboard(),
    staleTime: 60_000,
    refetchInterval: 120_000,
  });

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-sm font-semibold">
          <SlidersHorizontal className="h-4 w-4 text-primary" />
          专业分析工作台
        </CardTitle>
      </CardHeader>
      <CardContent>
        {isLoading || !data ? (
          <div className="grid gap-3 lg:grid-cols-5">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-32 w-full rounded-md" />
            ))}
          </div>
        ) : (
          <WorkbenchGrid data={data} />
        )}
      </CardContent>
    </Card>
  );
}

function WorkbenchGrid({ data }: { data: ProfessionalDashboard }) {
  return (
    <div className="grid gap-3 xl:grid-cols-5">
      <section className="min-w-0 rounded-md border border-border/70 bg-background/40 p-3">
        <SectionTitle icon={Newspaper} title="资讯决策流" />
        <div className="space-y-2">
          {data.decision_flow.slice(0, 3).map((item) => (
            <div key={item.id} className="min-w-0 border-b border-border/60 pb-2 last:border-0">
              <div className="flex items-center justify-between gap-2">
                <span className="truncate text-xs font-medium">{item.product_name ?? item.symbol ?? "全市场"}</span>
                <span className={`shrink-0 text-[11px] ${toneClass(item.impact)}`}>
                  {impactText[item.impact] ?? item.impact}
                </span>
              </div>
              <div className="mt-1 line-clamp-2 text-xs text-foreground">{item.title}</div>
              <div className="mt-1 truncate text-[11px] text-muted-foreground">
                {item.sector ?? "宏观"} · {item.dimension_label ?? "未分类"} · {(item.confidence * 100).toFixed(0)}%
              </div>
            </div>
          ))}
          {data.decision_flow.length === 0 ? <EmptyLine /> : null}
        </div>
      </section>

      <section className="min-w-0 rounded-md border border-border/70 bg-background/40 p-3">
        <SectionTitle icon={Activity} title="产业因子" />
        <div className="space-y-2">
          {data.factors.slice(0, 3).map((factor) => (
            <div key={factor.symbol} className="min-w-0 border-b border-border/60 pb-2 last:border-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-xs font-medium">{factor.product_name}</span>
                <span className="text-[11px] text-muted-foreground">{factor.quality}</span>
              </div>
              {factor.signals.slice(0, 2).map((signal) => (
                <div key={`${factor.symbol}-${signal.label}`} className="mt-1 flex items-center justify-between gap-2 text-[11px]">
                  <span className="truncate text-muted-foreground">{signal.label}</span>
                  <span className={`shrink-0 ${toneClass(signal.signal)}`}>
                    {signalText[signal.signal] ?? signal.value} · {signal.value}
                  </span>
                </div>
              ))}
            </div>
          ))}
          {data.factors.length === 0 ? <EmptyLine /> : null}
        </div>
      </section>

      <section className="min-w-0 rounded-md border border-border/70 bg-background/40 p-3">
        <SectionTitle icon={BellRing} title="异动预警" />
        <div className="space-y-2">
          {data.alerts.slice(0, 3).map((alert) => (
            <div key={`${alert.symbol}-${alert.timestamp}`} className="border-b border-border/60 pb-2 last:border-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-xs font-medium">{alert.product_name}</span>
                <span className={`text-[11px] ${toneClass(alert.severity)}`}>{alert.severity}</span>
              </div>
              <div className="mt-1 line-clamp-2 text-xs text-foreground">{alert.reason}</div>
              <div className="mt-1 text-[11px] text-muted-foreground">{alert.sector}</div>
            </div>
          ))}
          {data.alerts.length === 0 ? <EmptyLine /> : null}
        </div>
      </section>

      <section className="min-w-0 rounded-md border border-border/70 bg-background/40 p-3">
        <SectionTitle icon={FileText} title="报告流程" />
        <div className="space-y-2">
          {data.report_workflow.slice(0, 4).map((item) => (
            <div key={item.trigger} className="flex items-start justify-between gap-2 border-b border-border/60 pb-2 last:border-0">
              <div className="min-w-0">
                <div className="text-xs font-medium">{item.label}</div>
                <div className="mt-1 truncate text-[11px] text-muted-foreground">{item.symbol ?? item.summary}</div>
              </div>
              <span className={`shrink-0 text-[11px] ${item.status === "ready" ? "text-up" : "text-muted-foreground"}`}>
                {item.status === "ready" ? "已生成" : item.status === "running" ? "生成中" : "待触发"}
              </span>
            </div>
          ))}
        </div>
      </section>

      <section className="min-w-0 rounded-md border border-border/70 bg-background/40 p-3">
        <SectionTitle icon={Link2} title="外盘联动" />
        <div className="space-y-2">
          {data.overseas_links.slice(0, 3).map((link) => (
            <div key={`${link.local_symbol}-${link.overseas_symbol}`} className="border-b border-border/60 pb-2 last:border-0">
              <div className="flex items-center justify-between gap-2">
                <span className="text-xs font-medium">{link.local_name}</span>
                <span className="text-[11px] text-muted-foreground">{link.overseas_symbol}</span>
              </div>
              <div className="mt-1 line-clamp-2 text-xs text-foreground">{link.transmission}</div>
              <div className="mt-1 text-[11px] text-muted-foreground">{link.driver} · {link.status}</div>
            </div>
          ))}
          {data.overseas_links.length === 0 ? <EmptyLine /> : null}
        </div>
      </section>
    </div>
  );
}
