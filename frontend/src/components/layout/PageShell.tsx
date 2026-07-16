import type React from "react";
import { cn } from "@/lib/utils";

interface PageShellProps {
  children: React.ReactNode;
  className?: string;
}

export function PageShell({ children, className }: PageShellProps) {
  return (
    <div className="h-full overflow-auto bg-background">
      <div className={cn("mx-auto max-w-[1480px] px-8 py-7", className)}>{children}</div>
    </div>
  );
}
