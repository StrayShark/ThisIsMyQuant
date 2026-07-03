import { describe, expect, it, vi } from "vitest";
import { MARKET_CSS, marketUpDown, withMarketCandleColors } from "./market-colors";

describe("marketUpDown", () => {
  it("returns up colors for positive change", () => {
    expect(marketUpDown(1.5)).toEqual({
      fg: MARKET_CSS.up,
      bg: MARKET_CSS.upBg,
    });
  });

  it("returns up colors for zero change", () => {
    expect(marketUpDown(0)).toEqual({
      fg: MARKET_CSS.up,
      bg: MARKET_CSS.upBg,
    });
  });

  it("returns down colors for negative change", () => {
    expect(marketUpDown(-0.5)).toEqual({
      fg: MARKET_CSS.down,
      bg: MARKET_CSS.downBg,
    });
  });

  it("returns neutral colors for null", () => {
    expect(marketUpDown(null)).toEqual({
      fg: "var(--color-muted)",
      bg: MARKET_CSS.neutral,
    });
  });
});

describe("withMarketCandleColors", () => {
  it("overrides up/down colors with computed CSS variables", () => {
    vi.spyOn(window, "getComputedStyle").mockReturnValue({
      getPropertyValue: (name: string) =>
        name === "--market-up" ? "#00ff00" : name === "--market-down" ? "#ff0000" : "",
    } as CSSStyleDeclaration);

    const result = withMarketCandleColors({ upColor: "", downColor: "", lineWidth: 2 });
    expect(result.upColor).toBe("#00ff00");
    expect(result.downColor).toBe("#ff0000");
    expect(result.lineWidth).toBe(2);
  });
});
