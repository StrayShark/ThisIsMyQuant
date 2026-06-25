import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
}: {
  icon?: LucideIcon;
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <Card className="border-dashed">
      <CardContent className="flex flex-col items-center py-16 text-center">
        {Icon && (
          <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-md border border-border bg-muted/40">
            <Icon className="h-5 w-5 text-muted-foreground" />
          </div>
        )}
        <p className="text-sm font-medium text-foreground">{title}</p>
        {description && (
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">{description}</p>
        )}
        {action && <div className="mt-6">{action}</div>}
      </CardContent>
    </Card>
  );
}
