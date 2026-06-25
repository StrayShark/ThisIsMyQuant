/** Cursor 风格极简侧栏 + 顶栏布局。 */
import { NavLink } from "react-router-dom";
import { BarChart3, FileText, Layers, Settings } from "lucide-react";
import { cn } from "@/lib/utils";
import { Separator } from "@/components/ui/separator";
import { TopBar } from "@/components/TopBar";
import { useAppStore } from "@/app/store";

const navItems = [
  { to: "/", label: "行情", icon: BarChart3, end: true },
  { to: "/reports", label: "报告", icon: FileText, end: false },
  { to: "/symbols", label: "品种", icon: Layers, end: false },
  { to: "/settings", label: "设置", icon: Settings, end: false },
];

export function AppShell({ children }: { children: React.ReactNode }) {
  const { akshareOnline, jinshiOnline, statusMessage } = useAppStore();

  const online = akshareOnline;
  const statusLabel = online
    ? jinshiOnline
      ? "新浪 · 金十"
      : "新浪"
    : "数据离线";

  return (
    <div className="flex h-screen bg-background text-foreground">
      <aside className="flex w-[220px] shrink-0 flex-col border-r border-border bg-background">
        <div className="flex h-11 items-center px-4">
          <span className="text-sm font-medium tracking-tight text-foreground/90">
            ThisIsMyQuant
          </span>
        </div>

        <Separator />

        <nav className="flex flex-1 flex-col gap-0.5 p-2">
          {navItems.map(({ to, label, icon: Icon, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2.5 rounded-sm border-l-2 border-transparent py-2 pl-[10px] pr-3 text-sm font-medium text-muted-foreground transition-colors hover:bg-muted/40 hover:text-foreground",
                  isActive && "border-l-primary bg-muted/40 text-foreground"
                )
              }
            >
              <Icon className="h-4 w-4 shrink-0 opacity-70" />
              {label}
            </NavLink>
          ))}
        </nav>

        <div className="border-t border-border p-4" title={statusMessage}>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className={cn("h-2 w-2 shrink-0 rounded-full", online ? "bg-up" : "bg-down")} />
            <span className="truncate">{statusLabel}</span>
          </div>
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar />
        <main className="min-h-0 flex-1 overflow-hidden bg-background">{children}</main>
      </div>
    </div>
  );
}
