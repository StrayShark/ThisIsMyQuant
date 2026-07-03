/**
 * macOS Overlay 标题栏布局常量。
 * 须与 src-tauri/tauri.conf.json → windows[0].trafficLightPosition 保持一致。
 */
export const MAC_TRAFFIC_LIGHT = {
  x: 14,
  y: 24,
  size: 12,
  gap: 8,
} as const;

export const MAC_SHELL = {
  chromeHeight: 52,
  sidebarExpanded: 220,
} as const;

export function applyShellLayoutVars(): void {
  const html = document.documentElement;
  const isMacShell =
    html.classList.contains("tauri-mac") ||
    (typeof window !== "undefined" &&
      "__TAURI_INTERNALS__" in window &&
      /Mac|iPhone|iPod|iPad/.test(navigator.userAgent));

  if (!isMacShell) return;

  const { x, y, size, gap } = MAC_TRAFFIC_LIGHT;
  html.classList.add("tauri-mac");
  html.style.setProperty("--shell-chrome-h", `${MAC_SHELL.chromeHeight}px`);
  html.style.setProperty("--shell-controls-h", `${MAC_SHELL.chromeHeight}px`);
  html.style.setProperty("--shell-sidebar-expanded", `${MAC_SHELL.sidebarExpanded}px`);
  html.style.setProperty("--shell-traffic-x", `${x}px`);
  html.style.setProperty("--shell-traffic-y", `${y}px`);
  html.style.setProperty("--shell-traffic-size", `${size}px`);
  html.style.setProperty("--shell-traffic-gap", `${gap}px`);
}
