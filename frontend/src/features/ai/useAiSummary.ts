import { useCallback, useState } from "react";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import type { AiReportSummary, AiSummaryRequest } from "@/types";

const DEFAULT_DISCLAIMER = "仅供研究与复盘，不构成投资建议";

export function useAiSummary() {
  const showToast = useAppStore((s) => s.showToast);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [report, setReport] = useState<AiReportSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  const generate = useCallback(
    async (payload: AiSummaryRequest) => {
      setIsOpen(true);
      setIsLoading(true);
      setError(null);
      setReport(null);
      try {
        const data = await api.generateAiSummary(payload);
        setReport(data);
      } catch (err) {
        const message = err instanceof Error ? err.message : "AI 摘要生成失败";
        setError(message);
        showToast(message);
      } finally {
        setIsLoading(false);
      }
    },
    [showToast]
  );

  const close = useCallback(() => {
    setIsOpen(false);
  }, []);

  const reset = useCallback(() => {
    setReport(null);
    setError(null);
    setIsLoading(false);
  }, []);

  return {
    isOpen,
    isLoading,
    report,
    error,
    generate,
    close,
    reset,
    disclaimer: report?.disclaimer ?? DEFAULT_DISCLAIMER,
  };
}
