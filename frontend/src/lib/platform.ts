/** 是否在 Tauri 桌面壳内运行。 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function isTauriMac(): boolean {
  return isTauriRuntime() && /Mac|iPhone|iPod|iPad/.test(navigator.platform);
}
