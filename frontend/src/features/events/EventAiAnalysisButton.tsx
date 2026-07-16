import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Sparkles, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { api } from "@/api/client";
import type { MarketEvent } from "@/types";

interface EventAiAnalysisButtonProps {
  event: MarketEvent;
  className?: string;
}

export function EventAiAnalysisButton({ event, className }: EventAiAnalysisButtonProps) {
  const [resultVisible, setResultVisible] = useState(false);

  const mutation = useMutation({
    mutationFn: () =>
      api.generateAiSummary({
        task_type: "event_impact",
        target_symbol: event.affected_symbols[0] ?? null,
        prompt: event.title,
      }),
    onSuccess: () => setResultVisible(true),
  });

  return (
    <div className={className}>
      <Button
        type="button"
        variant="secondary"
        size="sm"
        className="gap-1.5"
        disabled={mutation.isPending}
        onClick={() => {
          setResultVisible(false);
          mutation.mutate();
        }}
      >
        {mutation.isPending ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <Sparkles className="h-4 w-4" />
        )}
        生成 AI 影响分析
      </Button>

      {resultVisible && mutation.data && (
        <Card className="mt-3 border-border/60 bg-muted/30">
          <CardHeader className="p-3 pb-0">
            <CardTitle className="text-xs font-medium text-foreground">
              AI 影响分析（{mutation.data.provider}）
            </CardTitle>
          </CardHeader>
          <CardContent className="p-3 pt-2">
            <p className="whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground">
              {mutation.data.content}
            </p>
            <p className="mt-2 text-[10px] text-muted-foreground/70">
              {mutation.data.disclaimer}
            </p>
          </CardContent>
        </Card>
      )}

      {mutation.error && (
        <p className="mt-2 text-xs text-destructive">{mutation.error.message}</p>
      )}
    </div>
  );
}
