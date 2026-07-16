# CMC 风格产品重构执行计划

> 版本：v0.1 · 2026-07-10  
> 状态：P0+P1 已完成，P2 清理中  
> 设计依据：`docs/CMC_PRODUCT_VISUAL_DESIGN.html`  
> 产品方向：国内期货与 A 股市场发现 + 自选 + 基础模拟盘 + 事件资讯 + 本地数据库 + 引用式 AI  
> 约束：不做实盘交易、不连接柜台；模拟盘先做基础业务，不暴露日内训练、回放、复杂条件单、OCO、移动止损等入口。

---

## 1. 重构目标

本轮重构参考 CoinMarketCap 的全站产品结构，但不照搬加密货币业务。核心目标是把当前分散的页面矩阵收敛为一条更清晰的用户路径：

```text
市场发现 → 标的详情 → 加入自选 → 基础模拟盘 → 事件资讯 → 本地数据 → 引用式 AI
```

一级导航目标：

```text
总览 / 市场 / 自选 / 模拟盘 / 事件资讯 / 数据库 / AI / 设置
```

旧功能收敛原则：

| 旧入口 | 新归属 | 处理方式 |
|---|---|---|
| 行情 | 市场 | 合并为 `/markets` 下的期货/A 股列表和详情入口。 |
| 品种 | 市场 | 合并为期货列表、板块和详情页。 |
| 因子 | 标的详情 / 事件资讯 / AI | 不再作为一级入口。 |
| 资讯 | 事件资讯 | 合并金十资讯、公告、财报和产业事件。 |
| 日历 | 事件资讯 | 作为事件流 Tab。 |
| 异动 | 市场排行榜 / 自选 | 作为发现榜单和自选异动。 |
| 助手 / Copilot | AI | 改为引用式 AI 分析与报告归档。 |
| 复盘 | 模拟盘 / AI | 基础阶段不做一级入口。 |
| 回放 | 实验功能 | 从主 UI 隐藏，后续再决定是否保留。 |

---

## 2. 阶段总览

| 阶段 | 名称 | 优先级 | 状态 | 目标结果 |
|---|---|---|---|---|
| P0-1 | 信息架构与壳层统一 | P0 | ✅ 已完成 | 顶部导航、路由、视觉 token 与页面容器统一。 |
| P0-2 | 市场发现与统一表格 | P0 | ✅ 已完成 | `/markets` 支持期货/A 股、排行榜、筛选、自选入口。 |
| P0-3 | 标的详情页重构 | P0 | ✅ 已完成 | 期货/A 股详情采用 CMC asset detail 结构。 |
| P0-4 | 自选模块 | P0 | ✅ 已完成 | 新增 `/watchlist`，成为高频入口。 |
| P0-5 | 基础模拟盘收敛 | P0 | ✅ 已完成 | 仅保留基础下单、持仓、委托、成交、资金摘要。 |
| P1-1 | 事件资讯中心 | P1 | ✅ 已完成 | 合并资讯、日历、公告、财报事件。 |
| P1-2 | 数据库中心升级 | P1 | ✅ 已完成 | 本地数据资产、同步状态、备份导出统一。 |
| P1-3 | 引用式 AI | P1 | ✅ 已完成 | AI 输出必须展示数据来源、日期、免责声明。 |
| P2 | 清理旧入口与文档对齐 | P2 | 🔄 进行中 | 删除/隐藏重复页面，更新 E2E 与产品文档。 |

---

## 3. P0-1 信息架构与壳层统一

### 3.1 导航与路由

- [x] 恢复 macOS 原生标题栏和信号灯样式。
- [x] 使用顶部导航替代复杂侧栏。
- [x] 默认视觉改为 Coinbase 风格浅色主题。
- [x] 新增 `/markets` 路由作为统一市场入口。
- [x] 新增 `/watchlist` 路由作为自选入口。
- [x] 新增 `/events` 路由作为事件资讯入口。
- [x] 新增 `/ai` 路由作为引用式 AI 入口。
- [x] 将主导航调整为：`总览 / 市场 / 自选 / 模拟盘 / 事件资讯 / 数据库 / AI / 设置`。
- [x] 将 `/workspace` 重定向或迁移到 `/markets/futures`。
- [x] 将 `/symbols` 重定向或迁移到 `/markets/futures`。
- [x] 将 `/news`、`/calendar` 重定向或迁移到 `/events`。
- [x] 将 `/copilot` 重定向或迁移到 `/ai`。
- [x] 从主导航隐藏 `/review`、`/replay`、`/factors`、`/anomalies`。

