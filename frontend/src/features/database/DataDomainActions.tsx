import { useMutation, useQueryClient } from "@tanstack/react-query";
import { RefreshCw, Download, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import type { DataDomainActionResult, DataDomainCode } from "@/types";
import { DOMAIN_LABELS } from "./utils";

interface DataDomainActionsProps {
  domain: DataDomainCode;
  size?: "sm" | "default";
  onComplete?: (action: "sync" | "export" | "cleanup", result: DataDomainActionResult) => void;
}

export function DataDomainActions({ domain, size = "sm", onComplete }: DataDomainActionsProps) {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);

  const actionMutation = useMutation({
    mutationFn: async ({ action }: { action: "sync" | "export" | "cleanup" }) => {
      const result =
        action === "sync"
          ? await api.syncDataDomain(domain)
          : action === "export"
            ? await api.exportDataDomain(domain)
            : await api.cleanupDataDomain(domain);
      if (!result.success) throw new Error(result.message);
      return result;
    },
    onSuccess: (result, variables) => {
      void queryClient.invalidateQueries({ queryKey: ["database-domain-summary"] });
      onComplete?.(variables.action, result);
      if (!onComplete) {
        showToast(result.path ? `${result.message}：${result.path}` : result.message);
      }
    },
    onError: (err, variables) => {
      const message = err instanceof Error ? err.message : String(err);
      onComplete?.(variables.action, { success: false, domain, action: variables.action, message });
      if (!onComplete) {
        showToast(message);
      }
    },
  });

  const isPending = (action: "sync" | "export" | "cleanup") =>
    actionMutation.isPending && actionMutation.variables?.action === action;

  const handleCleanup = () => {
    const label = DOMAIN_LABELS[domain] ?? domain;
    if (confirm(`确定要清理数据域「${label}」吗？此操作将删除该域的本地数据，无法撤销。`)) {
      actionMutation.mutate({ action: "cleanup" });
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Button
        variant="outline"
        size={size}
        onClick={() => actionMutation.mutate({ action: "sync" })}
        disabled={actionMutation.isPending}
      >
        <RefreshCw
          className={cn("size-4", isPending("sync") && "animate-spin")}
          aria-hidden="true"
        />
        同步
      </Button>
      <Button
        variant="outline"
        size={size}
        onClick={() => actionMutation.mutate({ action: "export" })}
        disabled={actionMutation.isPending}
      >
        <Download className="size-4" aria-hidden="true" />
        导出
      </Button>
      <Button
        variant="outline"
        size={size}
        onClick={handleCleanup}
        disabled={actionMutation.isPending}
        className="text-destructive hover:bg-destructive/10 hover:text-destructive"
      >
        <Trash2 className="size-4" aria-hidden="true" />
        清理
      </Button>
    </div>
  );
}
