import { cn } from "@/lib/utils";

interface DataQualityBadgeProps {
  status: string;
  message?: string | null;
  lastSuccessAt?: string | null;
}

export function DataQualityBadge({ status, message, lastSuccessAt }: DataQualityBadgeProps) {
  const color =
    status === "available" || status === "live"
      ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20"
      : status === "stale" || status === "error"
      ? "bg-rose-500/10 text-rose-500 border-rose-500/20"
      : "bg-amber-500/10 text-amber-500 border-amber-500/20";

  const label =
    status === "available" || status === "live"
      ? "数据可用"
      : status === "stale"
      ? "数据陈旧"
      : status === "error"
      ? "数据错误"
      : "待更新";

  return (
    <div className={cn("inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-[11px]", color)}>
      <span className="h-1.5 w-1.5 rounded-full bg-current" />
      <span>{label}</span>
      {message && <span className="text-muted-foreground">· {message}</span>}
      {lastSuccessAt && !message && <span className="text-muted-foreground">· {lastSuccessAt}</span>}
    </div>
  );
}
