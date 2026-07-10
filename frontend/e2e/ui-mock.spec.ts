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

    await page.locator("aside nav").getByRole("link", { name: "模拟盘" }).click();
    await expect(page.getByRole("heading", { name: "模拟盘" })).toBeVisible();
    await expect(page.getByText("虚拟资金练习国内期货下单与持仓管理")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "复盘" }).click();
    await expect(page.getByRole("heading", { name: "交易复盘" })).toBeVisible();
    await expect(page.getByText("记录交易计划、执行评分与绩效归因")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "回放" }).click();
    await expect(page.getByRole("heading", { name: "回放训练" })).toBeVisible();
    await expect(page.getByText("按历史行情练习下单，不显示未来数据")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "因子" }).click();
    await expect(page.getByText("因子中心")).toBeVisible();
    await expect(page.getByText("待接入产业专源")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "资讯" }).click();
    await expect(page.getByText("资讯决策中心")).toBeVisible();
    await expect(page.getByText("螺纹钢需求偏弱")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "日历" }).click();
    await expect(page.getByText("日历与宏观")).toBeVisible();
    await expect(page.getByText("美国CPI月率")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "异动" }).click();
    await expect(page.getByText("异动预警中心")).toBeVisible();
    await expect(page.getByText("雷达信号")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "助手" }).click();
    await expect(page.getByText("Copilot 研究助手")).toBeVisible();
    await expect(page.getByText("当前上下文")).toBeVisible();

    await page.getByRole("link", { name: "报告" }).click();
    await expect(page.getByText("短期偏多，关注突破前高").first()).toBeVisible({
      timeout: 10000,
    });

    await page.getByRole("link", { name: "品种" }).click();
    await expect(page.getByText("黑色建材")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "数据库" }).click();
    await expect(page.getByRole("heading", { name: "本地数据库" })).toBeVisible();
    await expect(page.getByText("查看和管理本地 SQLite 数据资产")).toBeVisible();

    await page.locator("aside nav").getByRole("link", { name: "品种" }).click();
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

  test("设置页展示数据源、大模型与模拟规则", async ({ page }) => {
    await page.goto("/#/settings?section=data");
    await expect(page.getByText("AKShare K 线").first()).toBeVisible({ timeout: 15000 });
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

test.describe("模拟盘", () => {
  test("支持高级订单类型与费用估算", async ({ page }) => {
    await page.goto("/#/simulation");
    await expect(page.getByRole("heading", { name: "模拟盘" })).toBeVisible({ timeout: 15000 });

    // 切换订单类型为条件单
    await page.locator("form").getByText("类型").locator("..").locator("select").selectOption("condition");
    await expect(page.getByText("触发价")).toBeVisible();
    await page.locator("form").getByText("触发价").locator("..").locator("input").fill("3150");

    // 填写止损止盈并提交
    await page.locator("form").getByText("止损价").locator("..").locator("input").fill("3100");
    await page.locator("form").getByText("止盈价").locator("..").locator("input").fill("3300");

    await page.getByRole("button", { name: "提交模拟委托" }).click();
    await expect(page.getByText("模拟委托已提交")).toBeVisible();

    // 费用估算展示
    await expect(page.getByText("预估保证金")).toBeVisible();
    await expect(page.getByText("预估总成本")).toBeVisible();

    // 委托列表中出现条件单
    await expect(page.getByRole("tab", { name: "委托" })).toBeVisible();
    await page.getByRole("tab", { name: "委托" }).click();
    await expect(page.getByRole("cell", { name: /条件单.*3150/ }).first()).toBeVisible();
  });
});

test.describe("回放训练", () => {
  test("可启动、步进并查看回放图表", async ({ page }) => {
    await page.goto("/#/replay");
    await expect(page.getByRole("heading", { name: "回放训练" })).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("按历史行情练习下单，不显示未来数据")).toBeVisible();

    // 开始回放
    await page.getByRole("button", { name: "开始" }).click();
    await expect(page.getByText("运行中").first()).toBeVisible();
    await expect(page.getByText("RB0")).toBeVisible();

    // 步进后当前价变化
    await page.getByRole("button", { name: "步进" }).click();
    await expect(page.getByText("3202.00")).toBeVisible();

    // 图表区域存在
    await expect(page.locator('[class*="rounded-md"][class*="border"]').nth(1)).toBeVisible();

    // 下单面板可用
    await expect(page.getByRole("button", { name: "提交模拟委托" })).toBeVisible();
  });
});

test.describe("品种页", () => {
  test("展示品种板块列表", async ({ page }) => {
    await page.goto("/#/symbols");
    await expect(page.getByText("黑色建材")).toBeVisible({ timeout: 15000 });
    await expect(page.getByText("螺纹钢")).toBeVisible();
  });
});


test.describe("A 股市场分析", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/stocks");
    await expect(page.getByRole("link", { name: "A股" })).toBeVisible({ timeout: 15000 });
  });

  test("A 股总览可访问并展示指数与市场宽度", async ({ page }) => {
    await expect(page.getByText("上证指数")).toBeVisible();
    await expect(page.getByText("深证成指")).toBeVisible();
    await expect(page.getByText("创业板指")).toBeVisible();
    await expect(page.getByText("科创50")).toBeVisible();
    await expect(page.getByText("市场宽度")).toBeVisible();
    await expect(page.getByText("上涨")).toBeVisible();
    await expect(page.getByText("行业/概念热力")).toBeVisible();
  });

  test("点击行业热力图下钻到行业详情", async ({ page }) => {
    await expect(page.getByText("银行").first()).toBeVisible();
    await page.getByText("银行").first().click();
    await expect(page.getByRole("button", { name: "返回" })).toBeVisible();
    await expect(page.getByText("成分股")).toBeVisible();
    await expect(page.getByText("浦发银行")).toBeVisible();
  });
});


