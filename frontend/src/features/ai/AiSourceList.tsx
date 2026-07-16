import { useNavigate } from "react-router-dom";
import {
  Newspaper,
  Calendar,
  LineChart,
  Briefcase,
  FileText,
  Building2,
  ExternalLink,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { AiSource } from "@/types";

interface AiSourceListProps {
  sources: AiSource[];
  className?: string;
}

const ICONS: Record<AiSource["type"], typeof Newspaper> = {
  quote: LineChart,
  news: Newspaper,
  calendar: Calendar,
  financial: Building2,
  position: Briefcase,
  report: FileText,
};

const TYPE_LABELS: Record<AiSource["type"], string> = {
  quote: "行情",
  news: "资讯",
  calendar: "日历",
  financial: "财务",
  position: "持仓",
  report: "研报",
};

function formatTime(displayTime?: string | null) {
  if (!displayTime) return null;
  try {
    return new Date(displayTime).toLocaleString("zh-CN", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return displayTime;
  }
}

function isStockSymbol(id: string) {
  return /^\d{6}(\.(SH|SZ|BJ))?$/i.test(id);
}

function sourcePath(source: AiSource): string | null {
  if (!source.id) {
    if (source.type === "news" || source.type === "calendar") return "/events";
    if (source.type === "position") return "/simulation";
    return null;
  }
  const id = encodeURIComponent(source.id);
  switch (source.type) {
    case "quote":
      return isStockSymbol(source.id) ? `/markets/stocks/${id}` : `/markets/futures/${id}`;
    case "financial":
      return `/markets/stocks/${id}`;
    case "report":
      return `/reports/${id}`;
    case "news":
    case "calendar":
      return "/events";
    case "position":
      return "/simulation";
    default:
      return null;
  }
}

export function AiSourceList({ sources, className }: AiSourceListProps) {
  const navigate = useNavigate();
  if (!sources || sources.length === 0) return null;

  return (
    <div className={cn("space-y-2", className)}>
      <h4 className="text-xs font-medium text-muted-foreground">数据来源</h4>
      <div className="flex flex-col gap-2">
        {sources.map((source, index) => {
          const Icon = ICONS[source.type];
          const internalPath = sourcePath(source);
          return (
            <button
              key={`${source.id ?? source.title}-${index}`}
              type="button"
              onClick={() => {
                if (source.url) {
                  window.open(source.url, "_blank", "noopener,noreferrer");
                } else if (internalPath) {
                  navigate(internalPath);
                }
              }}
              className={cn(
                "flex items-start gap-2 rounded-xl border border-border bg-muted/20 p-2.5 text-left transition-colors",
                (source.url || internalPath) && "hover:bg-muted/40"
              )}
            >
              <Icon className="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                  <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
                    {TYPE_LABELS[source.type]}
                  </span>
                  {(source.url || internalPath) && (
                    <ExternalLink className="h-3 w-3 text-muted-foreground/60" />
                  )}
                </div>
                <p className="mt-0.5 line-clamp-2 text-xs text-foreground">{source.title}</p>
                {source.display_time && (
                  <p className="mt-0.5 text-[10px] text-muted-foreground">
                    {formatTime(source.display_time)}
                  </p>
                )}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
