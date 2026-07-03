import { test, expect } from "@playwright/test";

test.describe("应用壳与导航", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("link", { name: "总览" })).toBeVisible({ timeout: 15000 });
  });

  test("启动后默认进入市场总览", async ({ page }) => {
    await expect(page.getByRole("link", { name: "总览" })).toBeVisible();
    await expect(page.getByText("专业分析工作台")).toBeVisible();
    await expect(page.getByText("热力图")).toBeVisible();
    await expect(page.getByText("LLM 多空建议")).toBeVisible();
  });

  test("各页路由均可访问", async ({ page }) => {
    await page.locator("aside nav").getByRole("link", { name: "行情" }).click();
    await expect(page.getByText("Copilot")).toBeVisible();
    await expect(page.getByText("主流品种")).toBeVisible();

    await page.getByRole("link", { name: "报告" }).click();
    await expect(page.getByText("短期偏多，关注突破前高").first()).toBeVisible({
      timeout: 10000,
    });

    await page.getByRole("link", { name: "品种" }).click();
    await expect(page.getByText("黑色建材")).toBeVisible();

    await page.locator("aside").getByRole("link", { name: "设置" }).click();
    await expect(page.getByText("每").first()).toBeVisible({ timeout: 10000 });
    await expect(page.getByRole("button", { name: "返回" })).toBeVisible();
    await page.getByRole("button", { name: "返回" }).click();
    await expect(page.getByText("黑色建材")).toBeVisible();
  });

  test("侧栏底部设置入口", async ({ page }) => {
    await expect(page.locator("aside").getByRole("link", { name: "设置" })).toBeVisible({
      timeout: 10000,
    });
  });
});

test.describe("市场总览", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/");
    await expect(page.getByRole("link", { name: "总览" })).toBeVisible({ timeout: 15000 });
  });

  test("展示板块热力与资讯摘要", async ({ page }) => {
    await expect(page.getByText("黑色建材").first()).toBeVisible();
    await expect(page.getByText("能源化工").first()).toBeVisible();
    await expect(page.getByText("航运运价").first()).toBeVisible();
    await expect(page.getByText("资讯决策流")).toBeVisible();
    await expect(page.getByText("产业因子")).toBeVisible();
    await expect(page.getByText("异动预警")).toBeVisible();
    await expect(page.getByText("报告流程")).toBeVisible();
    await expect(page.getByText("外盘联动")).toBeVisible();
    await expect(page.getByRole("link", { name: /螺纹钢 \+/ }).first()).toBeVisible();
    await expect(page.getByText("重要数据时间窗")).toBeVisible();
    await expect(page.getByText("资讯 · 宏观摘要")).toBeVisible();
    await expect(page.getByText("短期偏多，关注突破前高")).toBeVisible({ timeout: 10000 });
  });
});

test.describe("行情工作台", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/workspace");
    await expect(page.getByRole("link", { name: "总览" })).toBeVisible({ timeout: 15000 });
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
    await expect(page.getByText("短期偏多，关注突破前高").first()).toBeVisible({
      timeout: 15000,
    });
  });

  test("设置页展示数据源与大模型配置", async ({ page }) => {
    await page.goto("/#/settings?section=data");
    await expect(page.getByText("AKShare K 线").first()).toBeVisible({ timeout: 15000 });
    await page.goto("/#/settings?section=llm");
    await expect(page.getByRole("link", { name: "配置 LLM API Key" })).toBeVisible();
    await page.goto("/#/settings?section=storage");
    await expect(page.getByText("重载 .env（金十等）")).toBeVisible();
  });
});

test.describe("品种页", () => {
  test("展示品种板块列表", async ({ page }) => {
    await page.goto("/#/symbols");
    await expect(page.getByText("黑色建材")).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("螺纹钢")).toBeVisible();
  });
});
