import { useState, useEffect, useRef, useCallback } from "react";
import { useQuery } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { api } from "@/api/client";
import type { MarketAsset } from "@/types";
import { cn } from "@/lib/utils";

interface AssetSearchProps {
  className?: string;
}

export function AssetSearch({ className }: AssetSearchProps) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const containerRef = useRef<HTMLDivElement>(null);

  const debouncedQuery = query.trim();

  const { data: result } = useQuery({
    queryKey: ["asset-search", debouncedQuery],
    queryFn: () => api.searchAssets(debouncedQuery, 8),
    enabled: debouncedQuery.length > 0,
    staleTime: 30_000,
  });

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleSelect = useCallback(
    (asset: MarketAsset) => {
      setOpen(false);
      setQuery("");
      const path = asset.market === "futures" ? `/markets/futures/${asset.symbol}` : `/markets/stocks/${asset.symbol}`;
      navigate(path);
    },
    [navigate]
  );

  const assets = result?.assets ?? [];

  return (
    <div ref={containerRef} className={cn("relative", className)}>
      <div className="relative">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          type="text"
          placeholder="搜索标的代码或名称…"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && assets[0]) {
              handleSelect(assets[0]);
            }
            if (e.key === "Escape") {
              setOpen(false);
            }
          }}
          className="h-9 w-[240px] rounded-full pl-9 text-sm"
        />
      </div>

      {open && debouncedQuery.length > 0 && (
        <div className="absolute right-0 top-full z-50 mt-2 w-[320px] rounded-xl border border-border bg-card p-2 shadow-lg">
          {assets.length === 0 ? (
            <div className="px-3 py-4 text-center text-xs text-muted-foreground">未找到标的</div>
          ) : (
            <ul className="max-h-[280px] overflow-auto">
              {assets.map((asset) => (
                <li key={`${asset.market}:${asset.symbol}`}>
                  <button
                    type="button"
                    onClick={() => handleSelect(asset)}
                    className="flex w-full items-center justify-between rounded-lg px-3 py-2 text-left hover:bg-muted/50"
                  >
                    <div>
                      <div className="text-sm font-medium">{asset.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {asset.symbol} · {asset.market === "futures" ? "期货" : "A股"}
                      </div>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {asset.sector ?? asset.industry ?? "--"}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
