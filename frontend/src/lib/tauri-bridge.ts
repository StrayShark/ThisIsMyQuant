/** Tauri 核心就绪检测。 */

let appReady = false;

async function isTauri(): Promise<boolean> {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function waitForAppReady(): Promise<void> {
  if (!(await isTauri())) {
    appReady = true;
    return;
  }
  if (appReady) return;
  const { listen } = await import("@tauri-apps/api/event");
  return new Promise((resolve) => {
    listen("app-ready", () => {
      appReady = true;
      resolve();
    });
    setTimeout(() => {
      appReady = true;
      resolve();
    }, 8000);
  });
}

export function isAppReady(): boolean {
  return appReady;
}
