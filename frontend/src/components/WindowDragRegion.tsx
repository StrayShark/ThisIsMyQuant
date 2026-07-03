import { useCallback } from "react";
import { cn } from "@/lib/utils";
import { isTauriRuntime } from "@/lib/platform";

/** Tauri 窗口拖拽 / 双击最大化区域（置于交互控件下层）。 */
export function WindowDragRegion({ className }: { className?: string }) {
  const onMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (!isTauriRuntime() || e.button !== 0) return;
    e.preventDefault();
    void import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      const win = getCurrentWindow();
      if (e.detail === 2) {
        void win.toggleMaximize();
      } else {
        void win.startDragging();
      }
    });
  }, []);

  if (!isTauriRuntime()) return null;

  return (
    <div
      className={cn("absolute inset-0 z-0 cursor-default select-none", className)}
      data-tauri-drag-region
      onMouseDown={onMouseDown}
      aria-hidden
    />
  );
}
