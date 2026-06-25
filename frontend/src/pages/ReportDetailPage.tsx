import { Link, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import { api } from "@/api/client";
import { PageHeader } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { getFuturesProduct } from "@/data/futures";
import { DimensionSummary } from "@/features/analysis/DimensionSummary";

export function ReportDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data: report, isLoading, error } = useQuery({
    queryKey: ["report", id],
    queryFn: () => api.getReport(id!),
    enabled: Boolean(id),
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
              <CardHeader className="flex-row items-center gap-2 space-y-0 pb-2">
                <CardTitle className="text-sm font-semibold">报告正文</CardTitle>
                <Badge variant="secondary">{report.trigger}</Badge>
                <Badge variant="outline" className="font-mono">
                  {report.provider}
                </Badge>
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