test.describe("A 股个股与筛选器", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/stocks?tab=screener");
    await expect(page.getByRole("link", { name: "A股" })).toBeVisible({ timeout: 15000 });
  });

  test("筛选器页面可运行筛选并下钻到个股", async ({ page }) => {
    await expect(page.getByText("筛选条件")).toBeVisible();
    await expect(page.getByRole("heading", { name: "筛选结果" })).toBeVisible();
    await page.getByRole("button", { name: "运行筛选" }).click();
    await expect(page.getByText("浦发银行")).toBeVisible();
    await page.getByText("浦发银行").click();
    await expect(page.getByText("K 线走势")).toBeVisible();
    await expect(page.getByText("财务摘要")).toBeVisible();
  });
});

test.describe("A 股模拟组合", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/stocks?tab=portfolio");
    await expect(page.getByRole("link", { name: "A股" })).toBeVisible({ timeout: 15000 });
  });

  test("模拟组合页展示账户与持仓成交", async ({ page }) => {
    await expect(page.getByText("账户管理")).toBeVisible();
    await expect(page.getByText("A股模拟组合")).toBeVisible();
    await expect(page.getByText("总资产")).toBeVisible();
    await expect(page.getByRole("heading", { name: "持仓" })).toBeVisible();
    await expect(page.getByText("浦发银行")).toBeVisible();
    await expect(page.getByText("近期成交")).toBeVisible();
    await expect(page.getByText("买入").first()).toBeVisible();
  });

  test("可填写模拟下单并看到费用估算", async ({ page }) => {
    const ticket = page.locator("div").filter({ hasText: "模拟下单" }).first();
    await expect(ticket.getByText("模拟下单")).toBeVisible();
    await ticket.getByPlaceholder("600000.SH").fill("600000.SH");
    await ticket.getByPlaceholder("限价").fill("10.00");
    await ticket.locator('input[type="number"]').nth(1).fill("200");
    await expect(ticket.getByText("成交金额")).toBeVisible();
    await expect(ticket.getByText("总成本")).toBeVisible();
    await expect(ticket.getByRole("button", { name: "模拟买入" })).toBeEnabled();
  });
});


test.describe("A 股体验补全", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/#/stocks?tab=screener");
    await expect(page.getByRole("link", { name: "A股" })).toBeVisible({ timeout: 15000 });
  });

  test("个股页展示 K 线与复权切换", async ({ page }) => {
    await page.goto("/#/stocks?tab=stock&symbol=600000.SH");
    await expect(page.getByText("K 线走势")).toBeVisible();
    await expect(page.getByText("前复权")).toBeVisible();
    await expect(page.getByText("AI 研究速览")).toBeVisible();
  });

  test("财报 Tab 展示财务指标", async ({ page }) => {
    await page.goto("/#/stocks?tab=financials&symbol=600000.SH");
    await expect(page.getByText("营业收入").first()).toBeVisible();
    await expect(page.getByText("ROE").first()).toBeVisible();
    await expect(page.getByText("2025-12-31").first()).toBeVisible();
  });

  test("可将个股加入自选并在自选页查看", async ({ page }) => {
    await page.goto("/#/stocks?tab=stock&symbol=600000.SH");
    await expect(page.getByRole("button", { name: "加入自选" })).toBeVisible();
    await page.getByRole("button", { name: "加入自选" }).click();
    await page.goto("/#/stocks?tab=watchlist");
    await expect(page.getByText("我的自选股")).toBeVisible();
    await expect(page.getByText("600000.SH")).toBeVisible();
  });

  test("筛选器可加载已保存模板", async ({ page }) => {
    await expect(page.getByText("已保存模板")).toBeVisible();
    await page.getByText("E2E 低估值模板").click();
    await expect(page.getByRole("heading", { name: "筛选结果" })).toBeVisible();
    await expect(page.getByText("浦发银行")).toBeVisible();
  });

  test("筛选器可生成 AI 总结", async ({ page }) => {
    await page.getByRole("button", { name: "运行筛选" }).click();
    await expect(page.getByText("AI 总结")).toBeVisible();
    await page.getByRole("button", { name: "AI 总结" }).click();
    await expect(page.getByText("AI 筛选总结")).toBeVisible();
  });
});
