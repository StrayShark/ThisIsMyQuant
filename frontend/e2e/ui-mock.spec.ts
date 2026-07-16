import { test, expect } from "@playwright/test";

const DEFAULT_TIMEOUT = 15000;

test.describe("应用壳与导航", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("link", { name: "总览" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("启动后默认进入总览页", async ({ page }) => {
    await expect(page).toHaveURL(/\/(?:#\/)?$/);
    await expect(page.getByText("专业分析工作台")).toBeVisible();
  });

  test("新主导航可见且旧入口已移除", async ({ page }) => {
    const mainNav = page.locator('[aria-label="主导航"]');
    for (const label of ["总览", "市场", "自选", "模拟盘", "事件资讯", "数据库", "AI", "设置"]) {
      await expect(mainNav.getByRole("link", { name: label })).toBeVisible();
    }
    await expect(mainNav.getByRole("link", { name: "期货" })).toHaveCount(0);
    await expect(mainNav.getByRole("link", { name: "A股" })).toHaveCount(0);
  });

  test("顶部系统入口可见", async ({ page }) => {
    await expect(
      page.locator('[aria-label="系统导航"]').getByRole("link", { name: "状态" })
    ).toBeVisible();
  });
});

test.describe("总览页", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/");
    await expect(page.getByRole("link", { name: "总览" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("展示板块热力与资讯摘要", async ({ page }) => {
    await expect(page.getByText("黑色建材").first()).toBeVisible();
    await expect(page.getByText("能源化工").first()).toBeVisible();
    await expect(page.getByText("航运运价").first()).toBeVisible();
    await expect(page.getByText("专业分析工作台")).toBeVisible();
    await expect(page.getByText("热力图")).toBeVisible();
    await expect(page.getByText("LLM 多空建议")).toBeVisible();
    await expect(page.getByText("资讯决策流")).toBeVisible();
    await expect(page.getByText("产业因子")).toBeVisible();
    await expect(page.getByText("异动预警")).toBeVisible();
    await expect(page.getByText("报告流程")).toBeVisible();
    await expect(page.getByText("外盘联动")).toBeVisible();
    await expect(page.getByText("重要数据时间窗")).toBeVisible();
    await expect(page.getByText("资讯 · 宏观摘要")).toBeVisible();
    await expect(page.getByText("短期偏多，关注突破前高")).toBeVisible({ timeout: 10000 });
  });
});

test.describe("市场页", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/markets");
    await expect(page.getByRole("heading", { name: "市场" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("展示标题与标的表格", async ({ page }) => {
    await expect(page.getByText("统一期货与 A 股的发现入口")).toBeVisible();
    await expect(page.getByRole("row", { name: /螺纹钢/ }).first()).toBeVisible();
    await expect(page.getByRole("row", { name: /黄金/ }).first()).toBeVisible();
    await expect(page.getByRole("row", { name: /浦发银行/ }).first()).toBeVisible();
  });

  test("Tab 全部/期货/A股/自选 可切换", async ({ page }) => {
    const tabs = page.locator(".mb-5").getByRole("tablist");
    for (const tab of ["全部", "期货", "A股", "自选"]) {
      await tabs.getByRole("tab", { name: tab, exact: true }).click();
      await expect(tabs.getByRole("tab", { name: tab, exact: true })).toHaveAttribute("data-state", "active");
    }
  });

  test("表格支持排序", async ({ page }) => {
    await expect(page.getByRole("row", { name: /螺纹钢|黄金|浦发银行/ }).first()).toBeVisible();
    const header = page.getByRole("columnheader", { name: /最新价/ });
    await header.click();
    await expect(page.getByRole("row", { name: /螺纹钢|黄金|浦发银行/ }).first()).toBeVisible();
    await header.click();
    await expect(page.getByRole("row", { name: /螺纹钢|黄金|浦发银行/ }).first()).toBeVisible();
  });

  test("表格支持筛选", async ({ page }) => {
    await expect(page.getByRole("row", { name: /螺纹钢/ }).first()).toBeVisible();

    await page.locator("select", { hasText: "全部板块" }).first().selectOption("黑色建材");
    await expect(page.getByRole("row", { name: /螺纹钢/ }).first()).toBeVisible();
    await expect(page.getByRole("row", { name: /原油/ })).toHaveCount(0);

    await page.locator("select", { hasText: "全部状态" }).first().selectOption("live");
    await expect(page.getByRole("row", { name: /螺纹钢/ }).first()).toBeVisible();
  });

  test("点击期货标的进入期货详情", async ({ page }) => {
    await page.getByRole("row", { name: /螺纹钢/ }).first().click();
    await page.waitForURL(/\/#\/markets\/futures\/rb0/i);
  });

  test("点击 A 股标的进入 A 股详情", async ({ page }) => {
    await page.getByRole("row", { name: /浦发银行/ }).first().click();
    await page.waitForURL(/\/#\/markets\/stocks\/600000\.SH/i);
  });

  test("表格行内可以加自选", async ({ page }) => {
    const row = page.getByRole("row", { name: /黄金/ }).first();
    await row.getByRole("button", { name: "加入自选" }).click();
    await expect(page.getByText("已将 黄金 加入自选")).toBeVisible();
  });

  test("市场页有 AI 市场摘要按钮", async ({ page }) => {
    await expect(page.getByRole("button", { name: "AI 市场摘要" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });
});

test.describe("详情页", () => {
  test("期货详情页展示关键信息与操作", async ({ page }) => {
    await page.goto("/#/markets/futures/rb0");
    await expect(page.getByRole("heading", { name: /螺纹钢|RB0/ })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
    await expect(page.getByRole("button", { name: /(加入|移出)自选/ })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
    await expect(page.getByRole("button", { name: "模拟下单" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
    await expect(page.getByText("概览")).toBeVisible();
    await page.getByRole("tab", { name: /行情/ }).click();
    await expect(page.getByText(/买开 2 @ 3200\.00/)).toBeVisible({ timeout: 10000 });
  });

  test("A 股详情页展示关键信息与操作", async ({ page }) => {
    await page.goto("/#/markets/stocks/600000.SH");
    await expect(page.getByRole("heading", { name: /浦发银行|600000\.SH/ })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
    await expect(page.getByRole("button", { name: /(加入|移出)自选/ })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
    await expect(page.getByRole("button", { name: "模拟下单" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
    await expect(page.getByText("财务与估值")).toBeVisible({ timeout: DEFAULT_TIMEOUT });
    await expect(page.getByText("PE TTM")).toBeVisible();
    await expect(page.getByText("财务来源")).toBeVisible();
  });
});

test.describe("自选页", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/watchlist");
    await expect(page.getByRole("heading", { name: "自选" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("展示分组 Tabs", async ({ page }) => {
    await expect(page.getByRole("button", { name: "全部" }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: "默认自选", exact: true })).toBeVisible();
  });

  test("从市场页加入自选后，在自选页可见", async ({ page }) => {
    await page.goto("/#/markets");
    const row = page.getByRole("row", { name: /黄金/ }).first();
    await row.getByRole("button", { name: "加入自选" }).click();
    await expect(page.getByText("已将 黄金 加入自选")).toBeVisible();

    await page.goto("/#/watchlist");
    await expect(page.getByRole("row", { name: /黄金/ }).first()).toBeVisible();
  });

  test("可生成 AI 自选日报", async ({ page }) => {
    await page.getByRole("button", { name: "生成今日自选日报" }).click();
    await expect(page.getByText("E2E 模拟 AI 摘要")).toBeVisible({ timeout: 10000 });
    await expect(page.getByText("仅供研究与复盘，不构成投资建议").first()).toBeVisible();
  });
});

test.describe("模拟盘", () => {
  test("支持基础市价/限价委托与模拟标识", async ({ page }) => {
    await page.goto("/#/simulation");
    await expect(page.getByRole("heading", { name: "模拟盘" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });

    await expect(page.getByText("仅模拟")).toBeVisible();
    await expect(page.getByText("基础虚拟账户、基础下单、持仓和成交记录")).toBeVisible();
    await expect(page.getByText("基础下单", { exact: true })).toBeVisible();
    await expect(page.getByText("持仓", { exact: true })).toBeVisible();
    await expect(page.getByText("委托", { exact: true })).toBeVisible();
    await expect(page.getByText("成交", { exact: true })).toBeVisible();
    await expect(page.getByText("资金流水", { exact: true })).toBeVisible();
    await expect(page.getByText("累计收益率")).toBeVisible();
    await expect(page.getByText("最大回撤")).toBeVisible();
    await expect(page.getByText("品种贡献")).toBeVisible();
    await expect(page.getByText("AI 复盘摘要")).toBeVisible();
    await page.getByRole("button", { name: "生成复盘摘要" }).click();
    await expect(page.getByText("AI 复盘摘要已生成")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("E2E 模拟分析报告").first()).toBeVisible({ timeout: 5000 });
    await expect(
      page.getByText("本页面所有交易均为模拟，不构成投资建议，不连接实盘柜台。")
    ).toBeVisible();
    await expect(page.getByText("不连接实盘柜台").first()).toBeVisible();

    const typeSelect = page.getByText("类型").locator("..").locator("select");
    await typeSelect.selectOption("market");
    await typeSelect.selectOption("limit");

    await expect(page.getByText("预估保证金")).toBeVisible();
    await expect(page.getByText("预估总成本")).toBeVisible();

    await page.getByRole("button", { name: "提交模拟委托" }).click();
    await expect(page.getByText("模拟委托已提交")).toBeVisible();
  });
});

test.describe("事件资讯页", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/events");
    await expect(page.getByRole("heading", { name: "事件资讯" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("可访问并展示标题", async ({ page }) => {
    await expect(page.getByText("聚合财经日历、公告与市场事件")).toBeVisible();
  });

  test("可按来源筛选金十资讯", async ({ page }) => {
    await page.getByRole("button", { name: "金十", exact: true }).click();
    await expect(page.getByText("来源：金十").first()).toBeVisible({ timeout: 5000 });
  });

  test("可按来源筛选财经日历", async ({ page }) => {
    await page.getByRole("button", { name: "日历", exact: true }).click();
    await expect(page.getByText("来源：日历").first()).toBeVisible({ timeout: 5000 });
  });

  test("点击事件影响标的跳转到详情页", async ({ page }) => {
    await page.getByText("RB0", { exact: true }).first().click();
    await page.waitForURL(/\/#\/markets\/futures\/rb0/i);
  });
});

test.describe("数据库资产中心", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/database");
    await expect(page.getByRole("heading", { name: "本地数据库" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("展示数据域列表", async ({ page }) => {
    const domainNames = ["行情报价", "K线数据", "资讯", "财经日历", "研报", "模拟交易", "自选", "A股", "设置"];
    let visibleCount = 0;
    for (const name of domainNames) {
      if (await page.getByText(name).first().isVisible().catch(() => false)) {
        visibleCount += 1;
      }
    }
    expect(visibleCount).toBeGreaterThanOrEqual(5);
  });

  test("可执行全库备份", async ({ page }) => {
    await page.getByRole("button", { name: "立即备份" }).click();
    await expect(page.getByText(/已备份/)).toBeVisible({ timeout: 5000 });
  });

  test("可校验备份并生成恢复候选", async ({ page }) => {
    await page.getByPlaceholder(/粘贴 .*备份文件路径/).fill("data/quant.db.bak");
    await page.getByRole("button", { name: "准备恢复" }).click();
    await expect(page.getByText(/恢复候选/)).toBeVisible({ timeout: 5000 });
  });
});

test.describe("AI 分析页", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/ai");
    await expect(page.getByRole("heading", { name: "AI 分析" })).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("可访问并展示标题", async ({ page }) => {
    await expect(page.getByText("基于本地数据生成可解释的研究摘要")).toBeVisible();
  });

  test("可生成 AI 摘要并展示免责声明", async ({ page }) => {
    await page.getByRole("button", { name: "市场摘要", exact: true }).click();
    await page.getByRole("button", { name: "生成摘要" }).click();
    await expect(page.getByText("E2E 模拟 AI 摘要")).toBeVisible({ timeout: 10000 });
    await expect(page.getByText("仅供研究与复盘，不构成投资建议").first()).toBeVisible();
    await page.getByText("E2E 模拟资讯源").click();
    await page.waitForURL(/\/#\/events$/);
  });
});

test.describe("旧路由重定向", () => {
  test("/workspace 重定向到 /markets", async ({ page }) => {
    await page.goto("/#/workspace");
    await page.waitForURL(/\/#\/markets$/);
  });

  test("/symbols 重定向到 /markets", async ({ page }) => {
    await page.goto("/#/symbols");
    await page.waitForURL(/\/#\/markets$/);
  });

  test("/stocks 重定向到 /markets", async ({ page }) => {
    await page.goto("/#/stocks");
    await page.waitForURL(/\/#\/markets$/);
  });

  test("/news 与 /calendar 重定向到 /events", async ({ page }) => {
    await page.goto("/#/news");
    await page.waitForURL(/\/#\/events$/);
    await page.goto("/#/calendar");
    await page.waitForURL(/\/#\/events$/);
  });

  test("/factors 与 /anomalies 重定向到新模块", async ({ page }) => {
    await page.goto("/#/factors");
    await page.waitForURL(/\/#\/ai$/);
    await page.goto("/#/anomalies");
    await page.waitForURL(/\/#\/events$/);
  });
});

test.describe("报告与设置", () => {
  test("报告页展示 Mock 报告卡片", async ({ page }) => {
    await page.goto("/#/reports");
    await expect(page.getByText("短期偏多，关注突破前高").first()).toBeVisible({
      timeout: DEFAULT_TIMEOUT,
    });
  });

  test("设置页展示数据源、大模型与模拟规则", async ({ page }) => {
    await page.goto("/#/settings?section=data");
    await expect(page.getByText("AKShare K 线").first()).toBeVisible({ timeout: DEFAULT_TIMEOUT });

    await page.goto("/#/settings?section=llm");
    await expect(page.getByRole("link", { name: "配置 LLM API Key" })).toBeVisible();

    await page.goto("/#/settings?section=storage");
    await expect(page.getByText("重载 .env（金十等）")).toBeVisible();

    await page.goto("/#/settings?section=simulation");
    await expect(page.getByText("合约规则")).toBeVisible();
    await expect(page.getByText("风控规则").first()).toBeVisible();
    await expect(page.getByText("RB0")).toBeVisible();
  });
});