### 3.2 视觉与布局基础

- [x] 统一主色为 Coinbase Blue：`#0052ff`。
- [x] 统一页面背景为浅灰，卡片为白底轻边框。
- [x] 按 Coinbase 风格调整按钮、输入框、卡片圆角。
- [x] 抽象页面容器组件 `PageShell`。
- [x] 抽象页面标题组件 `PageHeader`。
- [x] 抽象顶部状态条 `GlobalMarketBar`。
- [x] 抽象数据状态组件 `DataQualityBadge`。
- [x] 全站移除大面积深色终端风 UI 默认样式。
- [x] 保留深色主题为可选项时，必须与 Coinbase 风格一致，不再使用 Matrix/Cursor 风格作为产品默认。

### 3.3 验收标准

- [x] 一级导航不超过 8 个入口。
- [x] 任何一级页面首屏不出现说明文档式大段文字。
- [x] 所有页面可在 1200px 宽度下无横向溢出。
- [x] `pnpm --dir frontend tsc` 通过。
- [x] `pnpm test:e2e` 覆盖新导航入口。

---

## 4. P0-2 市场发现与统一表格

### 4.1 数据与类型

- [x] 设计统一标的类型 `MarketAsset`。
- [x] 统一期货主力与 A 股资产字段映射。
- [x] 增加资产分类字段：`market`、`sector`、`industry`、`category`。
- [x] 增加数据质量字段：`quality`、`source`、`updated_at`。
- [x] 增加迷你走势图数据结构 `sparkline`。
- [x] 增加排行榜类型：`gainers`、`losers`、`turnover`、`volume_spike`、`watchlist_moves`、`event_related`。

### 4.2 后端 Command

- [x] 新增 `get_market_overview`。
- [x] 新增 `list_market_assets`。
- [x] 新增 `get_market_leaderboard`。
- [x] 新增 `get_asset_sparkline`。
- [x] 新增 `search_assets`。
- [x] 期货数据优先复用 AKShare/Sina 主力连续。
- [x] A 股数据优先复用现有 `commands/stock.rs` 与 stock 表。
- [x] 对缺数据资产返回 `stale` 或 `pending`，不得用 0 伪装。

### 4.3 前端页面

- [x] 新建 `frontend/src/pages/MarketsPage.tsx`。
- [x] 新建 `frontend/src/features/markets/AssetTable.tsx`。
- [x] 新建 `frontend/src/features/markets/MarketLeaderboard.tsx`。
- [x] 新建 `frontend/src/features/markets/MarketFilters.tsx`。
- [x] 新建 `frontend/src/features/markets/AssetIdentityCell.tsx`。
- [x] 新建 `frontend/src/features/markets/MiniSparkline.tsx`。
- [x] 支持 Tab：`全部 / 期货 / A股 / 自选`。
- [x] 支持筛选：板块、行业、数据状态、自选、成交额。
- [x] 支持排序：涨跌幅、成交额、成交量、更新时间。
- [x] 表格行点击进入标的详情。
- [x] 表格支持星标加入自选。

### 4.4 UI 要求

- [x] 表格列与 CMC 类似：资产、价格、涨跌、成交额、分类、迷你走势、状态。
- [x] 数字使用等宽字体。
- [x] 涨跌颜色只用于价格语义。
- [x] 数据状态必须显示文字 badge。
- [x] 移动/窄屏允许表格横向滚动，但页面本身不溢出。

### 4.5 验收标准

- [x] Mock E2E 覆盖 `/markets` 可访问。
- [x] Mock E2E 覆盖筛选和排序。
- [x] Mock E2E 覆盖点击资产进入详情页。
- [x] Mock E2E 覆盖加入自选。
- [x] Live E2E 至少覆盖 5 个期货板块资产和 3 个 A 股指数/股票资产。

---

## 5. P0-3 标的详情页重构

### 5.1 路由设计

