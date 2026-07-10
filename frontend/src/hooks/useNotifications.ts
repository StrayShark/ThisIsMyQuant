import { useEffect } from "react";
import { useAppStore } from "@/app/store";

interface NotificationPayload {
  msg_type?: string;
  level?: string;
  title: string;
  body: string;
  link?: string | null;
}

interface PositionRiskImpactPayload {
  symbol: string;
  account_id: string;
  account_name: string;
  position_side: string;
  position_qty: number;
  avg_price: number;
  current_price: number;
  unrealized_pnl: number;
  pnl_change_if_hit: number;
  risk_ratio: number;
  description: string;
}

/** 监听后端 notification / anomaly-position-risk 事件并 Toast 提示。 */
export function useNotifications() {
  const showToast = useAppStore((s) => s.showToast);

  useEffect(() => {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;

    let unlistenNotification: (() => void) | undefined;
    let unlistenRisk: (() => void) | undefined;

    void import("@tauri-apps/api/event").then(({ listen }) => {
      listen<NotificationPayload>("notification", (e) => {
        const { title, body } = e.payload;
        showToast(`${title} — ${body}`);
      }).then((fn) => {
        unlistenNotification = fn;
      });

      listen<PositionRiskImpactPayload>("anomaly-position-risk", (e) => {
        const p = e.payload;
        showToast(`异动持仓联动：${p.symbol} 影响账户 ${p.account_name} — ${p.description}`);
      }).then((fn) => {
        unlistenRisk = fn;
      });
    });

    return () => {
      unlistenNotification?.();
      unlistenRisk?.();
    };
  }, [showToast]);
}
