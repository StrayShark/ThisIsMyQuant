import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Trash2 } from "lucide-react";
import type { StockWatchlist } from "@/types";

interface StockWatchlistViewProps {
  onSelectStock: (tsCode: string) => void;
}

export function StockWatchlistView({ onSelectStock }: StockWatchlistViewProps) {
  const queryClient = useQueryClient();
  const [newName, setNewName] = useState("");

  const { data: watchlists, isLoading } = useQuery({
    queryKey: ["stock-watchlists"],
    queryFn: () => api.listStockWatchlists(),
  });

  const createMutation = useMutation({
    mutationFn: () => api.saveStockWatchlist({ name: newName || "我的自选股", symbols: [] }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stock-watchlists"] });
      setNewName("");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteStockWatchlist(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["stock-watchlists"] }),
  });

  if (isLoading) {
    return <div className="p-4 text-sm text-muted-foreground">加载中…</div>;
  }

  return (
    <div className="flex h-full flex-col gap-4 overflow-auto p-4">
      <div className="rounded-lg border border-border bg-card p-4 shadow-sm">
        <h3 className="mb-3 text-sm font-medium">新建股票池</h3>
        <div className="flex gap-2">
          <Input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="股票池名称"
            className="h-8 text-xs"
          />
          <Button
            size="sm"
            onClick={() => createMutation.mutate()}
            disabled={createMutation.isPending}
          >
            创建
          </Button>
        </div>
      </div>

      {watchlists && watchlists.length > 0 ? (
        watchlists.map((list: StockWatchlist) => (
          <div key={list.id} className="rounded-lg border border-border bg-card p-4 shadow-sm">
            <div className="mb-2 flex items-center justify-between">
              <h3 className="text-sm font-medium">{list.name}</h3>
              <button
                type="button"
                onClick={() => deleteMutation.mutate(list.id)}
                className="text-muted-foreground hover:text-rose-500"
                title="删除"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
            {list.symbols.length === 0 ? (
              <div className="text-xs text-muted-foreground">暂无股票，可在个股页添加</div>
            ) : (
              <div className="flex flex-wrap gap-2">
                {list.symbols.map((tsCode) => (
                  <button
                    key={tsCode}
                    type="button"
                    onClick={() => onSelectStock(tsCode)}
                    className="rounded-md bg-muted px-2 py-1 text-xs hover:bg-accent"
                  >
                    {tsCode}
                  </button>
                ))}
              </div>
            )}
          </div>
        ))
      ) : (
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
          暂无自选股池，请创建或从个股页添加
        </div>
      )}
    </div>
  );
}
