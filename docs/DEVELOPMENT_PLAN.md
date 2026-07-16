# 后续功能开发与 UI 重构规划

> 版本：v3.0 · 2026-07-11
> 状态：**CMC/Coinbase 风格 P0+P1 主线已落地，P2 旧入口清理与基础 live 验证已完成，进入 A 股深度数据增强**
> 依据：`docs/CMC_PRODUCT_EXECUTION_PLAN.md`、`docs/CMC_PRODUCT_VISUAL_DESIGN.html`

---

## 1. 当前产品主路径

ThisIsMyQuant 当前主路径已从早期「多入口分析工作台」收敛为：

```text
市场发现 → 标的详情 → 自选 → 基础模拟盘 → 事件资讯 → 本地数据 → 引用式 AI
```

一级导航：

```text
总览 / 市场 / 自选 / 模拟盘 / 事件资讯 / 数据库 / AI / 设置
```

当前产品边界：

- 仅做国内期货与 A 股的研究、模拟交易、本地数据和 AI 解释。
- 不接期货公司或券商实盘柜台，不发送真实委托。
- 模拟盘主界面只保留基础业务：市价/限价、买卖、开平、手数、费用估算、持仓、委托、成交、资金流水。
- 复杂条件单、OCO、移动止损、交易复盘、历史回放训练等进阶能力收敛为设置页实验区或后续规划，不作为主导航入口。

---

## 2. 已完成模块

| 模块 | 路由 | 状态 | 关键文件 |
|---|---|---|---|
| 总览 | `/` | 已完成 | `OverviewPage.tsx`、`get_professional_dashboard` |
| 市场 | `/markets` | 已完成 | `MarketsPage.tsx`、`commands/market.rs` |
| 期货详情 | `/markets/futures/:symbol` | 已完成 | `AssetDetailPage.tsx` |
| A 股详情 | `/markets/stocks/:symbol` | 已完成 | `AssetDetailPage.tsx` |
| 自选 | `/watchlist` | 已完成 | `WatchlistPage.tsx`、`commands/watchlist.rs` |
| 基础模拟盘 | `/simulation` | 已完成 | `SimulationPage.tsx`、`commands/simulation.rs` |
| 事件资讯 | `/events` | 已完成 | `EventsPage.tsx`、`commands/events.rs` |
| 数据库资产中心 | `/database` | 已完成 | `LocalDatabasePage.tsx`、`commands/data.rs` |
| 引用式 AI | `/ai` | 已完成 | `AiPage.tsx`、`commands/ai.rs`、`engine/ai_context.rs` |
| 报告 | `/reports` | 已完成 | `ReportsPage.tsx` |
| 状态 | `/status` | 已完成 | `StatusPage.tsx` |
| 设置 | `/settings` | 已完成 | `SettingsPage.tsx` |

旧入口收敛：

| 旧入口 | 当前处理 |
|---|---|
| `/workspace` | 重定向到 `/markets` |
| `/stocks` | 重定向到 `/markets` |
| `/symbols` | 重定向到 `/markets` |
| `/symbols/:symbol` | 重定向到 `/markets/futures/:symbol` |
| `/news`、`/calendar` | 重定向到 `/events` |
| `/copilot`、`/factors` | 重定向到 `/ai` |
| `/anomalies` | 重定向到 `/events` |
| `/review`、`/replay` | 保留为设置页实验区入口 |

---

## 3. 已完成工程项

### 3.1 信息架构与视觉

- 顶部导航替代复杂侧栏。
- 恢复 macOS 原生标题栏和信号灯。
- 默认主题改为 Coinbase 风格浅色 UI。
- 统一 `PageShell`、`PageHeader`、`GlobalMarketBar`、`DataQualityBadge`。
- 按 `docs/CMC_PRODUCT_VISUAL_DESIGN.html` 落地市场、详情、自选、事件、数据库、AI 页面结构。

### 3.2 市场发现

- 统一 `MarketAsset`、`MarketOverview`、`MarketLeaderboard`、`MarketFilters`。
- 后端新增 `get_market_overview`、`list_market_assets`、`get_market_leaderboard`、`get_asset_sparkline`、`search_assets`。
- 前端新增 `AssetTable`、`AssetIdentityCell`、`PriceChangeCell`、`MiniSparkline`、`MarketFilters`、`MarketLeaderboard`、`WatchButton`。
- 支持期货/A 股筛选、排序、搜索、星标自选和详情跳转。

