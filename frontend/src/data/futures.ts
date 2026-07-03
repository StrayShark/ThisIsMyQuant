/** 期货品种目录：从 Rust list_products 加载，E2E 用静态回退。 */
import type { FuturesProduct, FuturesSector, LiquidityTier } from "@/types";
import { api } from "@/api/client";

/** E2E / 离线静态目录（五大板块、27 个 v1 core） */
/** 与 Rust sectors.rs / 品种页一致的板块顺序 */
export const FUTURES_SECTOR_ORDER = [
  "black",
  "metals",
  "agriculture",
  "energy_chemical",
  "shipping",
] as const;

export const STATIC_FUTURES_CATALOG: FuturesSector[] = [
  {
    code: "black",
    name: "黑色建材",
    description: "钢材、炉料、煤焦和建材链。",
    products: [
      p("rb", "RB0", "螺纹钢", "SHFE"),
      p("hc", "HC0", "热卷", "SHFE"),
      p("i", "I0", "铁矿石", "DCE"),
      p("j", "J0", "焦炭", "DCE"),
      p("jm", "JM0", "焦煤", "DCE"),
    ],
  },
  {
    code: "metals",
    name: "有色贵金属",
    description: "基本金属、贵金属和新能源材料。",
    products: [
      p("cu", "CU0", "沪铜", "SHFE"),
      p("al", "AL0", "沪铝", "SHFE"),
      p("zn", "ZN0", "沪锌", "SHFE"),
      p("ni", "NI0", "沪镍", "SHFE"),
      p("lc", "LC0", "碳酸锂", "GFEX"),
      p("au", "AU0", "黄金", "SHFE"),
      p("ag", "AG0", "白银", "SHFE"),
    ],
  },
  {
    code: "agriculture",
    name: "农产品软商品",
    description: "油脂油料、谷物、棉糖和养殖链。",
    products: [
      p("m", "M0", "豆粕", "DCE"),
      p("y", "Y0", "豆油", "DCE"),
      p("p", "P0", "棕榈油", "DCE"),
      p("c", "C0", "玉米", "DCE"),
      p("sr", "SR0", "白糖", "CZCE"),
      p("cf", "CF0", "棉花", "CZCE"),
      p("lh", "LH0", "生猪", "DCE"),
    ],
  },
  {
    code: "energy_chemical",
    name: "能源化工",
    description: "油气、煤化工、聚酯、塑化和橡胶链。",
    products: [
      p("sc", "SC0", "原油", "INE"),
      p("fu", "FU0", "燃料油", "SHFE"),
      p("bu", "BU0", "沥青", "SHFE"),
      p("ta", "TA0", "PTA", "CZCE"),
      p("ma", "MA0", "甲醇", "CZCE"),
      p("pp", "PP0", "聚丙烯", "DCE"),
      p("ru", "RU0", "橡胶", "SHFE"),
    ],
  },
  {
    code: "shipping",
    name: "航运运价",
    description: "集运指数及航运链。",
    products: [p("ec", "EC0", "集运欧线", "INE")],
  },
];

function p(
  code: string,
  symbol: string,
  name: string,
  exchange: FuturesProduct["exchange"]
): FuturesProduct {
  return {
    code,
    symbol,
    name,
    exchange,
    liquidity_tier: "core",
  };
}

let catalog: FuturesSector[] = STATIC_FUTURES_CATALOG;
let allProductsCatalog: FuturesSector[] = STATIC_FUTURES_CATALOG;

export function setFuturesCatalog(sectors: FuturesSector[], allSectors?: FuturesSector[]) {
  catalog = sectors;
  allProductsCatalog = allSectors ?? sectors;
}

export function getFuturesCatalog(tier: "core" | "all" = "core"): FuturesSector[] {
  return tier === "all" ? allProductsCatalog : catalog;
}

/** @deprecated 使用 getFuturesCatalog() */
export const FUTURES_SECTORS = STATIC_FUTURES_CATALOG;

export const FUTURES_PRODUCTS: FuturesProduct[] = STATIC_FUTURES_CATALOG.flatMap(
  (s) => s.products
);

export function getFuturesProduct(symbol: string): FuturesProduct | undefined {
  const key = normalizeProduct(symbol);
  const pool = allProductsCatalog.flatMap((s) => s.products);
  return pool.find(
    (item) =>
      item.symbol.toLowerCase() === symbol.toLowerCase() ||
      item.code === key ||
      item.symbol.toLowerCase() === `${key}0`.toLowerCase()
  );
}

export function findFuturesSymbol(query: string): string | undefined {
  const q = query.trim();
  if (!q) return undefined;
  const pool = allProductsCatalog.flatMap((s) => s.products);
  const byName = pool.find((p) => p.name.includes(q));
  if (byName) return byName.symbol;
  const upper = q.toUpperCase();
  const bySym = pool.find(
    (p) => p.symbol.toUpperCase() === upper || p.code.toUpperCase() === upper
  );
  return bySym?.symbol;
}

export function normalizeProduct(symbol: string): string {
  const m = symbol.trim().match(/^([a-zA-Z]+)/);
  return (m?.[1] ?? symbol).toLowerCase();
}

export async function loadFuturesCatalog(options?: {
  tier?: LiquidityTier | "all";
  includeWatch?: boolean;
}): Promise<FuturesSector[]> {
  const tier = options?.tier ?? "core";
  try {
    const sectors = await api.listProducts({ tier });
    if (options?.includeWatch && tier === "core") {
      const watch = await api.listProducts({ tier: "watch" });
      const merged = mergeSectors(sectors, watch);
      setFuturesCatalog(sectors, mergeSectors(sectors, watch));
      return merged;
    }
    if (tier === "all") {
      setFuturesCatalog(sectors, sectors);
    } else {
      const all = await api.listProducts({ tier: "all" });
      setFuturesCatalog(sectors, all);
    }
    return getFuturesCatalog(tier === "all" ? "all" : "core");
  } catch {
    setFuturesCatalog(STATIC_FUTURES_CATALOG);
    return STATIC_FUTURES_CATALOG;
  }
}

function mergeSectors(base: FuturesSector[], extra: FuturesSector[]): FuturesSector[] {
  const map = new Map<string, FuturesSector>();
  for (const s of base) {
    map.set(s.code, { ...s, products: [...s.products] });
  }
  for (const s of extra) {
    const existing = map.get(s.code);
    if (existing) {
      const seen = new Set(existing.products.map((p) => p.symbol));
      for (const p of s.products) {
        if (!seen.has(p.symbol)) existing.products.push(p);
      }
    } else {
      map.set(s.code, s);
    }
  }
  return Array.from(map.values());
}
