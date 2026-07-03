import type { CalendarEvent, FuturesSector } from "@/types";

/** 宏观事件 → 可能影响的商品板块（用于总览联动）。 */
const EVENT_SECTOR_RULES: { pattern: RegExp; sectors: string[] }[] = [
  { pattern: /CPI|PPI|PCE|通胀|物价/, sectors: ["metals", "energy_chemical", "agriculture"] },
  { pattern: /非农|就业|失业率/, sectors: ["metals", "energy_chemical", "black"] },
  { pattern: /PMI|制造业/, sectors: ["black", "metals", "energy_chemical"] },
  { pattern: /美联储|FOMC|利率|降息|加息/, sectors: ["metals", "energy_chemical", "shipping"] },
  { pattern: /LPR|社融|M2|GDP/, sectors: ["black", "metals", "agriculture"] },
  { pattern: /原油|OPEC|库存报告/, sectors: ["energy_chemical"] },
  { pattern: /生猪|猪/, sectors: ["agriculture"] },
];

export interface CalendarWithProducts {
  event: CalendarEvent;
  sectorCodes: string[];
  productSymbols: string[];
}

export function mapCalendarToProducts(
  event: CalendarEvent,
  sectors: FuturesSector[]
): CalendarWithProducts {
  const sectorCodes = new Set<string>();

  if (event.country.startsWith("中国")) {
    sectorCodes.add("black");
    sectorCodes.add("metals");
  } else {
    sectorCodes.add("metals");
    sectorCodes.add("energy_chemical");
    sectorCodes.add("shipping");
  }

  for (const rule of EVENT_SECTOR_RULES) {
    if (rule.pattern.test(event.name)) {
      for (const s of rule.sectors) sectorCodes.add(s);
    }
  }

  const codes = [...sectorCodes];
  const productSymbols = sectors
    .filter((s) => codes.includes(s.code))
    .flatMap((s) => s.products.slice(0, 2).map((p) => p.symbol));

  return {
    event,
    sectorCodes: codes,
    productSymbols: [...new Set(productSymbols)].slice(0, 6),
  };
}