- [x] 期货详情使用 `/markets/futures/:symbol`。
- [x] A 股详情使用 `/markets/stocks/:symbol`。
- [x] 旧 `/symbols/:symbol` 保留重定向。
- [x] 旧 `/stocks?tab=stock&symbol=...` 保留重定向或兼容。

### 5.2 详情页结构

- [x] 实现 `AssetDetailPage` 页面骨架。
- [x] 实现 `AssetHeader`：名称、代码、分类、价格、涨跌、更新时间、自选、模拟下单。
- [x] 实现 `KlinePanel`：K 线、成交量、周期切换、数据状态。
- [x] 实现 `AssetStatsGrid`：关键指标。
- [x] 实现 `AssetDetailTabs`。
- [x] 实现右侧栏：持仓摘要、自选备注、提醒、相关事件、AI 快问。

### 5.3 期货详情字段

- [x] 中文名。
- [x] 主力代码。
- [x] 所属五大板块。
- [x] 最新价与涨跌幅。
- [x] 成交量。
- [x] 持仓量。
- [x] 合约乘数。
- [x] 最小变动价位。
- [x] 保证金率。
- [x] 手续费估算。
- [x] 交易所。
- [x] 相关品种。
- [x] 关联金十资讯与日历事件。
- [x] 当前模拟持仓。

### 5.4 A 股详情字段

- [x] 股票名与代码。
- [x] 行业/概念。
- [x] 最新价与涨跌幅。
- [x] 成交额。
- [x] 换手率。
- [x] 市值。
- [x] PE/PB。
- [x] ROE。
- [x] 营收同比。
- [x] 净利同比。
- [x] 最新报告期。
- [x] 复权口径。
- [x] 公告新闻。
- [x] 同行业对比。
- [x] A 股模拟组合持仓。

### 5.5 Tabs

- [x] `概览`：价格、关键指标、AI 摘要。
- [x] `行情`：图表、历史数据、技术指标。
- [x] `资讯`：相关新闻。
- [x] `事件`：日历、公告、财报、产业事件。
- [x] `相关标的`：相关期货/A 股/外盘参考。
- [x] `模拟持仓`：当前持仓、委托、成交。
- [x] `AI 摘要`：引用式标的速览。

### 5.6 验收标准

- [x] Mock E2E 覆盖期货详情。
- [x] Mock E2E 覆盖 A 股详情。
- [x] Mock E2E 覆盖自选按钮。
- [x] Mock E2E 覆盖详情页触发基础模拟下单入口。
- [x] Live E2E 覆盖期货详情数据与 600000.SH A 股详情数据；RB0/AU0/SC0 底层数据已覆盖。

---

## 6. P0-4 自选模块

### 6.1 数据模型

- [x] 设计统一自选表 `watchlist_items`。
- [x] 支持字段：资产类型、代码、名称、分组、备注、提醒配置、排序、创建时间、更新时间。
- [x] 兼容 A 股现有 `stock_watchlists`，必要时迁移或封装统一 command。
- [x] 支持默认分组：`全部`、`期货`、`A股`、`重点观察`。

### 6.2 后端 Command

- [x] `list_watchlist_items`。
- [x] `add_watchlist_item`。
- [x] `remove_watchlist_item`。
- [x] `update_watchlist_item`。
- [x] `list_watchlist_groups`。
- [x] `create_watchlist_group`。
- [x] `get_watchlist_summary`。
- [x] `get_watchlist_events`。

### 6.3 前端页面

- [x] 新建 `frontend/src/pages/WatchlistPage.tsx`。
- [x] 新建 `WatchlistTable`。
- [x] 新建 `WatchlistGroupTabs`。
- [x] 新建 `WatchlistSummaryPanel`。
- [x] 新建 `WatchlistEventPanel`。
- [x] 新建 `WatchlistAiSummary`。
- [x] 支持分组切换。
- [x] 支持备注编辑。
- [x] 支持提醒展示。
- [x] 支持从市场列表和详情页加入/移除自选。

### 6.4 验收标准

- [x] Mock E2E 覆盖新建分组。
- [x] Mock E2E 覆盖加入/移除自选。
- [x] Mock E2E 覆盖备注编辑。
- [x] Mock E2E 覆盖自选跳转详情页。
- [x] 自选页首屏展示自选异动和相关事件。

---

## 7. P0-5 基础模拟盘收敛

