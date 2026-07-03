import type { FuturesProduct, FuturesSector, KLine, RealtimeQuote } from "@/types";
import { FUTURES_SECTOR_ORDER } from "@/data/futures";

export interface ProductHeat {
  product: FuturesProduct;
  changePct: number | null;
  source: "realtime" | "kline" | "none";
}

export interface SectorHeat {
  sector: FuturesSector;
  avgChangePct: number | null;
  products: ProductHeat[];
}

export function parseChangeFromSummary(summary: string): number | null {
  const m = summary.match(/change%[=:\s]*([-+]?\d+(?:\.\d+)?)/i);
  if (!m) return null;
  const v = parseFloat(m[1]);
  return Number.isFinite(v) ? v : null;
}

export function changeFromKlines(klines: KLine[]): number | null {
  if (klines.length < 2) return null;
  const last = klines[klines.length - 1];
  const prev = klines[klines.length - 2];
  if (!prev.close) return null;
  return ((last.close - prev.close) / prev.close) * 100;
}

export function changeFromQuote(quote: RealtimeQuote | undefined): number | null {
  if (!quote || !Number.isFinite(quote.change_pct)) return null;
  return quote.change_pct;
}

export function buildSectorHeat(
  sectors: FuturesSector[],
  klineBySymbol: Map<string, KLine[]>,
  quoteBySymbol: Map<string, RealtimeQuote> = new Map()
): SectorHeat[] {
  const byCode = new Map(
    sectors.map((sector) => {
      const products: ProductHeat[] = sector.products.map((product) => {
        const sym = product.symbol.toLowerCase();
        const fromQuote = changeFromQuote(quoteBySymbol.get(sym));
        if (fromQuote !== null) {
          return { product, changePct: fromQuote, source: "realtime" as const };
        }
        const klines = klineBySymbol.get(sym);
        if (klines?.length) {
          return { product, changePct: changeFromKlines(klines), source: "kline" as const };
        }
        return { product, changePct: null, source: "none" as const };
      });

      const valid = products.map((p) => p.changePct).filter((v): v is number => v !== null);
      const avgChangePct =
        valid.length > 0 ? valid.reduce((a, b) => a + b, 0) / valid.length : null;

      return [sector.code, { sector, avgChangePct, products }] as const;
    })
  );

  return FUTURES_SECTOR_ORDER.map((code) => byCode.get(code)).filter(
    (s): s is SectorHeat => !!s
  );
}

export function heatColor(changePct: number | null): string {
  if (changePct === null) return "bg-muted/40 text-muted-foreground";
  if (changePct >= 2) return "bg-up/25 text-up border-up/30";
  if (changePct >= 0.5) return "bg-up/15 text-up border-up/20";
  if (changePct <= -2) return "bg-down/25 text-down border-down/30";
  if (changePct <= -0.5) return "bg-down/15 text-down border-down/20";
  return "bg-muted/30 text-foreground border-border";
}

export function formatPct(v: number | null): string {
  if (v === null) return "—";
  const sign = v > 0 ? "+" : "";
  return `${sign}${v.toFixed(2)}%`;
}
