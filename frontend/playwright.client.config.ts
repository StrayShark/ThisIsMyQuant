import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  timeout: 600_000,
  reporter: [["list"], ["html", { open: "never" }]],
  globalSetup: "./e2e/global-setup-client.ts",
  projects: [
    {
      name: "client-live",
      testMatch: /client-live\.spec\.ts/,
      use: {
        ...devices["Desktop Chrome"],
        channel: "chrome",
      },
    },
  ],
});
