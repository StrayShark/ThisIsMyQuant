import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export interface ScreenerCriteria {
  min_pe_ttm?: number;
  max_pe_ttm?: number;
  min_pb?: number;
  max_pb?: number;
  min_roe?: number;
  min_market_cap?: number;
  max_market_cap?: number;
}

interface StockScreenerBuilderProps {
  onRun: (criteria: ScreenerCriteria, name: string) => void;
  onSave: (criteria: ScreenerCriteria, name: string) => void;
  isRunning?: boolean;
}

export function StockScreenerBuilder({ onRun, onSave, isRunning }: StockScreenerBuilderProps) {
  const [name, setName] = useState("");
  const [criteria, setCriteria] = useState<ScreenerCriteria>({
    min_pe_ttm: undefined,
    max_pe_ttm: undefined,
    min_pb: undefined,
    max_pb: undefined,
    min_roe: undefined,
    min_market_cap: undefined,
    max_market_cap: undefined,
  });

  const update = (key: keyof ScreenerCriteria, value: string) => {
    const num = value === "" ? undefined : parseFloat(value);
    setCriteria((prev) => ({ ...prev, [key]: num }));
  };

  return (
    <div className="flex flex-col gap-4">
      <div>
        <label htmlFor="screen-name" className="text-xs">
          模板名称
        </label>
        <Input
          id="screen-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="未命名筛选"
          className="mt-1 h-8 text-xs"
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="text-xs">PE(TTM) 最小</label>
          <Input
            type="number"
            value={criteria.min_pe_ttm ?? ""}
            onChange={(e) => update("min_pe_ttm", e.target.value)}
            className="mt-1 h-8 text-xs"
          />
        </div>
        <div>
          <label className="text-xs">PE(TTM) 最大</label>
          <Input
            type="number"
            value={criteria.max_pe_ttm ?? ""}
            onChange={(e) => update("max_pe_ttm", e.target.value)}
            className="mt-1 h-8 text-xs"
          />
        </div>
        <div>
          <label className="text-xs">PB 最小</label>
          <Input
            type="number"
            value={criteria.min_pb ?? ""}
            onChange={(e) => update("min_pb", e.target.value)}
            className="mt-1 h-8 text-xs"
          />
        </div>
        <div>
          <label className="text-xs">PB 最大</label>
          <Input
            type="number"
            value={criteria.max_pb ?? ""}
            onChange={(e) => update("max_pb", e.target.value)}
            className="mt-1 h-8 text-xs"
          />
        </div>
        <div>
          <label className="text-xs">ROE 最小 (%)</label>
          <Input
            type="number"
            value={criteria.min_roe ?? ""}
            onChange={(e) => update("min_roe", e.target.value)}
            className="mt-1 h-8 text-xs"
          />
        </div>
        <div>
          <label className="text-xs">市值 最小（亿）</label>
          <Input
            type="number"
            value={criteria.min_market_cap ? criteria.min_market_cap / 1e8 : ""}
            onChange={(e) =>
              update("min_market_cap", e.target.value === "" ? "" : (parseFloat(e.target.value) * 1e8).toString())
            }
            className="mt-1 h-8 text-xs"
          />
        </div>
        <div>
          <label className="text-xs">市值 最大（亿）</label>
          <Input
            type="number"
            value={criteria.max_market_cap ? criteria.max_market_cap / 1e8 : ""}
            onChange={(e) =>
              update("max_market_cap", e.target.value === "" ? "" : (parseFloat(e.target.value) * 1e8).toString())
            }
            className="mt-1 h-8 text-xs"
          />
        </div>
      </div>

      <div className="flex gap-2">
        <Button size="sm" onClick={() => onRun(criteria, name)} disabled={isRunning} className="flex-1">
          {isRunning ? "运行中…" : "运行筛选"}
        </Button>
        <Button size="sm" variant="outline" onClick={() => onSave(criteria, name)} disabled={isRunning}>
          保存模板
        </Button>
      </div>
    </div>
  );
}
