import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { ChartUserConfig, CrosshairModeSetting, PriceScaleModeSetting } from "./chart-config";
import type { IndicatorSettings } from "./indicators/types";

interface ChartSettingsPanelProps {
  config: ChartUserConfig;
  onChange: (patch: Partial<ChartUserConfig>) => void;
  onReset: () => void;
  onClose: () => void;
  indicatorSettings: IndicatorSettings;
  onIndicatorSettingsChange: (patch: Partial<IndicatorSettings>) => void;
}

function Field({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-[11px] text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}

function Check({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2 text-xs text-foreground">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="rounded border-border"
      />
      {label}
    </label>
  );
}

export function ChartSettingsPanel({
  config,
  onChange,
  onReset,
  onClose,
  indicatorSettings,
  onIndicatorSettingsChange,
}: ChartSettingsPanelProps) {
  return (
    <div className="border-b border-border bg-muted/20 px-3 py-3">
      <div className="mb-3 flex items-center justify-between">
        <span className="text-xs font-semibold text-foreground">图表配置</span>
        <div className="flex gap-2">
          <Button type="button" variant="ghost" size="sm" className="h-7 text-xs" onClick={onReset}>
            恢复默认
          </Button>
          <Button type="button" variant="ghost" size="sm" className="h-7 text-xs" onClick={onClose}>
            收起
          </Button>
        </div>
      </div>

      <div className="grid max-h-[220px] gap-3 overflow-y-auto sm:grid-cols-2 lg:grid-cols-4">
        <section className="space-y-2">
          <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            布局
          </p>
          <Check
            label="显示成交量 pane"
            checked={config.showVolume}
            onChange={(v) => onChange({ showVolume: v })}
          />
          <Field label="主图高度比">
            <Input
              type="number"
              min={1}
              max={8}
              step={0.5}
              value={config.mainPaneStretch}
              onChange={(e) => onChange({ mainPaneStretch: Number(e.target.value) || 3 })}
              className="h-7 font-mono text-xs"
            />
          </Field>
          <Field label="成交量高度比">
            <Input
              type="number"
              min={0.2}
              max={4}
              step={0.1}
              value={config.volumePaneStretch}
              onChange={(e) => onChange({ volumePaneStretch: Number(e.target.value) || 1 })}
              className="h-7 font-mono text-xs"
              disabled={!config.showVolume}
            />
          </Field>
        </section>

        <section className="space-y-2">
          <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            蜡烛
          </p>
          <Check
            label="显示影线"
            checked={config.wickVisible}
            onChange={(v) => onChange({ wickVisible: v })}
          />
          <Check
            label="最新价线"
            checked={config.priceLineVisible}
            onChange={(v) => onChange({ priceLineVisible: v })}
          />
        </section>

        <section className="space-y-2">
          <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            坐标轴
          </p>
          <Field label="K 线间距">
            <Input
              type="number"
              min={2}
              max={20}
              value={config.barSpacing}
              onChange={(e) => onChange({ barSpacing: Number(e.target.value) || 6 })}
              className="h-7 font-mono text-xs"
            />
          </Field>
          <Field label="右侧留白 (bars)">
            <Input
              type="number"
              min={0}
              max={50}
              value={config.rightOffset}
              onChange={(e) => onChange({ rightOffset: Number(e.target.value) || 12 })}
              className="h-7 font-mono text-xs"
            />
          </Field>
          <Field label="价格刻度">
            <select
              value={config.priceScaleMode}
              onChange={(e) =>
                onChange({ priceScaleMode: e.target.value as PriceScaleModeSetting })
              }
              className="h-7 w-full rounded-md border border-border bg-background px-2 text-xs"
            >
              <option value="normal">线性</option>
              <option value="logarithmic">对数</option>
            </select>
          </Field>
          <Check
            label="反转价格轴"
            checked={config.invertScale}
            onChange={(v) => onChange({ invertScale: v })}
          />
        </section>

        <section className="space-y-2">
          <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            交互
          </p>
          <Field label="十字光标">
            <select
              value={config.crosshairMode}
              onChange={(e) =>
                onChange({ crosshairMode: e.target.value as CrosshairModeSetting })
              }
              className="h-7 w-full rounded-md border border-border bg-background px-2 text-xs"
            >
              <option value="magnet">磁吸 (TradingView 默认)</option>
              <option value="normal">自由</option>
            </select>
          </Field>
          <Check
            label="垂直网格"
            checked={config.gridVertVisible}
            onChange={(v) => onChange({ gridVertVisible: v })}
          />
          <Check
            label="水平网格"
            checked={config.gridHorzVisible}
            onChange={(v) => onChange({ gridHorzVisible: v })}
          />
          <Check
            label="滚轮缩放 / 拖拽"
            checked={config.scrollEnabled && config.scaleEnabled}
            onChange={(v) => onChange({ scrollEnabled: v, scaleEnabled: v })}
          />
          <Check
            label="惯性滚动"
            checked={config.kineticScroll}
            onChange={(v) => onChange({ kineticScroll: v })}
          />
        </section>

        <section className="space-y-2">
          <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            指标参数
          </p>
          <Field label="BOLL 周期">
            <Input
              type="number"
              min={5}
              max={120}
              value={indicatorSettings.bollPeriod}
              onChange={(e) =>
                onIndicatorSettingsChange({ bollPeriod: Number(e.target.value) || 20 })
              }
              className="h-7 font-mono text-xs"
            />
          </Field>
          <Field label="BOLL 倍数">
            <Input
              type="number"
              min={0.5}
              max={5}
              step={0.1}
              value={indicatorSettings.bollMult}
              onChange={(e) =>
                onIndicatorSettingsChange({ bollMult: Number(e.target.value) || 2 })
              }
              className="h-7 font-mono text-xs"
            />
          </Field>
          <Field label="KDJ 周期">
            <Input
              type="number"
              min={3}
              max={30}
              value={indicatorSettings.kdjPeriod}
              onChange={(e) =>
                onIndicatorSettingsChange({ kdjPeriod: Number(e.target.value) || 9 })
              }
              className="h-7 font-mono text-xs"
            />
          </Field>
        </section>
      </div>
    </div>
  );
}
