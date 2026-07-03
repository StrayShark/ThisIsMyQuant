import { NavLink } from "react-router-dom";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

/** Cursor 风格侧栏条目：无左侧色条，轻 hover / active 背景。 */
export function SidebarNavItem({
  to,
  label,
  icon: Icon,
  end = false,
  onClick,
}: {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
  onClick?: () => void;
}) {
  return (
    <NavLink
      to={to}
      end={end}
      onClick={onClick}
      aria-label={label}
      className={({ isActive }) =>
        cn(
          "flex items-center gap-2 rounded-[5px] px-2 py-[5px] text-[13px] leading-none transition-colors",
          isActive
            ? "bg-accent/60 text-foreground"
            : "text-muted-foreground hover:bg-accent/35 hover:text-foreground"
        )
      }
    >
      <Icon className="h-[15px] w-[15px] shrink-0 opacity-80" strokeWidth={1.75} />
      <span className="truncate">{label}</span>
    </NavLink>
  );
}
