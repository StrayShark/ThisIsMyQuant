import { Link, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import { api } from "@/api/client";
import { PageHeader } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { DimensionSummary } from "@/features/analysis/DimensionSummary";
import { triggerLabel } from "@/data/calendar";
import { dimensionLabel } from "@/data/dimensions";
import { getFuturesProduct } from "@/data/futures";

export function ReportDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data: report, isLoading, error } = useQuery({
    queryKey: ["report", id],
    queryFn: () => api.getReport(id!),
    enabled: Boolean(id),
  });

  const { data: followups } = useQuery({
    queryKey: ["followups", id],
    queryFn: () => api.listFollowups({ report_id: id!, limit: 50 }),
    enabled: Boolean(id),
  });

  const { data: linkedNews } = useQuery({
    queryKey: ["report-news", id, report?.news_ids],
    queryFn: () => api.listNewsByIds(report!.news_ids!),
    enabled: Boolean(report?.news_ids?.length),
  });

  const product = report ? getFuturesProduct(report.symbol) : undefined;

  return (
    <div className="page-scroll">
      <div className="page-inner max-w-3xl">
        <Button variant="ghost" size="sm" className="mb-4 -ml-2" asChild>
          <Link to="/reports">
            <ArrowLeft className="mr-1 h-4 w-4" />
            返回报告列表
          </Link>
        </Button>

        {isLoading ? (
          <Skeleton className="h-64 rounded-lg" />
        ) : error || !report ? (
          <PageHeader title="报告不存在" description="该报告可能已被删除或 ID 无效。" />
        ) : (
          <>
            <PageHeader
              title={product?.name ? `${product.name} 分析报告` : `${report.symbol} 分析报告`}
              description={new Date(report.created_at).toLocaleString("zh-CN")}
            />
            <Card>
              <CardHeader className="flex-row flex-wrap items-center gap-2 space-y-0 pb-2">
                <CardTitle className="text-sm font-semibold">报告正文</CardTitle>
                <Badge variant="secondary">{triggerLabel(report.trigger)}</Badge>
                <Badge variant="outline" className="font-mono">
                  {report.provider}
                </Badge>
                {report.anomaly_reason && (
                  <Badge variant="outline" className="text-amber-600 dark:text-amber-400">
                    异动：{report.anomaly_reason}
                  </Badge>
                )}
                {report.news_ids && report.news_ids.length > 0 && (
                  <Badge variant="outline" className="text-[10px]">
                    引用 {report.news_ids.length} 条资讯
                  </Badge>
                )}
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-xs text-muted-foreground">{report.context_summary}</p>
                {report.dimension_summary && (
                  <div className="space-y-2">
                    <h3 className="text-sm font-semibold">分维度要点</h3>
                    <DimensionSummary summary={report.dimension_summary} />
                  </div>
                )}
                <article className="whitespace-pre-wrap text-sm leading-relaxed text-foreground">
                  {report.content}
                </article>
              </CardContent>
            </Card>

            {linkedNews && linkedNews.length > 0 && (
              <Card className="mt-4">
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-semibold">关联资讯</CardTitle>
                </CardHeader>
                <CardContent>
                  <ScrollArea className="max-h-[280px]">
                    <div className="space-y-3 pr-3">
                      {linkedNews.map((item) => {
                        const primary = item.classifications[0];
                        return (
                          <article
                            key={item.id}
                            className="border-b border-border pb-3 last:border-0"
                          >
                            <div className="mb-1 flex flex-wrap gap-1.5">
                              {primary && (
                                <Badge variant="secondary" className="text-[10px]">
                                  {primary.dimension_label ||
                                    dimensionLabel(primary.dimension_code)}
                                </Badge>
                              )}
                              <span className="text-[10px] text-muted-foreground">
                                {item.display_time}
                              </span>
                            </div>
                            <p className="text-sm font-medium">{item.title}</p>
                            {item.summary && (
                              <p className="mt-1 text-xs text-muted-foreground">{item.summary}</p>
                            )}
                          </article>
                        );
                      })}
                    </div>
                  </ScrollArea>
                </CardContent>
              </Card>
            )}

            {followups && followups.length > 0 && (
              <Card className="mt-4">
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-semibold">追问记录</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {followups.map((f) => (
                    <div key={f.id} className="border-b border-border pb-3 last:border-0">
                      <p className="text-sm font-medium text-foreground">问：{f.question}</p>
                      <p className="mt-2 whitespace-pre-wrap text-sm leading-relaxed text-muted-foreground">
                        {f.answer}
                      </p>
                      <p className="mt-2 text-[10px] text-muted-foreground">
                        {new Date(f.created_at).toLocaleString("zh-CN")} · {f.provider}
                      </p>
                    </div>
                  ))}
                </CardContent>
              </Card>
            )}

            <Button
              variant="outline"
              className="mt-6"
              onClick={() => {
                navigate("/");
              }}
            >
              在行情页查看该品种
            </Button>
          </>
        )}
      </div>
    </div>
  );
}
