import { useState, useRef, useEffect } from "react";
import { Plus, X, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { WatchlistGroup } from "@/types";

interface WatchlistGroupTabsProps {
  groups: WatchlistGroup[];
  activeGroupId: string;
  onChange: (groupId: string) => void;
  onCreate: (name: string) => void;
  onDelete: (group: WatchlistGroup) => void;
  isLoading?: boolean;
}

export function WatchlistGroupTabs({
  groups,
  activeGroupId,
  onChange,
  onCreate,
  onDelete,
  isLoading,
}: WatchlistGroupTabsProps) {
  const [isCreating, setIsCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isCreating) {
      inputRef.current?.focus();
    }
  }, [isCreating]);

  const handleCreate = () => {
    const trimmed = newName.trim();
    if (trimmed) {
      onCreate(trimmed);
    }
    setNewName("");
    setIsCreating(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleCreate();
    } else if (e.key === "Escape") {
      setNewName("");
      setIsCreating(false);
    }
  };

  return (
    <div className="flex flex-wrap items-center gap-2">
      <div className="inline-flex h-9 flex-wrap items-center gap-1 rounded-lg bg-muted p-1">
        <TabButton
          label="全部"
          isActive={activeGroupId === "all"}
          onClick={() => onChange("all")}
        />
        {groups.map((group) => (
          <div key={group.id} className="group relative">
            <TabButton
              label={group.name}
              isActive={activeGroupId === group.id}
              onClick={() => onChange(group.id)}
            />
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(group);
              }}
              className={cn(
                "absolute -right-1 -top-1 flex h-4 w-4 items-center justify-center rounded-full bg-background text-muted-foreground opacity-0 shadow-sm ring-1 ring-border transition-opacity hover:text-destructive",
                activeGroupId === group.id && "opacity-100",
                "group-hover:opacity-100"
              )}
              aria-label={`删除分组 ${group.name}`}
            >
              <Trash2 className="h-2.5 w-2.5" />
            </button>
          </div>
        ))}
        {isCreating && (
          <div className="flex h-7 items-center rounded-sm bg-background px-1">
            <Input
              ref={inputRef}
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={handleKeyDown}
              onBlur={handleCreate}
              placeholder="分组名称"
              className="h-6 w-28 rounded-sm border-border px-2 py-0 text-xs"
            />
            <button
              type="button"
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => {
                setNewName("");
                setIsCreating(false);
              }}
              className="ml-1 rounded-full p-0.5 text-muted-foreground hover:text-foreground"
              aria-label="取消"
            >
              <X className="h-3 w-3" />
            </button>
          </div>
        )}
      </div>

      {!isCreating && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-8 gap-1 rounded-full text-xs"
          onClick={() => setIsCreating(true)}
          disabled={isLoading}
        >
          <Plus className="h-3.5 w-3.5" />
          新建分组
        </Button>
      )}
    </div>
  );
}

interface TabButtonProps {
  label: string;
  isActive: boolean;
  onClick: () => void;
}

function TabButton({ label, isActive, onClick }: TabButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "relative inline-flex h-7 max-w-[140px] items-center justify-center truncate rounded-sm px-3 text-xs font-medium transition-colors",
        isActive
          ? "bg-background text-foreground shadow-sm ring-1 ring-border"
          : "text-muted-foreground hover:bg-background/50 hover:text-foreground"
      )}
      aria-pressed={isActive}
    >
      <span className="truncate">{label}</span>
    </button>
  );
}
