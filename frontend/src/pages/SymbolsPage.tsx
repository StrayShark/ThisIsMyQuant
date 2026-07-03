import { useNavigate } from "react-router-dom";
import { useAppStore } from "@/app/store";
import { FUTURES_SECTORS } from "@/data/futures";
import { useFuturesCatalog } from "@/hooks/useFuturesCatalog";

export function SymbolsPage() {
  const navigate = useNavigate();
  const { currentSymbol, setCurrentSymbol } = useAppStore();
  const { data: sectors = FUTURES_SECTORS } = useFuturesCatalog("core");

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <div className="grid gap-4 lg:grid-cols-2">
          {sectors.map((sector) => (
            <section key={sector.code} className="rounded-lg border border-border bg-card">
              <div className="panel-header">
                <div className="text-sm font-medium">{sector.name}</div>
                <span className="text-xs text-muted-foreground">{sector.products.length} 个</span>
              </div>
              <div className="grid grid-cols-2 gap-2 p-3 sm:grid-cols-3">
                {sector.products.map((product) => {
                  const active = product.symbol === currentSymbol;
                  return (
                    <button
                      key={product.symbol}
                      type="button"
                      onClick={() => {
                        setCurrentSymbol(product.symbol);
                        navigate(`/symbols/${product.symbol}`);
                      }}
                      className={`rounded-md border px-3 py-2 text-left transition-colors ${
                        active
                          ? "border-primary bg-muted/50"
                          : "border-border hover:bg-muted/30"
                      }`}
                    >
                      <div className="truncate text-sm font-medium text-foreground">
                        {product.name}
                      </div>
                      <div className="font-mono text-xs text-muted-foreground">
                        {product.symbol} · {product.exchange}
                      </div>
                    </button>
                  );
                })}
              </div>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}