### 7.1 产品边界

- [x] 前端默认只暴露市价/限价、买卖、开仓/平仓、手数、费用估算。
- [x] 前端不暴露止损、止盈、OCO、条件单、移动止损。
- [x] 前端不暴露回放训练作为主业务入口。
- [x] 文档明确高级订单与回放为实验/后续能力。
- [x] 设置页模拟规则只保留基础规则展示，避免暴露过多撮合细节。

### 7.2 页面结构

- [x] 账户摘要：权益、可用资金、保证金、风险度。
- [x] 基础下单：标的、方向、开平、类型、价格、手数、费用估算。
- [x] 持仓表：品种、方向、数量、成本、现价、浮盈亏、保证金。
- [x] 委托表：时间、品种、方向、价格、数量、状态、撤单。
- [x] 成交表：时间、品种、方向、成交价、数量。
- [x] 增加资金流水。
- [x] 增加账户重置入口，但必须二次确认。
- [x] 增加“仅模拟，不构成真实交易”固定标识。

### 7.3 后端与数据

- [x] 确认 `place_sim_order` 对基础参数有完整校验。
- [x] 确认 `estimate_sim_order` 缺行情时返回可解释错误。
- [x] 委托拒单原因标准化。
- [x] 持仓与成交增加数据质量/估算状态。

### 7.4 验收标准

- [x] Mock E2E 覆盖基础下单与费用估算。
- [x] Mock E2E 覆盖撤单。
- [x] Mock E2E 覆盖空持仓/空成交状态。
- [x] Live E2E 覆盖基础限价委托提交和撤单。
- [x] 页面不出现“日内训练”“回放训练”“OCO”“移动止损”等主业务文案。

---
## 8. P1-1 事件资讯中心

### 8.1 信息合并

- [x] 新增 `/events` 页面。
- [x] 合并金十资讯。
- [x] 合并财经日历。
- [x] 合并 A 股公告。
- [x] 合并财报日历。
- [x] 合并产业事件。
- [x] 支持按时间、标的、板块、来源、重要性筛选。

### 8.2 数据模型

- [x] 设计统一事件 DTO `MarketEvent`。
- [x] 字段包含：时间、标题、来源、事件类型、影响标的、影响板块、重要性、方向、原始链接/来源 ID。
- [x] 资讯入库时生成影响标的列表。
- [x] 日历事件补齐影响板块映射。
- [x] A 股公告与财报事件预留数据源字段。

### 8.3 页面组件

- [x] `EventTimeline`。
- [x] `EventFilterBar`。
- [x] `EventImpactTags`。
- [x] `EventDetailDrawer`。
- [x] `RelatedAssetsPanel`。
- [x] `EventAiAnalysisButton`。

### 8.4 验收标准

- [x] Mock E2E 覆盖 `/events` 可访问。
- [x] Mock E2E 覆盖筛选金十资讯。
- [x] Mock E2E 覆盖筛选财经日历。
- [x] Mock E2E 覆盖事件跳转标的详情。
- [x] Live E2E 覆盖金十连通和日历事件拉取。

---

## 9. P1-2 数据库中心升级

### 9.1 数据资产视角

- [x] 将数据库页从统计卡升级为数据资产中心。
- [x] 展示行情、资讯、日历、报告、模拟交易、自选、配置等数据域。
- [x] 每个数据域展示记录数、时间范围、最后更新、来源、质量、占用空间。
- [x] 支持单数据域同步。
- [x] 支持单数据域导出。
- [x] 支持单数据域清理。
- [x] 支持全库备份。

### 9.2 后端 Command

- [x] 扩展 `get_database_summary` 为分域结构。
- [x] 新增 `sync_data_domain`。
- [x] 新增 `export_data_domain`。
- [x] 新增 `cleanup_data_domain`。
- [x] 所有清理操作必须二次确认。

### 9.3 验收标准

- [x] Mock E2E 覆盖数据库页数据域列表。
- [x] Mock E2E 覆盖备份入口。
- [x] Rust 测试覆盖 summary 查询。
- [x] Rust 测试覆盖导出路径生成，不覆盖用户文件。

---

## 10. P1-3 引用式 AI

### 10.1 产品原则

