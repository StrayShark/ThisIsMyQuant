import { cn } from "@/lib/utils";
import type { DataQualityStatus } from "@/types";

interface DataQualityBadgeProps {
  status: DataQualityStatus;
  label?: string;
  className?: string;
}

const statusConfig: Record<
  DataQualityStatus,
  { label: string; dot: string; bg: string; text: string }
> = {
  live: {
    label: "实时",
    dot: "bg-green-500",
    bg: "bg-green-500/10",
    text: "text-green-600",
  },
  history: {
    label: "历史",
    dot: "bg-blue-500",
    bg: "bg-blue-500/10",
    text: "text-blue-600",
  },
  stale: {
    label: "陈旧",
    dot: "bg-amber-500",
    bg: "bg-amber-500/10",
    text: "text-amber-600",
  },
  error: {
    label: "错误",
    dot: "bg-red-500",
    bg: "bg-red-500/10",
    text: "text-red-600",
  },
  pending: {
    label: "待更新",
    dot: "bg-slate-400",
    bg: "bg-slate-400/10",
    text: "text-slate-600",
  },
  estimated: {
    label: "估算",
    dot: "bg-purple-500",
    bg: "bg-purple-500/10",
    text: "text-purple-600",
  },
  reference: {
    label: "参考",
    dot: "bg-cyan-500",
    bg: "bg-cyan-500/10",
    text: "text-cyan-600",
  },
  local: {
    label: "本地",
    dot: "bg-emerald-500",
    bg: "bg-emerald-500/10",
    text: "text-emerald-600",
  },
};

export function DataQualityBadge({ status, label, className }: DataQualityBadgeProps) {
  const config = statusConfig[status] ?? statusConfig.pending;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium",
        config.bg,
        config.text,
        className
      )}
    >
      <span className={cn("h-1.5 w-1.5 rounded-full", config.dot)} />
      {label ?? config.label}
    </span>
  );
}
