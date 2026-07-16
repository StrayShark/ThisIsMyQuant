/** Coinbase 风格应用壳：原生窗口标题栏 + 简洁顶部导航。 */
import type React from "react";
import { NavLink } from "react-router-dom";
import {
  BarChart3,
  Database,
  Settings,
  Activity,
  LayoutDashboard,
  TrendingUp,
  Star,
  Newspaper,
  Sparkles,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { ToastBanner } from "@/components/ToastBanner";
import { GlobalMarketBar } from "@/components/layout/GlobalMarketBar";

const mainNavItems = [
  { to: "/", label: "总览", icon: LayoutDashboard, end: true },
  { to: "/markets", label: "市场", icon: BarChart3, end: true },
  { to: "/watchlist", label: "自选", icon: Star, end: true },
  { to: "/simulation", label: "模拟盘", icon: TrendingUp, end: true },
  { to: "/events", label: "事件资讯", icon: Newspaper, end: true },
  { to: "/database", label: "数据库", icon: Database, end: true },
  { to: "/ai", label: "AI", icon: Sparkles, end: true },
  { to: "/settings?section=schedule", label: "设置", icon: Settings, end: false },
];

const utilityNavItems = [
  { to: "/status", label: "状态", icon: Activity, end: false },
];

export function AppShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <header className="flex h-16 shrink-0 items-center justify-between border-b border-border bg-background px-6">
        <div className="flex min-w-0 items-center gap-8">
          <NavLink to="/" className="flex items-center gap-2 text-[15px] font-semibold text-foreground">
            <span className="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-sm font-semibold text-primary-foreground">
              Q
            </span>
            <span className="truncate">ThisIsMyQuant</span>
          </NavLink>
          <nav className="flex items-center gap-1" aria-label="主导航">
            {mainNavItems.map((item) => (
              <TopNavItem key={item.to} {...item} />
            ))}
          </nav>
        </div>
        <nav className="flex items-center gap-1" aria-label="系统导航">
          {utilityNavItems.map((item) => (
            <TopNavItem key={item.to} {...item} compact />
          ))}
        </nav>
      </header>

      <GlobalMarketBar className="shrink-0" />
      <main className="min-h-0 flex-1 overflow-hidden bg-[var(--color-canvas-soft)]">
        {children}
      </main>

      <ToastBanner />
    </div>
  );
}

function TopNavItem({
  to,
  label,
  icon: Icon,
  end,
  compact = false,
}: {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
  compact?: boolean;
}) {
  return (
    <NavLink
      to={to}
      end={end}
      aria-label={label}
      className={({ isActive }) =>
        cn(
          "inline-flex h-10 items-center gap-2 rounded-full px-4 text-sm font-medium transition-colors",
          compact && "px-3",
          isActive
            ? "bg-primary text-primary-foreground"
            : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        )
      }
    >
      <Icon className="h-4 w-4" strokeWidth={1.8} />
      <span>{label}</span>
    </NavLink>
  );
}
