/** Cursor 风格极简顶栏：合约搜索。 */
import { useCallback, useEffect, useState } from "react";
import { Search } from "lucide-react";
import { useNavigate, useLocation } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { useAppStore } from "@/app/store";
import { findFuturesSymbol, getFuturesCatalog, getFuturesProduct } from "@/data/futures";
import { useFuturesCatalog } from "@/hooks/useFuturesCatalog";
import { ensureMarketSubscription } from "@/lib/market-subscribe";

export function TopBar() {
  const navigate = useNavigate();
  const location = useLocation();
  const { currentSymbol, setCurrentSymbol, watchlist } = useAppStore();
  useFuturesCatalog("all");
  const searchProducts = getFuturesCatalog("all").flatMap((s) => s.products);
  const currentProduct = getFuturesProduct(currentSymbol);
  const [query, setQuery] = useState(currentProduct?.name || currentSymbol);

  useEffect(() => {
    setQuery(currentProduct?.name || currentSymbol);
  }, [currentProduct?.name, currentSymbol]);

  const submit = useCallback(() => {
    const sym = findFuturesSymbol(query) || query.trim().toUpperCase();
    if (!sym) return;
    setCurrentSymbol(sym);
    void ensureMarketSubscription(sym);
    if (location.pathname !== "/") navigate("/");
  }, [query, setCurrentSymbol, navigate, location.pathname]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        document.getElementById("symbol-search")?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <header className="flex h-16 shrink-0 items-center border-b border-border px-4">
      <div className="relative mx-auto w-full max-w-xl">
        <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          id="symbol-search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          placeholder="搜索中文品种，如 螺纹钢 / 黄金"
          className="h-8 border-border bg-muted/30 pl-9 pr-14 text-sm"
          list="watchlist-suggestions"
        />
        <datalist id="watchlist-suggestions">
          {searchProducts
            .filter(
              (product) =>
                watchlist.includes(product.symbol) ||
                watchlist.some((w) => w.toLowerCase().startsWith(product.code))
            )
            .map((product) => (
            <option key={product.symbol} value={product.name}>
              {product.symbol} · {product.exchange}
            </option>
          ))}
        </datalist>
        <kbd className="pointer-events-none absolute right-3 top-1/2 hidden -translate-y-1/2 rounded border border-border bg-muted/50 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground sm:inline">
          ⌘K
        </kbd>
      </div>
    </header>
  );
}