### 3.3 标的详情

- 统一期货与 A 股详情页。
- 支持资产头部、K 线面板、关键指标、详情 Tabs、右侧自选备注/事件/持仓/AI 快问。
- 旧品种详情路由兼容重定向。

### 3.4 自选

- 新增 `watchlist_groups`、`watchlist_items`。
- 支持分组、备注、提醒、排序和自选摘要。
- 支持从市场列表和详情页加入/移除自选。
- 自选页已接入 AI 自选日报，可基于自选标的和事件上下文生成今日跟踪清单、驱动解释、明日观察点与风险提示。

### 3.5 基础模拟盘

- 主 UI 收敛为基础下单、持仓、委托、成交、资金流水、账户重置。
- 复杂订单能力保留在后端与实验区，不作为主业务入口。
- 页面固定展示“仅模拟，不构成真实交易”边界。
- 模拟盘已接入 AI 复盘摘要，可基于最近 30 天成交、持仓、资金曲线和复盘日记生成纪律、风险与改进动作。

### 3.6 事件资讯

- 合并金十资讯、财经日历、公告、财报和产业事件为统一 `MarketEvent`。
- 支持来源、重要性、标的/板块筛选。
- 支持事件影响标的跳转详情页。
- 支持事件触发 AI 影响分析。

### 3.7 数据库资产中心

- 从数据库统计卡升级为数据域资产中心。
- 支持行情、K 线、资讯、日历、报告、模拟交易、自选、A 股、设置等数据域。
- 支持同步、导出、清理、备份，清理操作要求确认。

### 3.8 引用式 AI

- 新增 `AiContextBundle`。
- AI 输入按行情、资讯、事件、财务、持仓、订单、报告分区。
- AI 输出要求包含 `sources`、`data_date`、`disclaimer`。
- AI 页支持任务选择、摘要生成、来源展示和历史任务。

---

## 4. 测试状态

最近一次验证：

```bash
pnpm --dir frontend tsc                 # 通过
PLAYWRIGHT_PORT=5175 pnpm test:e2e      # 34 passed
cargo test --manifest-path src-tauri/Cargo.toml --lib  # 65 passed
cargo test --manifest-path src-tauri/Cargo.toml --test sim_trading_test  # 13 passed
pnpm test:e2e:client                    # 2 passed
KIMI_API_KEY / MOONSHOT_API_KEY probe   # 403 quota exhausted
```

已验证 live 链路：

- Tauri 客户端健康检查。
- Doubao / MiniMax 真实业务链路可用；最近一次 live E2E 中 provider 列表为 `doubao,minimax`，在线 provider 为 `minimax`。
- Kimi / Moonshot provider 已接入 catalog、`.env` 同步、mock 设置页和真实设置页；当前 `~/global_env` 中 `KIMI_API_KEY` / `MOONSHOT_API_KEY` 对 Kimi Coding API 返回 403 `usage limit`，说明密钥已通过格式/身份校验但账号额度耗尽，等待可用额度后执行 Kimi-only 全业务验证。
- 五大期货板块主力数据：RB0、AU0、M0、SC0、EC0。
- 金十资讯连通。
- 财经日历连通。
- A 股核心资产、3 个指数和 600000.SH 最新行情；东财不可用时降级新浪实时行情。
- 统一标的详情真实数据：期货详情和 600000.SH A 股详情。
- 基础模拟盘限价委托提交与撤单。
- 引用式 AI sources 字段。
- LLM 明日/短期分析生成，报告包含免责声明并入库。

新增待验证链路：

- Kimi-only 全业务链路：LLM health、明日/短期分析、AI sources。前置条件是 Kimi/Moonshot 账号恢复可用额度。
- A 股公告/财报事件已接入统一事件流；当前以本地目录、财务缓存和 fallback 摘要生成研究提醒，后续再增强交易所/巨潮/东财公告源。
- A 股财务与估值已进入个股详情页和因子计算；东财/AKShare 不可用时标记 `estimated+fallback`，避免页面空白。

---

## 5. 当前剩余任务

### P2-1 文档一致性

