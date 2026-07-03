import { test } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), "../.screenshots");

test.describe("UI 截图验收", () => {
  test("捕获主要页面", async ({ page }) => {
    test.setTimeout(120_000);

    await page.setViewportSize({ width: 1440, height: 900 });

    await page.goto("/#/");
    await page.getByRole("link", { name: "总览" }).waitFor({ timeout: 20000 });
    await page.waitForTimeout(1500);
    await page.screenshot({ path: path.join(dir, "01-overview.png"), fullPage: false });

    await page.goto("/#/workspace");
    await page.getByText("Copilot").waitFor({ timeout: 15000 });
    await page.waitForTimeout(800);
    await page.screenshot({ path: path.join(dir, "03-workspace.png"), fullPage: false });

    await page.goto("/#/symbols");
    await page.getByText("黑色建材").waitFor({ timeout: 15000 });
    await page.screenshot({ path: path.join(dir, "04-symbols.png"), fullPage: false });

    await page.goto("/#/reports");
    await page.getByText("短期偏多，关注突破前高").first().waitFor({ timeout: 15000 });
    await page.screenshot({ path: path.join(dir, "05-reports.png"), fullPage: false });
  });
});
