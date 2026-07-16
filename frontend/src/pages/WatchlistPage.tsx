import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { PageShell } from "@/components/layout/PageShell";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { api } from "@/api/client";
import { useAppStore } from "@/app/store";
import { WatchlistGroupTabs } from "@/features/watchlist/WatchlistGroupTabs";
import { WatchlistTable } from "@/features/watchlist/WatchlistTable";
import { WatchlistSummaryPanel } from "@/features/watchlist/WatchlistSummaryPanel";
import { WatchlistEventPanel } from "@/features/watchlist/WatchlistEventPanel";
import { WatchlistAiSummary } from "@/features/watchlist/WatchlistAiSummary";
import { cn } from "@/lib/utils";

export function WatchlistPage() {
  const queryClient = useQueryClient();
  const showToast = useAppStore((s) => s.showToast);
  const [selectedGroupId, setSelectedGroupId] = useState<string>("all");

  const { data: groups = [], isLoading: groupsLoading } = useQuery({
    queryKey: ["watchlist-groups"],
    queryFn: () => api.listWatchlistGroups(),
  });

  const createGroupMutation = useMutation({
    mutationFn: (name: string) => api.createWatchlistGroup({ name }),
    onSuccess: (group) => {
      showToast(`分组 ${group.name} 已创建`);
      queryClient.invalidateQueries({ queryKey: ["watchlist-groups"] });
      setSelectedGroupId(group.id);
    },
    onError: (err: Error) => showToast(err.message || "创建分组失败"),
  });

  const deleteGroupMutation = useMutation({
    mutationFn: (id: string) => api.deleteWatchlistGroup(id),
    onSuccess: () => {
      showToast("分组已删除");
      queryClient.invalidateQueries({ queryKey: ["watchlist-groups"] });
      setSelectedGroupId("all");
    },
    onError: (err: Error) => showToast(err.message || "删除分组失败"),
  });

  const handleCreateGroup = (name: string) => {
    createGroupMutation.mutate(name);
  };

  const handleDeleteGroup = (group: { id: string; name: string }) => {
    if (!confirm(`确定删除分组「${group.name}」？该分组内的自选标的也会被移除。`)) {
      return;
    }
    deleteGroupMutation.mutate(group.id);
  };

  return (
    <PageShell>
      <PageHeader
        title="自选"
        description="我关注的标的与异动"
      >
        <Button
          size="sm"
          className="h-8 gap-1 rounded-full text-xs"
          onClick={() => {
            const name = prompt("请输入新分组名称", "重点观察");
            if (name?.trim()) {
              handleCreateGroup(name.trim());
            }
          }}
        >
          <Plus className="h-3.5 w-3.5" />
          新建分组
        </Button>
      </PageHeader>

      {groups.length === 0 && !groupsLoading ? (
        <Card>
          <CardContent className="flex h-64 flex-col items-center justify-center gap-4 p-8 text-center">
            <p className="text-sm text-muted-foreground">暂无分组</p>
            <Button
              size="sm"
              className="h-8 gap-1 rounded-full text-xs"
              onClick={() => {
                const name = prompt("请输入新分组名称", "重点观察");
                if (name?.trim()) {
                  handleCreateGroup(name.trim());
                }
              }}
            >
              <Plus className="h-3.5 w-3.5" />
              新建分组
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          <WatchlistGroupTabs
            groups={groups}
            activeGroupId={selectedGroupId}
            onChange={setSelectedGroupId}
            onCreate={handleCreateGroup}
            onDelete={handleDeleteGroup}
            isLoading={groupsLoading}
          />

          <div className={cn("mt-5 grid gap-5", "grid-cols-1 lg:grid-cols-[1fr_360px]")}>
            <WatchlistTable groupId={selectedGroupId} />
            <div className="space-y-4">
              <WatchlistSummaryPanel />
              <WatchlistEventPanel />
              <WatchlistAiSummary />
            </div>
          </div>
        </>
      )}
    </PageShell>
  );
}
