import * as Dialog from "@radix-ui/react-dialog";
import { X, Bell, Plus, Calendar } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { useAppStore } from "@/app/store";
import { EventImpactTags } from "./EventImpactTags";
import { EventAiAnalysisButton } from "./EventAiAnalysisButton";
import type { EventSource, EventImportance, MarketEvent } from "@/types";

interface EventDetailDrawerProps {
  event: MarketEvent | null;
  open: boolean;
  onClose: () => void;
  onSymbolClick?: (symbol: string) => void;
  onSectorClick?: (sector: string) => void;
}

const SOURCE_LABELS: Record<EventSource, string> = {
  jin10: "金十",
  calendar: "财经日历",
  announcement: "公告",
  earnings: "财报",
  industry: "产业",
};

const IMPORTANCE_CONFIG: Record<
  EventImportance,
  { label: string; color: string }
> = {
  high: { label: "高", color: "bg-red-500/10 text-red-600 border-red-500/20" },
  medium: {
    label: "中",
    color: "bg-amber-500/10 text-amber-600 border-amber-500/20",
  },
  low: {
    label: "低",
    color: "bg-slate-500/10 text-slate-600 border-slate-500/20",
  },
};

const DIRECTION_CONFIG: Record<
  Exclude<MarketEvent["direction"], null | undefined>,
  { label: string; variant: "up" | "down" | "secondary" }
> = {
  bullish: { label: "偏多", variant: "up" },
  bearish: { label: "偏空", variant: "down" },
  neutral: { label: "中性", variant: "secondary" },
};

export function EventDetailDrawer({
  event,
  open,
  onClose,
  onSymbolClick,
  onSectorClick,
}: EventDetailDrawerProps) {
  const showToast = useAppStore((s) => s.showToast);

  if (!event) return null;

  const importance = IMPORTANCE_CONFIG[event.importance];
  const direction = event.direction ? DIRECTION_CONFIG[event.direction] : null;

  const handleAddToWatchlist = () => {
    showToast(`已将 ${event.affected_symbols.join(", ") || "相关标的"} 加入关注（占位）`);
  };

  const handleSetReminder = () => {
    showToast("已设置事件提醒（占位）");
  };

  return (
    <Dialog.Root open={open} onOpenChange={(v) => !v && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
        <Dialog.Content
          className={cn(
            "fixed right-0 top-0 z-50 h-full w-full max-w-md border-l border-border bg-background shadow-xl",
            "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-right-full"
          )}
        >
          <div className="flex h-full flex-col">
            <div className="flex items-start justify-between border-b border-border p-5">
              <div>
                <Dialog.Title className="text-base font-semibold text-foreground">
                  事件详情
                </Dialog.Title>
                <p className="mt-0.5 text-xs text-muted-foreground">
                  {SOURCE_LABELS[event.source]} · {event.event_type}
                </p>
              </div>
              <Dialog.Close asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8 rounded-full">
                  <X className="h-4 w-4" />
                  <span className="sr-only">关闭</span>
                </Button>
              </Dialog.Close>
            </div>

            <div className="flex-1 space-y-5 overflow-y-auto p-5">
              <div className="space-y-2">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge
                    variant="outline"
                    className={cn("text-[11px] font-medium", importance.color)}
                  >
                    重要性 {importance.label}
                  </Badge>
                  {direction && (
                    <Badge variant={direction.variant} className="text-[11px]">
                      {direction.label}
                    </Badge>
                  )}
                </div>
                <h2 className="text-lg font-semibold leading-snug text-foreground">
                  {event.title}
                </h2>
                <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Calendar className="h-3.5 w-3.5" />
                  {new Date(event.display_time).toLocaleString("zh-CN", {
                    month: "short",
                    day: "numeric",
                    hour: "2-digit",
                    minute: "2-digit",
                    weekday: "short",
                  })}
                </div>
              </div>

              {event.summary && (
                <div className="rounded-xl border border-border bg-muted/30 p-3">
                  <p className="text-sm leading-relaxed text-foreground">
                    {event.summary}
                  </p>
                </div>
              )}

              <div>
                <h3 className="mb-2 text-xs font-medium text-muted-foreground">
                  影响范围
                </h3>
                <EventImpactTags
                  event={event}
                  onSymbolClick={onSymbolClick}
                  onSectorClick={onSectorClick}
                />
              </div>

              <Separator />

              <EventAiAnalysisButton event={event} />
            </div>

            <div className="border-t border-border p-4">
              <div className="grid grid-cols-2 gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="gap-1.5"
                  onClick={handleAddToWatchlist}
                >
                  <Plus className="h-3.5 w-3.5" />
                  关联自选
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="gap-1.5"
                  onClick={handleSetReminder}
                >
                  <Bell className="h-3.5 w-3.5" />
                  设置提醒
                </Button>
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
