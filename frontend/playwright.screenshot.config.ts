import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  testMatch: /ui-screenshot\.spec\.ts/,
  timeout: 120_000,
  use: {
    baseURL: "http://localhost:5173",
    ...devices["Desktop Chrome"],
    channel: "chrome",
  },
  webServer: {
    command: "pnpm dev",
    url: "http://localhost:5173",
    reuseExistingServer: true,
    env: { VITE_E2E_MOCK: "true" },
  },
});
