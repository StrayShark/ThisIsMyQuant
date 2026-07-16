import type { UserPreferences } from "@/types";

export type QuoteColorScheme = UserPreferences["quote_color_scheme"];
export type AppTheme = UserPreferences["theme"];

/** 写入 html[data-theme] 的实际主题 id（system 会解析为 dark/light）。 */
export type AppliedTheme = Exclude<AppTheme, "system">;

export const THEME_OPTIONS: {
  id: AppTheme;
  title: string;
  description: string;
}[] = [
  { id: "light", title: "Coinbase 浅色", description: "白底、蓝色主操作、金融卡片" },
  { id: "system", title: "跟随系统", description: "系统浅色时使用 Coinbase，深色时使用克制暗色" },
];

export const DEFAULT_APPEARANCE: Pick<UserPreferences, "quote_color_scheme" | "theme"> = {
  quote_color_scheme: "green_up",
  theme: "light",
};

const VALID_THEMES = new Set<string>(["light", "system"]);

export function normalizeAppearance(raw: {
  quote_color_scheme?: string;
  theme?: string;
}): Pick<UserPreferences, "quote_color_scheme" | "theme"> {
  const quote_color_scheme: QuoteColorScheme =
    raw.quote_color_scheme === "red_up" ? "red_up" : "green_up";
  const theme: AppTheme =
    raw.theme && VALID_THEMES.has(raw.theme) ? (raw.theme as AppTheme) : "light";
  return { quote_color_scheme, theme };
}

export function resolveAppliedTheme(theme: AppTheme): AppliedTheme {
  if (theme === "light") return "light";
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return "light";
}

export function applyAppearance(prefs: Pick<UserPreferences, "quote_color_scheme" | "theme">) {
  const root = document.documentElement;
  root.dataset.quoteScheme = prefs.quote_color_scheme === "red_up" ? "red_up" : "green_up";
  root.dataset.theme = resolveAppliedTheme(prefs.theme);

  const applied = resolveAppliedTheme(prefs.theme);
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) {
    meta.setAttribute(
      "content",
      applied === "light" ? "#ffffff" : applied === "matrix" ? "#050805" : "#0a0a0a"
    );
  }

  window.dispatchEvent(new Event("appearance-change"));
}

let systemListener: (() => void) | null = null;

/** 跟随系统主题时监听 OS 切换。 */
export function bindSystemThemeListener(
  theme: AppTheme,
  onChange: () => void
) {
  if (systemListener) {
    window.matchMedia("(prefers-color-scheme: dark)").removeEventListener("change", systemListener);
    systemListener = null;
  }
  if (theme !== "system") return;

  systemListener = () => onChange();
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", systemListener);
}
