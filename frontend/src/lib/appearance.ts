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
  { id: "cursor", title: "Cursor 黑白", description: "灰阶极简，白/黑 accent" },
  { id: "matrix", title: "Matrix", description: "Codex 磷光绿终端风" },
  { id: "dark", title: "深色", description: "经典深色 + 蓝色强调" },
  { id: "light", title: "浅色", description: "明亮背景" },
  { id: "system", title: "跟随系统", description: "自动匹配 OS 深/浅" },
];

export const DEFAULT_APPEARANCE: Pick<UserPreferences, "quote_color_scheme" | "theme"> = {
  quote_color_scheme: "green_up",
  theme: "cursor",
};

const VALID_THEMES = new Set<string>(["dark", "light", "system", "cursor", "matrix"]);

export function normalizeAppearance(raw: {
  quote_color_scheme?: string;
  theme?: string;
}): Pick<UserPreferences, "quote_color_scheme" | "theme"> {
  const quote_color_scheme: QuoteColorScheme =
    raw.quote_color_scheme === "red_up" ? "red_up" : "green_up";
  const theme: AppTheme =
    raw.theme && VALID_THEMES.has(raw.theme) ? (raw.theme as AppTheme) : "cursor";
  return { quote_color_scheme, theme };
}

export function resolveAppliedTheme(theme: AppTheme): AppliedTheme {
  if (theme === "light") return "light";
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  if (theme === "matrix") return "matrix";
  if (theme === "cursor") return "cursor";
  return "dark";
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
