import { useEffect } from "react";
import { X } from "lucide-react";
import { useAppStore } from "@/app/store";
import { cn } from "@/lib/utils";

export function ToastBanner() {
  const toastMessage = useAppStore((s) => s.toastMessage);
  const clearToast = useAppStore((s) => s.clearToast);

  useEffect(() => {
    if (!toastMessage) return;
    const timer = window.setTimeout(() => clearToast(), 6000);
    return () => window.clearTimeout(timer);
  }, [toastMessage, clearToast]);

  if (!toastMessage) return null;

  return (
    <div
      className={cn(
        "pointer-events-none fixed bottom-4 left-1/2 z-[100] flex max-w-md -translate-x-1/2",
        "animate-in fade-in slide-in-from-bottom-2 duration-200"
      )}
      role="status"
    >
      <div className="pointer-events-auto flex items-start gap-2 rounded-lg border border-amber-500/40 bg-amber-500/10 px-4 py-2.5 text-sm text-foreground shadow-lg backdrop-blur-sm">
        <span className="flex-1">{toastMessage}</span>
        <button
          type="button"
          onClick={clearToast}
          className="shrink-0 rounded p-0.5 text-muted-foreground hover:text-foreground"
          aria-label="关闭"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  );
}
