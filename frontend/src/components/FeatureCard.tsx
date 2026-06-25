import type { LucideIcon } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";

export function FeatureCard({
  icon: Icon,
  title,
  description,
  className,
  onClick,
}: {
  icon: LucideIcon;
  title: string;
  description: string;
  className?: string;
  onClick?: () => void;
}) {
  return (
    <Card
      className={cn(
        "transition-colors hover:border-hairline-strong hover:bg-muted/20",
        onClick && "cursor-pointer",
        className
      )}
      onClick={onClick}
    >
      <CardContent className="p-5">
        <div className="mb-3 flex h-8 w-8 items-center justify-center rounded-md border border-border bg-muted/40">
          <Icon className="h-4 w-4 text-foreground" />
        </div>
        <h3 className="text-sm font-semibold text-foreground">{title}</h3>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  );
}
