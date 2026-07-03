import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: "http://localhost:5173",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "ui-mock",
      testMatch: /ui-mock\.spec\.ts/,
      use: {
        ...devices["Desktop Chrome"],
        channel: "chrome",
      },
    },
  ],
  webServer: {
    command: "node ./node_modules/vite/bin/vite.js",
    url: "http://localhost:5173",
    reuseExistingServer: process.env.PLAYWRIGHT_REUSE_SERVER === "true",
    env: {
      VITE_E2E_MOCK: "true",
    },
  },
});