- [x] AI 不是泛聊天入口，而是本地数据解释层。
- [x] 每段 AI 输出必须展示引用来源。
- [x] 每段 AI 输出必须展示数据日期。
- [x] 每段 AI 输出必须包含“仅供研究与复盘，不构成投资建议”。
- [x] 不允许 AI 使用未标明来源的数据。

### 10.2 AI 入口

- [x] 总览页：生成市场摘要。
- [x] 市场页：解释排行榜原因。
- [x] 详情页：生成标的速览。
- [x] 自选页：生成自选摘要。
- [x] 模拟盘：解释持仓风险和盈亏来源。
- [x] 事件页：解释事件影响。
- [x] AI 页：报告任务、历史归档、引用来源查看。

### 10.3 后端与 Prompt

- [x] 统一 `AiContextBundle`。
- [x] Prompt 输入分区：行情、资讯、事件、财务、模拟持仓、历史报告。
- [x] 输出要求包含 `sources` 列表。
- [x] 报告表保存引用来源。
- [x] LLM 失败时保留上下文和重试入口。

### 10.4 验收标准

- [x] Mock E2E 覆盖 AI 摘要按钮。
- [x] Live E2E 覆盖至少 1 个期货分析生成。
- [ ] Live E2E 覆盖至少 1 个事件影响分析。
- [x] Live E2E 断言报告包含免责声明。
- [x] Live E2E 断言引用式 AI 输出包含来源引用。

---
## 11. P2 旧功能清理与文档对齐

### 11.1 代码清理

- [x] 删除旧因子页 `FactorCenterPage`。
- [x] 删除旧资讯页 `NewsDecisionPage`。
- [x] 删除旧日历页 `MacroCalendarPage`。
- [x] 删除旧异动页 `AnomalyCenterPage`。
- [x] 删除旧 Copilot 页 `CopilotPage`。
- [x] 删除旧侧栏组件 `SidebarNavItem`。
- [x] 将旧 `/factors` 重定向到 `/ai`。
- [x] 将旧 `/anomalies` 重定向到 `/events`。
- [x] 将 `/review` 与 `/replay` 放到设置页实验区。
- [x] 删除旧 `DashboardPage`、`SymbolsPage`、`SymbolDetailPage`、`AStockPage` 页面入口。
- [ ] 评估 `TradingReviewPage` 与 `MarketReplayPage` 是否继续作为实验区保留。
- [ ] 仍有价值的旧内容迁移到新页面或组件。
- [x] 删除旧 `features/news`、`features/calendar`、`features/symbol`、`features/stocks` 无引用组件。
- [ ] 删除无引用 E2E mock 数据。

### 11.2 文档更新

- [x] 更新 `docs/DESIGN.md` 到 Coinbase/CMC 风格信息架构。
- [x] 更新 `docs/REQUIREMENTS.md` 的页面矩阵和范围边界。
- [x] 更新 `docs/DEVELOPMENT_PLAN.md`，引用本执行计划。
- [x] 更新 `docs/SIMULATION_TRADING_DESIGN.md`，明确基础模拟盘优先。
- [x] 更新 `docs/USAGE.md`，说明新导航与基础模拟盘使用方式。
- [x] 更新 `docs/ARCHITECTURE.md` 的真实页面树、command 和数据流。
- [x] 标记旧 A 股设计稿为历史设计。

### 11.3 验收标准

- [ ] 文档中一级导航描述一致。
- [ ] 文档中不再把回放、复杂条件单作为当前主功能。
- [ ] 文档中明确 CMC 参考的是产品结构，不是业务照搬。
- [x] `rg "当前最新 UI 设计稿"` 不再指向过期稿。

---

## 12. 测试计划

### 12.1 前端基础检查

- [x] `pnpm --dir frontend tsc`
- [x] `pnpm --dir frontend lint`
- [x] `pnpm --dir frontend test`
- [x] `pnpm --dir frontend build`

### 12.2 Mock E2E

- [x] 应用启动进入总览。
- [x] 顶部导航所有主入口可访问。
- [x] `/markets` 列表展示期货和 A 股资产。
- [x] 市场表格筛选、排序、跳转详情。
- [x] 详情页展示 K 线、关键指标、事件、AI 摘要入口。
- [x] 自选添加、移除、分组、备注。
- [x] 基础模拟盘下单、费用估算、撤单。
- [x] 事件资讯筛选、跳转详情。
- [x] 数据库资产中心展示数据域。
- [x] AI 输出展示引用来源。

