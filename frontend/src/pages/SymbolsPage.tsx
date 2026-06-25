import { useNavigate } from "react-router-dom";
import { BarChart3, Brain, Database, Radio } from "lucide-react";
import { PageHeader } from "@/components/PageHeader";
import { FeatureCard } from "@/components/FeatureCard";
import { useAppStore } from "@/app/store";
import { FUTURES_SECTORS } from "@/data/futures";
import { useFuturesCatalog } from "@/hooks/useFuturesCatalog";

const modules = [
  {
    icon: Database,
    title: "AKShare K 线",
    description: "免费历史与分钟 K 线，覆盖国内主要期货品种，无需 API Key。",
  },
  {
    icon: Radio,
    title: "K 线轮询",
    description: "周期性刷新分钟线，驱动图表 WebSocket 增量更新。",
  },
  {
    icon: BarChart3,
    title: "K 线图表",
    description: "TradingView Lightweight Charts，多周期切换与实时增量更新。",
  },
  {
    icon: Brain,
    title: "金十资讯",
    description: "金十财经期货板块新闻，纳入 LLM 分析上下文。",
  },
];

export function SymbolsPage() {
  const navigate = useNavigate();
  const { currentSymbol, setCurrentSymbol } = useAppStore();
  const { data: sectors = FUTURES_SECTORS } = useFuturesCatalog("core");

  return (
    <div className="page-scroll">
      <div className="page-inner">
        <PageHeader
          title="品种"
          description="按板块浏览高流动性主力连续合约（core tier）。"
        />

        <div className="mb-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {modules.map((m) => (
            <FeatureCard key={m.title} {...m} onClick={() => navigate("/")} />
          ))}
        </div>

        <div className="grid gap-4 lg:grid-cols-2">
          {sectors.map((sector) => (
            <section key={sector.code} className="rounded-lg border border-border bg-card">
              <div className="panel-header">
                <div>
                  <div className="text-sm font-medium">{sector.name}</div>
                  <div className="text-xs text-muted-foreground">{sector.description}</div>
                </div>
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
                        navigate("/");
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
