import { useEffect, type ReactNode } from "react";
import {
  applyAppearance,
  bindSystemThemeListener,
  normalizeAppearance,
  THEME_OPTIONS,
  type AppTheme,
  type QuoteColorScheme,
} from "@/lib/appearance";
import { useUserPreferences } from "@/hooks/useUserPreferences";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

function OptionCard({
  active,
  title,
  preview,
  onClick,
}: {
  active: boolean;
  title: string;
  preview?: ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex w-full flex-col rounded-lg border p-3 text-left transition-colors",
        active
          ? "border-primary bg-primary/5 ring-1 ring-primary/30"
          : "border-border bg-muted/10 hover:bg-muted/25"
      )}
    >
      {preview}
      <span className="mt-2 text-sm font-medium text-foreground">{title}</span>
    </button>
  );
}

function ThemePreview({ id }: { id: AppTheme }) {
  if (id === "cursor") {
    return (
      <div className="flex h-8 w-full items-center gap-1 rounded border border-neutral-700 bg-[#0a0a0a] px-2">
        <span className="h-2 w-2 rounded-full bg-white/90" />
        <span className="h-1 flex-1 rounded bg-neutral-600" />
      </div>
    );
  }
  if (id === "matrix") {
    return (
      <div className="flex h-8 w-full flex-col justify-center rounded border border-[#1a3d1a] bg-[#050805] px-2 font-mono text-[10px] text-[#39ff14]">
        <span className="opacity-90">{">"} market.ready</span>
      </div>
    );
  }
  if (id === "light") {
    return <div className="h-8 w-full rounded border border-neutral-200 bg-white" />;
  }
  if (id === "system") {
    return <div className="h-8 w-full rounded border border-neutral-500 bg-gradient-to-r from-black to-white" />;
  }
  return <div className="h-8 w-full rounded border border-neutral-700 bg-black" />;
}

function QuotePreview({ up, down }: { up: string; down: string }) {
  return (
    <div className="flex gap-3 font-mono text-sm">
      <span style={{ color: up }}>+1.25%</span>
      <span style={{ color: down }}>-0.86%</span>
    </div>
  );
}

export function AppearanceSettingsPanel() {
  const { prefs, isLoading, update } = useUserPreferences();

  useEffect(() => {
    if (!prefs) return;
    const appearance = normalizeAppearance(prefs);
    applyAppearance(appearance);
    bindSystemThemeListener(appearance.theme, () => applyAppearance(appearance));
  }, [prefs?.quote_color_scheme, prefs?.theme, prefs]);

  if (isLoading || !prefs) {
    return <Skeleton className="h-64 rounded-lg" />;
  }

  const greenUp = prefs.quote_color_scheme === "green_up";
  const redUp = prefs.quote_color_scheme === "red_up";

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-semibold">外观与涨跌色</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <section>
          <p className="mb-3 text-xs font-medium text-muted-foreground">涨跌色</p>
          <div className="grid gap-3 sm:grid-cols-2">
            <OptionCard
              active={greenUp}
              title="绿涨红跌"
              preview={<QuotePreview up="#50e3c2" down="#ee0000" />}
              onClick={() => update({ quote_color_scheme: "green_up" as QuoteColorScheme })}
            />
            <OptionCard
              active={redUp}
              title="红涨绿跌"
              preview={<QuotePreview up="#ee0000" down="#50e3c2" />}
              onClick={() => update({ quote_color_scheme: "red_up" as QuoteColorScheme })}
            />
          </div>
        </section>

        <section>
          <p className="mb-3 text-xs font-medium text-muted-foreground">主题</p>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {THEME_OPTIONS.map(({ id, title }) => (
              <OptionCard
                key={id}
                active={prefs.theme === id}
                title={title}
                preview={<ThemePreview id={id} />}
                onClick={() => update({ theme: id })}
              />
            ))}
          </div>
        </section>
      </CardContent>
    </Card>
  );
}