### 12.3 Live E2E

- [x] Tauri 客户端健康检查。
- [x] LLM provider 健康检查。
- [x] 五大期货板块主力数据检查：RB0、AU0、M0、SC0、EC0。
- [x] A 股指数/个股数据检查。
- [x] 金十资讯连通。
- [x] 财经日历连通。
- [x] 标的详情真实数据检查。
- [x] 引用式 AI 分析生成。
- [x] 报告入库检查。

### 12.4 Rust 测试

- [x] market asset 自选过滤和真实 A 股降级链路由 live/mock E2E 覆盖。
- [x] watchlist CRUD 测试。
- [ ] event aggregation 专项测试。
- [ ] database domain summary 专项测试。
- [x] AI context sources fallback 测试。
- [x] simulation 基础下单/撤单校验测试。

---

## 13. 组件实现清单

### 13.1 Layout

- [x] `TopNav`
- [x] `PageShell`
- [x] `PageHeader`
- [x] `GlobalMarketBar`
- [ ] `SectionHeader`

### 13.2 Market

- [x] `AssetTable`
- [x] `AssetIdentityCell`
- [x] `PriceChangeCell`
- [x] `MiniSparkline`
- [x] `MarketFilters`
- [x] `MarketLeaderboard`
- [ ] `CategoryTabs`
- [x] `WatchButton`
- [x] `DataQualityBadge`

### 13.3 Detail

- [x] `AssetHeader`
- [x] `KlinePanel`
- [x] `AssetStatsGrid`
- [x] `AssetDetailTabs`
- [x] `RelatedAssetsPanel`
- [x] `AssetEventsPanel`
- [x] `AssetPositionPanel`
- [x] `AssetAiSummary`

### 13.4 Watchlist

- [x] `WatchlistTable`
- [x] `WatchlistGroupTabs`
- [x] `WatchlistSummaryPanel`
- [x] `WatchlistEventPanel`
- [x] `WatchlistAiSummary`
- [x] `WatchlistNoteEditor`

### 13.5 Simulation

- [x] `BasicOrderTicket`
- [x] `PositionTable`
- [x] `OrderTable`
- [x] `TradeTable`
- [x] `CashFlowTable`
- [x] `AccountResetDialog`

### 13.6 Events

- [x] `EventTimeline`
- [x] `EventFilterBar`
- [x] `EventImpactTags`
- [x] `EventDetailDrawer`
- [x] `EventAiAnalysisButton`

### 13.7 Database / AI

- [x] `DataDomainCard`
- [x] `DataDomainTable`
- [x] `DataSyncAction`
- [x] `AiTaskPanel`
- [x] `AiSourceList`
- [x] `AiReportCard`

---

## 14. 推荐执行顺序

1. [x] 完成路由和导航收敛。
2. [x] 抽象 `AssetTable` 与 `DataQualityBadge`。
3. [x] 实现 `/markets`，先接 mock，再接真实期货/A 股数据。
4. [x] 实现统一详情页，先覆盖期货，再覆盖 A 股。
5. [x] 实现 `/watchlist`，打通市场列表和详情页星标。
6. [x] 补齐基础模拟盘撤单、资金流水和空态。
7. [x] 合并资讯/日历到 `/events`。
8. [x] 升级数据库页为数据资产中心。
9. [x] 实现引用式 AI 上下文和报告来源。
10. [ ] 清理旧入口、旧文档和旧测试。

---

## 15. Definition of Done

本轮重构完成时必须满足：

- [x] 一级导航与本计划一致。
- [x] CMC 风格视觉设计落地到真实页面，不只停留在 HTML 设计稿。
- [x] 市场列表、详情页、自选、基础模拟盘、事件资讯、数据库、AI 均有可访问页面。
- [x] 模拟盘保持基础业务边界，不暴露复杂交易入口。
- [x] 所有核心数据均显示来源、时间和质量状态。
- [x] AI 输出有来源引用、日期和免责声明。
- [x] Mock E2E 覆盖核心页面。
- [x] Live E2E 覆盖真实数据与 AI 分析。
- [ ] 文档、测试、代码三者一致。
