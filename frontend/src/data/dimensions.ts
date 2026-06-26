/** 分析维度标签（与 Rust dimensions.rs 对齐）。 */
export const DIMENSION_LABELS: Record<string, string> = {
  seasonality: "季节性",
  weather: "天气",
  overseas_upstream: "海外上游",
  domestic_supply: "国内供给",
  demand: "需求",
  inventory: "库存",
  spread_arb: "价差套利",
  policy: "政策监管",
  macro: "国内宏观",
  overseas_finance: "国外金融环境",
  geopolitics: "地缘",
  earnings: "企业财报",
  flow: "资金持仓",
  technical: "技术面",
};

export function dimensionLabel(code: string): string {
  return DIMENSION_LABELS[code] ?? code;
}
