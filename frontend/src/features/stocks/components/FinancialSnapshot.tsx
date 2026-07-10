import type { StockFinancialMetric, StockValuationSnapshot } from "@/types";

interface FinancialSnapshotProps {
  financial?: StockFinancialMetric | null;
  valuation?: StockValuationSnapshot | null;
}

function formatWanYi(value?: number | null): string {
  if (value === undefined || value === null) return "--";
  if (Math.abs(value) >= 1e12) return `${(value / 1e12).toFixed(2)}万亿`;
  if (Math.abs(value) >= 1e8) return `${(value / 1e8).toFixed(2)}亿`;
  if (Math.abs(value) >= 1e4) return `${(value / 1e4).toFixed(2)}万`;
  return value.toFixed(2);
}

export function FinancialSnapshot({ financial, valuation }: FinancialSnapshotProps) {
  const items = [
    { label: "营业收入", value: formatWanYi(financial?.revenue), sub: `同比 ${financial?.revenue_yoy?.toFixed(2) ?? "--"}%` },
    { label: "净利润", value: formatWanYi(financial?.net_profit), sub: `同比 ${financial?.net_profit_yoy?.toFixed(2) ?? "--"}%` },
    { label: "ROE", value: financial?.roe ? `${financial.roe.toFixed(2)}%` : "--", sub: "净资产收益率" },
    { label: "资产负债率", value: financial?.debt_ratio ? `${financial.debt_ratio.toFixed(2)}%` : "--", sub: "总负债/总资产" },
    { label: "PE(TTM)", value: valuation?.pe_ttm?.toFixed(2) ?? "--", sub: "市盈率" },
    { label: "PB", value: valuation?.pb?.toFixed(2) ?? "--", sub: "市净率" },
    { label: "总市值", value: formatWanYi(valuation?.market_cap), sub: "元" },
    { label: "股息率", value: valuation?.dividend_yield ? `${valuation.dividend_yield.toFixed(2)}%` : "--", sub: "TTM" },
  ];

  return (
    <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-medium">财务摘要</h3>
        {financial?.report_period && (
          <span className="text-xs text-muted-foreground">报告期 {financial.report_period}</span>
        )}
      </div>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        {items.map((item) => (
          <div key={item.label} className="rounded-md bg-muted/40 p-2.5">
            <div className="text-xs text-muted-foreground">{item.label}</div>
            <div className="mt-0.5 text-sm font-semibold tabular-nums">{item.value}</div>
            <div className="text-[10px] text-muted-foreground">{item.sub}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
