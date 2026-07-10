import { useState, useEffect } from "react";
import { useMutation } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface StockOrderTicketProps {
  accountId: string;
  tsCode?: string;
  name?: string;
  onSuccess?: () => void;
}

export function StockOrderTicket({ accountId, tsCode: initialTsCode = "", name: initialName = "", onSuccess }: StockOrderTicketProps) {
  const [tsCode, setTsCode] = useState(initialTsCode);
  const [name, setName] = useState(initialName);
  const [side, setSide] = useState<"buy" | "sell">("buy");
  const [price, setPrice] = useState<string>("");
  const [quantity, setQuantity] = useState<string>("100");
  const [estimate, setEstimate] = useState<{
    estimated_amount: number;
    commission: number;
    stamp_tax: number;
    transfer_fee: number;
    total_cost: number;
  } | null>(null);

  const priceNum = parseFloat(price) || 0;
  const quantityNum = parseInt(quantity, 10) || 0;

  useEffect(() => {
    if (priceNum > 0 && quantityNum > 0) {
      api
        .estimateStockPaperOrder({ price: priceNum, quantity: quantityNum, side })
        .then(setEstimate)
        .catch(() => setEstimate(null));
    } else {
      setEstimate(null);
    }
  }, [priceNum, quantityNum, side]);

  const placeMutation = useMutation({
    mutationFn: () =>
      api.placeStockPaperOrder({
        account_id: accountId,
        ts_code: tsCode,
        side,
        order_type: "limit",
        price: priceNum,
        quantity: quantityNum,
      }),
    onSuccess: () => {
      onSuccess?.();
      setPrice("");
      setQuantity("100");
    },
  });

  return (
    <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
      <h3 className="mb-3 text-sm font-medium">模拟下单</h3>
      <div className="mb-3 flex gap-2">
        <button
          type="button"
          onClick={() => setSide("buy")}
          className={cn(
            "flex-1 rounded-md py-1 text-xs font-medium transition-colors",
            side === "buy"
              ? "bg-emerald-500/20 text-emerald-500"
              : "bg-muted text-muted-foreground hover:bg-accent"
          )}
        >
          买入
        </button>
        <button
          type="button"
          onClick={() => setSide("sell")}
          className={cn(
            "flex-1 rounded-md py-1 text-xs font-medium transition-colors",
            side === "sell"
              ? "bg-rose-500/20 text-rose-500"
              : "bg-muted text-muted-foreground hover:bg-accent"
          )}
        >
          卖出
        </button>
      </div>

      <div className="space-y-3">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-xs text-muted-foreground">代码</label>
            <Input
              value={tsCode}
              onChange={(e) => setTsCode(e.target.value)}
              placeholder="600000.SH"
              className="mt-1 h-8 text-xs"
            />
          </div>
          <div>
            <label className="text-xs text-muted-foreground">名称</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="浦发银行"
              className="mt-1 h-8 text-xs"
            />
          </div>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="text-xs text-muted-foreground">价格</label>
            <Input
              type="number"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              placeholder="限价"
              className="mt-1 h-8 text-xs"
            />
          </div>
          <div>
            <label className="text-xs text-muted-foreground">数量</label>
            <Input
              type="number"
              value={quantity}
              onChange={(e) => setQuantity(e.target.value)}
              step={100}
              className="mt-1 h-8 text-xs"
            />
          </div>
        </div>
      </div>

      {estimate && (
        <div className="mt-3 space-y-1 rounded-md bg-muted/40 p-2 text-[11px] text-muted-foreground">
          <div className="flex justify-between">
            <span>成交金额</span>
            <span className="tabular-nums">{estimate.estimated_amount.toFixed(2)}</span>
          </div>
          <div className="flex justify-between">
            <span>佣金</span>
            <span className="tabular-nums">{estimate.commission.toFixed(2)}</span>
          </div>
          <div className="flex justify-between">
            <span>印花税</span>
            <span className="tabular-nums">{estimate.stamp_tax.toFixed(2)}</span>
          </div>
          <div className="flex justify-between">
            <span>过户费</span>
            <span className="tabular-nums">{estimate.transfer_fee.toFixed(2)}</span>
          </div>
          <div className="flex justify-between font-medium text-foreground">
            <span>{side === "buy" ? "总成本" : "净收入"}</span>
            <span className="tabular-nums">{estimate.total_cost.toFixed(2)}</span>
          </div>
        </div>
      )}

      <div className="mt-3 text-[10px] text-amber-500">模拟交易 · T+1 · 100 股整数倍</div>

      <Button
        size="sm"
        className="mt-3 w-full"
        onClick={() => placeMutation.mutate()}
        disabled={placeMutation.isPending || !tsCode || priceNum <= 0 || quantityNum <= 0}
      >
        {placeMutation.isPending ? "提交中…" : side === "buy" ? "模拟买入" : "模拟卖出"}
      </Button>
    </div>
  );
}
