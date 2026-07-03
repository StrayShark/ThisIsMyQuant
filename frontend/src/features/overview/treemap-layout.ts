import squarify from "squarify";
import { FUTURES_SECTOR_ORDER } from "@/data/futures";
import type { SectorHeat } from "./sector-heat";
import { productWeight } from "./treemap-colors";

export interface TreemapTile {
  id: string;
  label: string;
  symbol?: string;
  changePct: number | null;
  value: number;
  sectorCode?: string;
}

/** 坐标相对于板块内品种区域左上角 */
export interface TreemapRect extends TreemapTile {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export interface TreemapSectorGroup {
  id: string;
  label: string;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  labelHeight: number;
  products: TreemapRect[];
}

type SquarifyInput = TreemapTile;

const SECTOR_GAP = 1;
const PRODUCT_GAP = 1;
const LABEL_H = 26;

/** 单格至少能容纳一行 8px 涨跌幅 + 一行 8px 名称 */
export const MIN_TILE_W = 36;
export const MIN_TILE_H = 28;

/** 权重最大/最小比，避免 squarify 产生极窄条 */
const MAX_WEIGHT_RATIO = 3.5;

function orderSectors(sectorHeat: SectorHeat[]): SectorHeat[] {
  const byCode = new Map(sectorHeat.map((s) => [s.sector.code, s]));
  return FUTURES_SECTOR_ORDER.map((code) => byCode.get(code)).filter(
    (s): s is SectorHeat => !!s
  );
}

function layoutSectorGrid(
  sectorHeat: SectorHeat[],
  width: number,
  height: number
): Array<{ sector: SectorHeat; x0: number; y0: number; x1: number; y1: number }> {
  const ordered = orderSectors(sectorHeat);
  const n = ordered.length;
  if (n === 0) return [];

  const half = SECTOR_GAP / 2;

  // 5 个板块：上 3 下 2，底行每格更宽，避免右下空位浪费
  if (n === 5) {
    const rowH = height / 2;
    const topColW = width / 3;
    const bottomColW = width / 2;
    return ordered.map((sector, i) => {
      if (i < 3) {
        return {
          sector,
          x0: i * topColW + half,
          y0: half,
          x1: (i + 1) * topColW - half,
          y1: rowH - half,
        };
      }
      const bottomIndex = i - 3;
      return {
        sector,
        x0: bottomIndex * bottomColW + half,
        y0: rowH + half,
        x1: (bottomIndex + 1) * bottomColW - half,
        y1: height - half,
      };
    });
  }

  const cols = 3;
  const rows = Math.ceil(n / cols);
  const cellW = width / cols;
  const cellH = height / rows;

  return ordered.map((sector, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    return {
      sector,
      x0: col * cellW + half,
      y0: row * cellH + half,
      x1: (col + 1) * cellW - half,
      y1: (row + 1) * cellH - half,
    };
  });
}

function productTiles(
  sector: SectorHeat,
  klineBySymbol: Map<string, import("@/types").KLine[]>
): TreemapTile[] {
  return sector.products.map(({ product, changePct }) => {
    const klines = klineBySymbol.get(product.symbol.toLowerCase());
    return {
      id: product.symbol,
      label: product.name,
      symbol: product.symbol,
      changePct,
      value: productWeight(klines, 1),
      sectorCode: sector.sector.code,
    };
  });
}

/** 压缩成交额跨度：开方 + 下限，防止一块占满、其余成细条 */
export function prepareTileWeights(tiles: TreemapTile[]): TreemapTile[] {
  if (tiles.length <= 1) return tiles;
  const compressed = tiles.map((t) => ({
    ...t,
    value: Math.sqrt(Math.max(t.value, 1)),
  }));
  const max = Math.max(...compressed.map((t) => t.value));
  const minAllowed = max / MAX_WEIGHT_RATIO;
  return compressed.map((t) => ({
    ...t,
    value: Math.max(t.value, minAllowed),
  }));
}

function withGap(rect: TreemapRect, gap: number): TreemapRect {
  return {
    ...rect,
    x0: rect.x0 + gap,
    y0: rect.y0 + gap,
    x1: rect.x1 - gap,
    y1: rect.y1 - gap,
  };
}

function rectSize(rect: Pick<TreemapRect, "x0" | "y0" | "x1" | "y1">) {
  return { w: rect.x1 - rect.x0, h: rect.y1 - rect.y0 };
}

export function isReadableTile(rect: Pick<TreemapRect, "x0" | "y0" | "x1" | "y1">): boolean {
  const { w, h } = rectSize(rect);
  return w >= MIN_TILE_W && h >= MIN_TILE_H;
}

function heroWidthRatio(heroValue: number, totalValue: number, count: number): number {
  const share = heroValue / totalValue;
  const minRatio = count <= 5 ? 0.34 : 0.3;
  const maxRatio = count <= 5 ? 0.46 : 0.4;
  return Math.min(maxRatio, Math.max(minRatio, share * 1.15));
}

/** 左侧主品种 + 右侧等分网格，保证最小格尺寸一致 */
function layoutHeroGrid(
  tiles: TreemapTile[],
  innerW: number,
  innerH: number,
  gap: number
): TreemapRect[] {
  const sorted = [...tiles].sort((a, b) => b.value - a.value);
  const total = sorted.reduce((sum, t) => sum + t.value, 0);
  const hero = sorted[0];
  const rest = sorted.slice(1);
  const heroW = innerW * heroWidthRatio(hero.value, total, sorted.length);
  const sideW = innerW - heroW;
  const cols = rest.length <= 2 ? 1 : 2;
  const rows = Math.ceil(rest.length / cols);
  const cellW = sideW / cols;
  const cellH = innerH / rows;

  const rects: TreemapRect[] = [
    {
      ...hero,
      x0: 0,
      y0: 0,
      x1: heroW,
      y1: innerH,
    },
  ];

  rest.forEach((tile, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    rects.push({
      ...tile,
      x0: heroW + col * cellW,
      y0: row * cellH,
      x1: heroW + (col + 1) * cellW,
      y1: (row + 1) * cellH,
    });
  });

  return rects.map((r) => withGap(r, gap));
}

function layoutSquarify(
  tiles: TreemapTile[],
  innerW: number,
  innerH: number,
  gap: number
): TreemapRect[] {
  const prepared = prepareTileWeights(tiles);
  return squarify(prepared as SquarifyInput[], {
    x0: 0,
    y0: 0,
    x1: innerW,
    y1: innerH,
  }).map((rect) => withGap(rect as TreemapRect, gap));
}

function layoutEqualGrid(
  tiles: TreemapTile[],
  innerW: number,
  innerH: number,
  gap: number
): TreemapRect[] {
  const n = tiles.length;
  const cols = n <= 4 ? 2 : 3;
  const rows = Math.ceil(n / cols);
  const cellW = innerW / cols;
  const cellH = innerH / rows;

  return tiles.map((tile, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    return withGap(
      {
        ...tile,
        x0: col * cellW,
        y0: row * cellH,
        x1: (col + 1) * cellW,
        y1: (row + 1) * cellH,
      },
      gap
    );
  });
}

/** 按板块面积与品种数选择布局，确保最小格可读 */
export function layoutSectorProducts(
  tiles: TreemapTile[],
  innerW: number,
  innerH: number
): TreemapRect[] {
  if (tiles.length === 0 || innerW <= 16 || innerH <= 16) return [];

  const gap = PRODUCT_GAP / 2;
  const sorted = [...tiles].sort((a, b) => b.value - a.value);

  let rects: TreemapRect[];
  if (sorted.length === 1) {
    rects = [withGap({ ...sorted[0], x0: 0, y0: 0, x1: innerW, y1: innerH }, gap)];
  } else if (sorted.length <= 3) {
    rects = layoutSquarify(sorted, innerW, innerH, gap);
  } else {
    rects = layoutHeroGrid(sorted, innerW, innerH, gap);
  }

  if (rects.every(isReadableTile)) return rects;

  // 兜底：等分网格，优先保证文字可见
  const equal = sorted.map((t) => ({ ...t, value: 1 }));
  if (sorted.length <= 3) {
    rects = layoutSquarify(equal, innerW, innerH, gap);
  } else if (sorted.length <= 6) {
    rects = layoutEqualGrid(equal, innerW, innerH, gap);
  } else {
    rects = layoutHeroGrid(equal, innerW, innerH, gap);
  }

  if (rects.every(isReadableTile)) return rects;

  return layoutEqualGrid(equal, innerW, innerH, gap);
}

/** 五大商品板块网格 + 板块内布局。 */
export function layoutNestedTreemap(
  sectorHeat: SectorHeat[],
  width: number,
  height: number,
  klineBySymbol: Map<string, import("@/types").KLine[]>
): TreemapSectorGroup[] {
  if (width <= 0 || height <= 0 || sectorHeat.length === 0) return [];

  const sectorGrid = layoutSectorGrid(sectorHeat, width, height);

  return sectorGrid.map(({ sector, x0, y0, x1, y1 }) => {
    const sectorW = x1 - x0;
    const sectorH = y1 - y0;
    const labelHeight = sectorW > 36 && sectorH > 32 ? LABEL_H : 0;
    const innerW = sectorW;
    const innerH = sectorH - labelHeight;

    const tiles = productTiles(sector, klineBySymbol).filter((t) => t.value > 0);
    const products =
      tiles.length > 0 ? layoutSectorProducts(tiles, innerW, innerH) : [];

    return {
      id: sector.sector.code,
      label: sector.sector.name,
      x0,
      y0,
      x1,
      y1,
      labelHeight,
      products,
    };
  });
}
