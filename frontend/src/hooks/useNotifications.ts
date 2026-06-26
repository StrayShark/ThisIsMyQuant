import { useEffect } from "react";
import { useAppStore } from "@/app/store";

interface NotificationPayload {
  msg_type?: string;
  level?: string;
  title: string;
  body: string;
  link?: string | null;
}

/** 监听后端 notification 事件并 Toast 提示。 */
export function useNotifications() {
  const showToast = useAppStore((s) => s.showToast);

  useEffect(() => {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;

    let unlisten: (() => void) | undefined;

    void import("@tauri-apps/api/event").then(({ listen }) => {
      listen<NotificationPayload>("notification", (e) => {
        const { title, body } = e.payload;
        showToast(`${title} — ${body}`);
      }).then((fn) => {
        unlisten = fn;
      });
    });

    return () => unlisten?.();
  }, [showToast]);
}