- [x] `docs/DESIGN.md` 更新到 CMC/Coinbase 信息架构。
- [x] `docs/REQUIREMENTS.md` 更新页面矩阵和范围边界。
- [x] `docs/CMC_PRODUCT_EXECUTION_PLAN.md` 记录执行清单和验证状态。
- [x] `docs/DEVELOPMENT_PLAN.md` 重写为当前路线。
- [x] `docs/ARCHITECTURE.md` 更新前端页面树和后端 command。
- [ ] `docs/USAGE.md` 补充新导航和自选/事件/AI 使用说明。
- [ ] `docs/SIMULATION_TRADING_DESIGN.md` 明确主 UI 基础模拟盘优先，复杂订单作为实验区。

### P2-2 旧代码清理

- [x] 删除旧侧栏组件 `SidebarNavItem`。
- [x] 删除旧资讯页 `NewsDecisionPage`。
- [x] 删除旧日历页 `MacroCalendarPage`。
- [x] 删除旧 Copilot 页 `CopilotPage`。
- [x] 删除旧因子页 `FactorCenterPage`。
- [x] 删除旧异动页 `AnomalyCenterPage`。
- [ ] 评估 `TradingReviewPage` 与 `MarketReplayPage` 是否继续保留为实验区。
- [x] 删除旧 `DashboardPage`、`SymbolsPage`、`SymbolDetailPage`、`AStockPage` 页面入口。
- [x] 清理旧 A 股页面 `AStockPage` 重复入口。
- [x] 删除旧 `features/news`、`features/calendar`、`features/symbol`、`features/stocks` 组件目录，后续 A 股增强接入统一市场和详情组件。

### P2-3 新模块增强

- [x] `/markets` 接入真实 A 股核心资产与指数，东财失败时降级到新浪实时行情。
- [ ] `/markets` 增加 ETF 过滤。
- [x] 详情页补真实 A 股行情 live 校验；财务/估值摘要进入详情页，缺源时显示 `estimated+fallback`。
- [x] 自选提醒配置在自选表格与详情侧栏持久化。
- [x] 事件资讯接入 A 股公告/财报研究事件；真实公告源作为后续增强。
- [x] 引用式 AI 的 `sources` 字段在 AI 卡片/摘要中展示，支持跳转行情、资讯/事件、持仓和报告。
- [x] 模拟盘增加基础绩效摘要，并在期货 K 线详情展示模拟成交标记。
- [x] 自选页增加 AI 自选日报入口，并纳入 mock E2E 闭环。
- [x] 模拟盘增加 AI 复盘摘要入口，并纳入 mock E2E 闭环。
- [x] 数据库备份改为 SQLite 一致性备份，并支持备份文件校验与恢复候选生成。

---

## 6. 下一阶段建议

### A. 真实 A 股链路

基础链路已通过 live E2E，当前 A 股已从“行情可用”推进到“基础研究可用”：

- 财务摘要、估值和报告期进入个股详情。
- 行业/概念与成分股具备东财优先、fallback 兜底。
- 公告、财报日历进入统一事件流。
- 下一步重点是把公告源升级为交易所/巨潮/东财真实公告，并补资金流与同行对比。

### B. 自选作为默认工作流

自选是用户高频入口，建议继续增强：

- 价格提醒。
- 事件提醒。
- 自选分组排序。
- 自选导入导出。
- 自选 AI 日报已完成，后续可补充定时生成、日报历史和一键加入明日观察清单。

### C. 数据库中心稳定化

本地数据资产中心是产品差异化能力，建议补齐：

- 数据域分页。
- 导出任务进度。
- 清理前影响预估。
- 备份恢复候选校验与重启恢复流程。
- 数据源错误历史。

### D. 引用式 AI 可信度

AI 不做泛聊天，继续加强可追溯：

- 所有报告展示来源列表。
- 支持点击来源跳回行情、资讯/事件、持仓、报告和 A 股财务页。
- live E2E 断言来源字段。
- 失败时保存上下文和重试入口。

---

## 7. Definition of Done

本轮 CMC 重构全部完成需要满足：

- [x] 一级导航与 CMC 执行计划一致。
- [x] CMC 风格视觉设计落地到真实页面。
- [x] 市场、详情、自选、基础模拟盘、事件资讯、数据库、AI 均有可访问页面。
- [x] 模拟盘保持基础业务边界。
- [x] Mock E2E 覆盖核心页面。
- [x] 真实期货与 LLM client live E2E 通过。
- [x] 真实 A 股 live E2E 通过。
- [x] 旧页面入口和旧 feature 组件清理。
- [x] 文档、测试、代码三者对齐到当前 1~6 阶段。
