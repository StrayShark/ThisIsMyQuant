import { test, expect } from "@playwright/test";

test.describe("应用壳与导航", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByText("ThisIsMyQuant")).toBeVisible({ timeout: 15000 });
  });

  test("启动后显示侧栏与主工作台", async ({ page }) => {
    await expect(page.getByRole("link", { name: "行情" })).toBeVisible();
    await expect(page.getByText("Copilot")).toBeVisible();
    await expect(page.getByText("主流品种")).toBeVisible();
  });

  test("四页路由均可访问", async ({ page }) => {
    await page.getByRole("link", { name: "报告" }).click();
    await expect(page.getByRole("heading", { name: "分析报告" })).toBeVisible();

    await page.getByRole("link", { name: "品种" }).click();
    await expect(page.getByRole("heading", { name: "品种" })).toBeVisible();

    await page.getByRole("link", { name: "设置" }).click();
    await expect(page.getByRole("heading", { name: "设置" })).toBeVisible();

    await page.getByRole("link", { name: "行情" }).click();
    await expect(page.getByText("Copilot")).toBeVisible();
  });

  test("侧栏数据源状态显示", async ({ page }) => {
    await expect(page.getByText(/新浪|数据离线/).first()).toBeVisible({ timeout: 10000 });
  });
});

test.describe("行情工作台", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/");
    await expect(page.getByText("ThisIsMyQuant")).toBeVisible({ timeout: 15000 });
  });

  test("K 线周期切换 Tab 可用", async ({ page }) => {
    const tabs = page.locator(".panel-header [role='tab']");
    await tabs.filter({ hasText: /^5m$/ }).click();
    await expect(tabs.filter({ hasText: /^5m$/ })).toHaveAttribute("data-state", "active");
    await tabs.filter({ hasText: /^1d$/ }).click();
    await expect(tabs.filter({ hasText: /^1d$/ })).toHaveAttribute("data-state", "active");
  });

  test("品种切换更新图表标题", async ({ page }) => {
    await page.getByRole("button", { name: /黄金/ }).click();
    await expect(page.locator(".panel-header").getByText("AU0 · 1d")).toBeVisible();
  });

  test("Copilot 流式分析按钮可触发", async ({ page }) => {
    const submit = page.locator("button").filter({ has: page.locator(".lucide-arrow-up") });
    await submit.click();
    await expect(page.getByText(/E2E 流式分析片段|E2E 模拟分析报告/)).toBeVisible({
      timeout: 8000,
    });
  });
});

test.describe("报告与设置", () => {
  test("报告页展示 Mock 报告卡片", async ({ page }) => {
    await page.goto("/#/reports");
    await expect(page.getByRole("heading", { name: "分析报告" })).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("E2E 模拟分析报告")).toBeVisible({ timeout: 10000 });
  });

  test("设置页展示数据源与大模型配置", async ({ page }) => {
    await page.goto("/#/settings");
    await expect(page.getByText("AKShare K 线")).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("金十资讯")).toBeVisible();
    await expect(page.getByText("大模型")).toBeVisible();
    await expect(page.getByText("刷新配置")).toBeVisible();
  });
});

test.describe("品种页", () => {
  test("展示六大板块模块说明", async ({ page }) => {
    await page.goto("/#/symbols");
    await expect(page.getByRole("heading", { name: "品种" })).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("AKShare K 线")).toBeVisible();
    await expect(page.getByText("金十资讯")).toBeVisible();
  });
});
