import { describe, expect, it } from "vitest";
import {
  isReadableTile,
  layoutSectorProducts,
  prepareTileWeights,
  type TreemapTile,
} from "./treemap-layout";

function tile(id: string, value: number): TreemapTile {
  return {
    id,
    label: id,
    symbol: id,
    changePct: 1,
    value,
  };
}

describe("prepareTileWeights", () => {
  it("limits max/min weight ratio", () => {
    const prepared = prepareTileWeights([
      tile("a", 1_000_000),
      tile("b", 10_000),
      tile("c", 1_000),
    ]);
    const values = prepared.map((t) => t.value);
    expect(Math.max(...values) / Math.min(...values)).toBeLessThanOrEqual(3.5);
  });
});

describe("layoutSectorProducts", () => {
  it("keeps all tiles readable in a typical sector cell", () => {
    const tiles = [
      tile("lc", 900),
      tile("cu", 600),
      tile("al", 400),
      tile("zn", 300),
      tile("ni", 200),
      tile("au", 150),
      tile("ag", 100),
    ];
    const rects = layoutSectorProducts(tiles, 380, 210);
    expect(rects).toHaveLength(7);
    expect(rects.every(isReadableTile)).toBe(true);
  });

  it("keeps five-product sector readable", () => {
    const tiles = [
      tile("rb", 500),
      tile("hc", 400),
      tile("i", 300),
      tile("j", 200),
      tile("jm", 100),
    ];
    const rects = layoutSectorProducts(tiles, 360, 200);
    expect(rects).toHaveLength(5);
    expect(rects.every(isReadableTile)).toBe(true);
  });
});
