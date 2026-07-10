/** 侧栏与顶栏分离：左列导航，右列顶栏 + 内容。 */
import { useEffect } from "react";
import { useLocation, useNavigate, Link } from "react-router-dom";
import {
  BarChart3,
  BellRing,
  CalendarClock,
  Database,
  FileText,
  Layers,
  Newspaper,
  Settings,
  Activity,
  LayoutDashboard,
  ArrowLeft,
  Sparkles,
  TrendingUp,
  History,
  PlayCircle,
  HardDrive,
  PieChart,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { ToastBanner } from "@/components/ToastBanner";
import { SidebarNavItem } from "@/components/SidebarNavItem";
import { WindowDragRegion } from "@/components/WindowDragRegion";
import { applyShellLayoutVars, MAC_SHELL } from "@/lib/shell-layout";
import {
  SETTINGS_SECTIONS,
  consumeSettingsReturnPath,
  parseSettingsSection,
  saveSettingsReturnPath,
} from "@/features/settings/settings-sections";

const mainNavItems = [
  { to: "/", label: "总览", icon: LayoutDashboard, end: true },
  { to: "/workspace", label: "行情", icon: BarChart3, end: true },
  { to: "/stocks", label: "A股", icon: PieChart, end: true },
  { to: "/simulation", label: "模拟盘", icon: TrendingUp, end: true },
  { to: "/review", label: "复盘", icon: History, end: true },
  { to: "/replay", label: "回放", icon: PlayCircle, end: true },
  { to: "/factors", label: "因子", icon: Database, end: true },
  { to: "/news", label: "资讯", icon: Newspaper, end: true },
  { to: "/calendar", label: "日历", icon: CalendarClock, end: true },
  { to: "/anomalies", label: "异动", icon: BellRing, end: true },
  { to: "/copilot", label: "助手", icon: Sparkles, end: true },
  { to: "/reports", label: "报告", icon: FileText, end: false },
  { to: "/symbols", label: "品种", icon: Layers, end: false },
  { to: "/database", label: "数据库", icon: HardDrive, end: true },
  { to: "/status", label: "状态", icon: Activity, end: false },
];

export function AppShell({ children }: { children: React.ReactNode }) {
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    applyShellLayoutVars();
  }, []);

  const onSettings = location.pathname === "/settings";
  const settingsSection = parseSettingsSection(
    new URLSearchParams(location.search).get("section")
  );

  const openSettings = () => {
    saveSettingsReturnPath(`${location.pathname}${location.search}`);
  };

  const leaveSettings = () => {
    navigate(consumeSettingsReturnPath());
  };

  return (
    <div
      className="grid h-screen bg-background text-foreground"
      style={{
        gridTemplateColumns: `${MAC_SHELL.sidebarExpanded}px minmax(0, 1fr)`,
        gridTemplateRows: "var(--shell-chrome-h) minmax(0, 1fr)",
      }}
    >
      {/* 左上：侧栏顶区（与右侧顶栏分离，为 macOS 信号灯留空） */}
      <div
        className="relative col-start-1 row-start-1 border-b border-r border-border bg-background"
        aria-hidden
      >
        <WindowDragRegion />
      </div>

      {/* 左下：导航 */}
      <aside className="col-start-1 row-start-2 flex min-h-0 min-w-0 flex-col border-r border-border bg-background">
        {onSettings ? (
          <>
            <div className="px-2 pt-2">
              <button
                type="button"
                onClick={leaveSettings}
                title="返回"
                aria-label="返回"
                className="flex w-full items-center gap-2 rounded-[5px] px-2 py-[5px] text-[13px] text-muted-foreground transition-colors hover:bg-accent/35 hover:text-foreground"
              >
                <ArrowLeft className="h-[15px] w-[15px] shrink-0" strokeWidth={1.75} />
                <span>返回</span>
              </button>
            </div>
            <nav className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-2">
              {SETTINGS_SECTIONS.map(({ id, label, icon: Icon }) => {
                const active = settingsSection === id;
                return (
                  <Link
                    key={id}
                    to={`/settings?section=${id}`}
                    className={cn(
                      "flex items-center gap-2 rounded-[5px] px-2 py-[5px] text-[13px] leading-none transition-colors",
                      active
                        ? "bg-accent/60 text-foreground"
                        : "text-muted-foreground hover:bg-accent/35 hover:text-foreground"
                    )}
                  >
                    <Icon className="h-[15px] w-[15px] shrink-0 opacity-80" strokeWidth={1.75} />
                    <span className="truncate">{label}</span>
                  </Link>
                );
              })}
            </nav>
          </>
        ) : (
          <>
            <nav className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-2 pt-1">
              {mainNavItems.map(({ to, label, icon, end }) => (
                <SidebarNavItem key={to} to={to} label={label} icon={icon} end={end} />
              ))}
            </nav>
            <div className="px-2 pb-3">
              <SidebarNavItem
                to="/settings?section=schedule"
                label="设置"
                icon={Settings}
                end={false}
                onClick={openSettings}
              />
            </div>
          </>
        )}
      </aside>

      {/* 右上：顶栏（仅主内容区，不与侧栏贯通） */}
      <div
        className="relative col-start-2 row-start-1 min-w-0 border-b border-border bg-background"
        data-tauri-drag-region
      >
        <WindowDragRegion />
      </div>

      {/* 右下：页面内容 */}
      <main className="col-start-2 row-start-2 min-h-0 min-w-0 overflow-hidden bg-background">
        {children}
      </main>

      <ToastBanner />
    </div>
  );
}
