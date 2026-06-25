/** 品种列表（左侧栏）。 */
import { BarChart3 } from "lucide-react";
import { useAppStore } from "@/app/store";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { FUTURES_SECTORS } from "@/data/futures";
import { useFuturesCatalog } from "@/hooks/useFuturesCatalog";
import { cn } from "@/lib/utils";

export function SymbolList() {
  const { currentSymbol, setCurrentSymbol } = useAppStore();
  const { data: sectors = FUTURES_SECTORS, isLoading } = useFuturesCatalog("core");

  return (
    <div className="flex h-full flex-col">
      <div className="panel-header px-3">
        <span className="text-sm font-medium">主流品种</span>
        <BarChart3 className="h-4 w-4 text-muted-foreground" />
      </div>

      <ScrollArea className="flex-1">
        <div className="pb-3">
          {isLoading && (
            <div className="px-3 py-4 text-xs text-muted-foreground">加载品种目录…</div>
          )}
          {sectors.map((sector) => (
            <section key={sector.code} className="border-b border-border/50 last:border-0">
              <div className="sticky top-0 z-10 flex items-center justify-between bg-background/95 px-3 py-2 backdrop-blur">
                <span className="text-xs font-semibold text-foreground">{sector.name}</span>
                <span className="text-[10px] text-muted-foreground">{sector.products.length}</span>
              </div>
              <div className="space-y-0.5 px-2 pb-2">
                {sector.products.map((product) => {
                  const active = product.symbol === currentSymbol;
                  return (
                    <button
                      key={product.symbol}
                      type="button"
                      onClick={() => setCurrentSymbol(product.symbol)}
                      className={cn(
                        "flex w-full items-center justify-between rounded-md px-2 py-2 text-left transition-colors hover:bg-muted/30",
                        active && "bg-muted/50 ring-1 ring-border"
                      )}
                    >
                      <span className="min-w-0">
                        <span className="block truncate text-sm font-medium text-foreground">
                          {product.name}
                        </span>
                        <span className="block font-mono text-[11px] text-muted-foreground">
                          {product.symbol} · {product.exchange}
                        </span>
                      </span>
                      {active && <Badge variant="secondary">主力</Badge>}
                    </button>
                  );
                })}
              </div>
            </section>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
