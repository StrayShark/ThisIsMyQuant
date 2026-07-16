import { useState } from "react";
import { Check, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type { WatchlistItem } from "@/types";

interface WatchlistNoteEditorProps {
  item: WatchlistItem;
  onSave: (payload: {
    notes: string;
    alert_price: number | null;
    alert_pct: number | null;
  }) => void;
  onCancel: () => void;
  className?: string;
}

export function WatchlistNoteEditor({
  item,
  onSave,
  onCancel,
  className,
}: WatchlistNoteEditorProps) {
  const [notes, setNotes] = useState(item.notes ?? "");
  const [alertPrice, setAlertPrice] = useState(
    item.alert_price === null || item.alert_price === undefined ? "" : String(item.alert_price)
  );
  const [alertPct, setAlertPct] = useState(
    item.alert_pct === null || item.alert_pct === undefined ? "" : String(item.alert_pct)
  );

  const handleSave = () => {
    onSave({
      notes: notes.trim(),
      alert_price: alertPrice.trim() === "" ? null : Number(alertPrice),
      alert_pct: alertPct.trim() === "" ? null : Number(alertPct),
    });
  };

  return (
    <div
      className={cn(
        "rounded-lg border border-border bg-card p-3 shadow-sm",
        className
      )}
      onClick={(e) => e.stopPropagation()}
    >
      <Textarea
        value={notes}
        onChange={(e) => setNotes(e.target.value)}
        placeholder="添加备注…"
        className="mb-2 min-h-[60px] resize-none rounded-md text-xs"
      />
      <div className="flex items-center gap-2">
        <div className="flex flex-1 items-center gap-2">
          <span className="text-xs text-muted-foreground">价格</span>
          <Input
            type="number"
            value={alertPrice}
            onChange={(e) => setAlertPrice(e.target.value)}
            placeholder="触发价"
            className="h-7 w-24 rounded-md text-xs"
          />
          <span className="text-xs text-muted-foreground">%</span>
          <Input
            type="number"
            value={alertPct}
            onChange={(e) => setAlertPct(e.target.value)}
            placeholder="涨跌幅"
            className="h-7 w-24 rounded-md text-xs"
          />
        </div>
        <div className="flex items-center gap-1">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={onCancel}
            aria-label="取消"
          >
            <X className="h-3.5 w-3.5" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-green-600 hover:text-green-600"
            onClick={handleSave}
            aria-label="保存"
          >
            <Check className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>
    </div>
  );
}
