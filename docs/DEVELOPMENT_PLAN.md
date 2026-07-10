# 后续功能开发与 UI 重构规划

> 版本：v2.2 · 2026-07-09  
> 状态：**期货模拟盘阶段已落地，下一阶段进入 A 股市场分析实现**  
> 依据：`docs/REQUIREMENTS.md` v1.3、`docs/DESIGN.md` v3.3、`docs/A_STOCK_MARKET_ANALYSIS_DESIGN.md`、`docs/A_STOCK_FEATURE_IMPLEMENTATION.md`、`docs/A_STOCK_UI_INTERACTION_DESIGN.html`

---

## 1. 规划背景

近期仓库出现两类关键变更：

1. **分析工作台 UI 矩阵已落地**：`FactorCenterPage`、`NewsDecisionPage`、`MacroCalendarPage`、`AnomalyCenterPage`、`CopilotPage` 五个页面已作为未跟踪文件加入工作区，并补充到 `App.tsx` 路由与侧栏导航。`OverviewPage` 已完成热力图与定时分析重构。
2. **模拟盘设计文档新增**：`docs/SIMULATION_TRADING_DESIGN.md` 首次系统定义了虚拟账户、本地撮合、订单/成交/持仓/资金、保证金/手续费规则、交易复盘、回放训练和本地数据库 Schema。

这意味着产品已从「期货分析工作台」演进为「**国内期货与 A 股的模拟盘 + 本地数据库 + 分析复盘工作台**」。本规划用于对齐文档、代码与下一阶段的实施顺序。

当前最新 UI 设计稿为 `docs/A_STOCK_UI_INTERACTION_DESIGN.html`。

---

## 2. 当前实现状态速览

### 2.1 已完成的分析链路（M5 基本落地）

| 页面 | 路由 | 状态 | 关键依赖 |
|---|---|---|---|
| 总览 | `/` | ✅ 已实现 | `get_professional_dashboard` |
| 行情 | `/workspace` | ✅ 已实现 | K 线、指标、实时订阅 |
| 因子 | `/factors` | ✅ 已实现 | `get_professional_dashboard` |
| 资讯 | `/news` | ✅ 已实现 | `get_professional_dashboard` |
| 日历 | `/calendar` | ✅ 已实现 | `list_calendar_events` |
| 异动 | `/anomalies` | ✅ 已实现 | `get_professional_dashboard` |
| 助手 | `/copilot` | ✅ 已实现 | `AiPanel`、流式分析 |
| 报告 | `/reports` / `/reports/:id` / `/reports/compare` | ✅ 已实现 | `list_reports`、`get_report` |
| 品种 | `/symbols` / `/symbols/:symbol` | ✅ 已实现 | `list_products` |
| 状态 | `/status` | ✅ 已实现 | `get_status_dashboard` |
| 设置 | `/settings` | ✅ 已实现 | 多 section 设置页 |

### 2.2 已补齐的核心功能（M6/M7 已落地）

| 设计页面 | 路由 | 前端状态 | 后端状态 |
|---|---|---|---|
| **模拟盘** | `/simulation` | ✅ `SimulationPage.tsx` | ✅ `commands/simulation.rs` + `services/sim_trading.rs` + `engine/sim_*` |
| **交易复盘** | `/review` | ✅ `TradingReviewPage.tsx` | ✅ `generate_trade_review` command + 日记表 |
| **回放训练** | `/replay` | ✅ `MarketReplayPage.tsx`（图表、下单面板、持仓/委托/成交） | ✅ `services/replay_runner.rs` + `sim_replay_sessions` + 真实 replay command |
| **本地数据库** | `/database` | ✅ `LocalDatabasePage.tsx` | ✅ `get_database_summary` + `backup_database` |

### 2.3 已修复的不一致点

1. **侧栏导航顺序**：已按 `DESIGN.md` 调整为「总览 → 行情 → 模拟盘 → 复盘 → 回放 → 因子 → 资讯 → 日历 → 异动 → 助手 → 报告 → 品种 → 数据库 → 状态 → 设置」。
2. **架构图对齐**：`commands/simulation.rs`、`engine/sim_matching.rs` / `sim_account.rs` / `sim_contract.rs` / `sim_risk.rs`、`services/sim_trading.rs` 已真实创建。
3. **孤儿组件清理**：已删除未引用的 `FeatureCard.tsx` 和 `MorningBriefing.tsx`。
4. **总览页信息层级**：顶部新增模拟账户权益/可用资金/今日盈亏/风险度卡片。
5. **品种详情四段式**：Header、多周期图表 + 驱动解释、报告时间线 / 关联品种 / 追问记录 Tabs。

---

## 3. 实施完成记录

### 阶段一：补齐页面矩阵与导航（已完成）

| 任务 | 状态 | 关键文件 |
|---|---|---|
| 新增模拟盘页面 | ✅ | `frontend/src/pages/SimulationPage.tsx` |
| 新增复盘页面 | ✅ | `frontend/src/pages/TradingReviewPage.tsx` |
| 新增回放页面 | ✅ | `frontend/src/pages/MarketReplayPage.tsx` |
| 新增数据库页面 | ✅ | `frontend/src/pages/LocalDatabasePage.tsx` |
| 注册路由 | ✅ | `frontend/src/App.tsx` |
| 重构侧栏导航 | ✅ | `frontend/src/components/AppShell.tsx` |
| 扩展类型定义 | ✅ | `frontend/src/types.ts` |
| 扩展 API 客户端 | ✅ | `frontend/src/api/client.ts` |
| Mock E2E 覆盖 | ✅ | `frontend/src/api/e2e-mock.ts`、`frontend/e2e/ui-mock.spec.ts` |
| 清理孤儿组件 | ✅ | 删除 `FeatureCard.tsx`、`MorningBriefing.tsx` |

### 阶段二：模拟盘后端核心（已完成）

| 任务 | 状态 | 关键文件 |
|---|---|---|
| 数据库 Schema | ✅ | `src-tauri/src/db/sqlite.rs` 新增 7 张表 |
| DTO 定义 | ✅ | `src-tauri/src/models.rs` |
| 合约规则 engine | ✅ | `src-tauri/src/engine/sim_contract.rs` |
| 账户与资金 engine | ✅ | `src-tauri/src/engine/sim_account.rs` |
| 撮合 engine | ✅ | `src-tauri/src/engine/sim_matching.rs` |
| 风控 engine | ✅ | `src-tauri/src/engine/sim_risk.rs` |
| 交易服务 | ✅ | `src-tauri/src/services/sim_trading.rs` |
| Command 层 | ✅ | `src-tauri/src/commands/simulation.rs` |
| 启动初始化 | ✅ | `src-tauri/src/lib.rs` |
| 事件推送 | ✅ | `sim-order-update`、`sim-account-update` |

### 阶段三：模拟盘前端与交易复盘（已完成）

| 任务 | 状态 | 关键文件 |
|---|---|---|
| API 客户端扩展 | ✅ | `frontend/src/api/client.ts`、`e2e-mock.ts` |
| 模拟盘页面组装 | ✅ | `frontend/src/pages/SimulationPage.tsx` |
| 交易复盘页面 | ✅ | `frontend/src/pages/TradingReviewPage.tsx` |
| LLM 复盘 Command | ✅ | `src-tauri/src/commands/analysis.rs` `generate_trade_review` |
| LLM 复盘入口 | ✅ | `TradingReviewPage` "生成 LLM 复盘" 按钮 |

### 阶段四：回放训练、本地数据库与 UI 重构（已完成）

| 任务 | 状态 | 关键文件 |
|---|---|---|
| 回放页面 | ✅ | `frontend/src/pages/MarketReplayPage.tsx` |
| 回放 Runner 服务 | ✅ | `src-tauri/src/services/replay_runner.rs` |
| 回放状态持久化 | ✅ | `db/sqlite.rs` `sim_replay_sessions` |
| 回放 Command | ✅ | `commands/simulation.rs` `start/stop/step/get_replay_state/get_replay_klines` |
| 本地数据库页面 | ✅ | `frontend/src/pages/LocalDatabasePage.tsx` |
| 数据库 summary / backup | ✅ | `db/sqlite.rs`、`commands/simulation.rs` |
| 总览账户摘要条 | ✅ | `frontend/src/pages/OverviewPage.tsx` |
| 品种详情四段式 | ✅ | `frontend/src/pages/SymbolDetailPage.tsx` |

### 阶段五：高级订单、绩效分析与异动-持仓-风险联动（已完成）

| 任务 | 状态 | 关键文件 |
|---|---|---|
| 高级订单 UI | ✅ | `frontend/src/pages/SimulationPage.tsx` 支持市价/限价/止损/止损限价/条件单、止损止盈、OCO |
| 实时行情与 WS 联动 | ✅ | `frontend/src/hooks/useRealtimeQuotes.ts`、`frontend/src/ws/socket.ts` |
| 费用估算 | ✅ | `frontend/src/pages/SimulationPage.tsx` 调用 `estimate_sim_order` |
| 账户选择器 | ✅ | `frontend/src/pages/SimulationPage.tsx` 多账户切换 |
| 绩效分析后端 | ✅ | `src-tauri/src/services/sim_trading.rs` `get_performance` |
| 绩效 Command | ✅ | `src-tauri/src/commands/simulation.rs` `get_sim_performance` |
| 绩效统计 Tab | ✅ | `frontend/src/pages/TradingReviewPage.tsx` |
| 模拟规则设置面板 | ✅ | `frontend/src/features/settings/SimulationRulesPanel.tsx` |
| 异动持仓风险联动 | ✅ | `src-tauri/src/services/anomaly_watcher.rs` `evaluate_position_risk` |
| 前端风险联动 Toast | ✅ | `frontend/src/hooks/useNotifications.ts` |

---

## 4. 测试与验收结果

| 测试层 | 命令 | 结果 |
|---|---|---|
| Rust 单元/集成测试 | `cargo test --manifest-path src-tauri/Cargo.toml` | ✅ 41 lib + 16 integration + 1 client passed |
| 前端类型检查 | `pnpm --dir frontend tsc` | ✅ 通过 |
| 前端单元测试 | `pnpm --dir frontend test` | ✅ 25 passed |
| 前端 Lint | `pnpm --dir frontend lint` | ✅ 0 errors（仅既有 warnings） |
| Mock E2E | `pnpm test:e2e` | ✅ 12 passed |

---

## 5. 已知局限与后续建议

1. **撮合实时性**：当前 `market_poll` tick 驱动已能触发挂单撮合、止损/条件单与风控强平；但极端行情下仍依赖轮询间隔。
2. **回放训练**：已实现历史 K 线逐 bar 回放、回放下单撮合与状态持久化；后续可补充倍速播放平滑性、跳转指定 bar、账单导入动态回放。
3. **数据质量标识**：模拟盘页面中的持仓/成交暂未显示 `estimated` 等质量状态，后续可按 `DESIGN.md` 统一标注。
4. **绩效分析深化**：已实现核心指标；后续可补充夏普/索提诺、参数优化、月度归因报告等。
5. **异动-持仓-风险联动**：已实现通知与事件推送；后续可在 `AnomalyCenterPage` 或 `SimulationPage` 增加可视化风险影响面板。

> 详细后续规划见 `docs/SIMULATION_ROADMAP.md`。

---

## 6. A 股下一阶段实施规划

详细实现文档见 `docs/A_STOCK_FEATURE_IMPLEMENTATION.md`。本节只保留总排期与交付边界。

### A1：数据源与本地表（P0）

| 任务 | 交付 |
|---|---|
| 股票目录 | `stock_symbols` 表、`list_stock_symbols` command。 |
| 指数与日 K | `stock_daily_bars`、`stock_index_daily_bars` 表。 |
| 行业/概念 | `stock_boards`、`stock_board_members`、`stock_board_snapshots` 表。 |
| 同步服务 | `stock_data_sync`，支持进度事件和 stale/error 降级。 |

### A2：A 股总览与行业概念（P0）

| 任务 | 交付 |
|---|---|
| A 股入口 | 侧栏新增 `A股`，路由 `/stocks`。 |
| 总览页 | 指数卡、市场宽度、涨跌停、成交额、行业热力。 |
| 行业下钻 | 点击热力图进入行业/概念详情，展示成分股和领涨领跌。 |
| Mock E2E | 覆盖 A 股总览与行业点击链路。 |

### A3：个股工作台与筛选器（P0/P1）

| 任务 | 交付 |
|---|---|
| 个股工作台 | K 线、基础资料、财务摘要、估值、资金面、公告新闻。 |
| 股票筛选器 | 条件构建、运行筛选、保存模板、保存结果快照。 |
| 因子快照 | 动量、质量、估值、成长、波动、流动性基础打分。 |
| 数据质量 | 报告期、复权口径、数据源和更新时间必须可见。 |

### A4：模拟组合与 LLM 复盘（P1）

| 任务 | 交付 |
|---|---|
| A 股纸面组合 | 与期货模拟账户隔离，支持 T+1、涨跌停、费用。 |
| 组合归因 | 收益、回撤、行业暴露、个股贡献。 |
| LLM 个股速览 | 引用行情、财务、估值、公告和行业数据。 |
| LLM 组合复盘 | 解释持仓表现、风险暴露和执行问题。 |

---

## 7. 快速参考

```bash
# 后端测试
cargo test --manifest-path src-tauri/Cargo.toml --lib

# 前端检查
pnpm --dir frontend tsc
pnpm --dir frontend lint
pnpm --dir frontend test

# E2E
pnpm test:e2e
```

---

## 8. A4 之后：后续功能路线图（v2.3 / v2.4 规划草案）

> 状态：草案 · 2026-07-09  
> 范围：A 股模块从「可用」到「好用」、期货模拟盘从「能下单」到「像真盘」、以及分析工作台闭环。

### 8.1 优先级总览

| 阶段 | 主题 | 优先级 | 预计周期 | 核心交付 |
|---|---|---|---|---|
| A5 | A 股体验补全（K 线、财报、自选、数据同步） | P0 | 2–3 周 | 真实日 K 图表、财报中心、自选股、筛选模板快照、收盘同步 |
| A6 | A 股资金情绪与组合绩效 | P1 | 2 周 | 主力资金/龙虎榜/涨停梯队、组合收益曲线/回撤/归因 |
| B1 | 期货模拟盘真实撮合与高级订单 | P0 | 2–3 周 | 限价/止损/止盈/OCO/移动止损/条件单触发、部分成交、滑点 |
| B2 | 回放训练与账单导入 | P1 | 2 周 | CSV/Excel 成交导入、动态回放、K 线交易标记 |
| B3 | 模拟盘绩效与风控深化 | P1 | 2 周 | 夏普/索提诺、月度归因、强平模拟、规则导入导出 |
| C1 | 分析工作台联动 | P1 | 2 周 | 因子溯源、外盘联动、异动快评、报告任务队列 |
| D1 | 工程与质量 | 贯穿 | 持续 | 数据质量标识统一、本地数据导入导出、client live E2E、性能优化 |

---

### 8.2 A5：A 股体验补全（P0）

A4 已实现模拟组合骨架，但个股页仍缺少真实 K 线、财报中心和股票池。本阶段让 A 股页达到「日常可用」。

| 任务 | 关键文件/命令 | 验收标准 |
|---|---|---|
| 个股日 K 图表 | `frontend/src/features/stocks/StockDetailWorkspace.tsx`、 `get_stock_klines` | 接入 lightweight-charts，支持 `none/qfq/hfq` 切换，显示成交量。 |
| 财报中心 Tab | `frontend/src/features/stocks/StockFinancialCenter.tsx`、 `list_stock_financials` | 展示盈利能力、成长、现金流、负债、杜邦拆解，显示报告期。 |
| 自选股/股票池 | `db/sqlite.rs stock_watchlists`、`commands/stock.rs` 新增 `save/list/delete_stock_watchlist` | 个股页可「加自选」；筛选结果可保存为股票池。 |
| 筛选模板快照 | `stock_screen_templates/results` 已有表，补齐加载/删除模板、结果历史对比 | 筛选器可加载历史模板，结果列表显示命中原因和数据源。 |
| 收盘自动同步 | `services/stock_data_sync.rs`、调度任务 | 每日 15:35 自动同步 A 股日 K、财务、估值；失败时返回 `stale` 状态。 |
| A 股 LLM 筛选总结 | `commands/stock.rs` 扩展 | 对筛选结果生成共同特征、行业集中度、风险暴露、可观察理由。 |

---

### 8.3 A6：A 股资金情绪与组合绩效（P1）

在 A5 基础上补充短线资金观察工具和组合量化复盘。

| 任务 | 关键文件/命令 | 验收标准 |
|---|---|---|
| 资金情绪面板 | `adapters/stock_data.rs`、新增 `get_stock_flow_snapshot` | 展示主力资金、北向资金、龙虎榜、涨停梯队（先 AKShare，缺数据时 `estimated`）。 |
| ETF 观察 | `stock_symbols` 扩展 `market=ETF`、指数替代卡 | A 股总览可展示宽基/行业 ETF 涨跌。 |
| 组合绩效 | `services/stock_paper_trading.rs`、新增 `get_stock_paper_performance` | 计算收益曲线、最大回撤、胜率、盈亏比、行业暴露、个股贡献。 |
| 组合 LLM 归因 | `generate_stock_portfolio_review` 已存在，增强 prompt | 解释组合表现、风险暴露和执行问题。 |

---

### 8.4 B1：期货模拟盘真实撮合与高级订单（P0，已完成 ✅）

当前模拟盘 UI 已接入高级订单类型，后端撮合已在行情轮询中持续触发。

| 任务 | 关键文件 | 验收标准 |
|---|---|---|
| 行情驱动挂单撮合 | `services/market_poll.rs` → `SimTradingService::on_price_update` | 限价买入（最新价/卖一 ≤ 委托价）、限价卖出（最新价/买一 ≥ 委托价）成交。 |
| 三价取中成交价 | `engine/sim_matching.rs` | 买入成交价 = max(委托价, 卖一价, 最新价)；卖出相反。 |
| 止损/止盈/OCO/移动止损 | `engine/sim_matching.rs`、`services/sim_trading.rs`、`sim_orders` 字段 | 触发后转市价/限价平仓，OCO 成交/部分成交後互撤，移动止损随极值动态收紧。 |
| 部分成交与滑点 | `engine/sim_matching.rs` | 盘口量不足时保留剩余委托；无盘口量时一次性全成；滑点按 tick 向不利方向偏移。 |
| 风控执行 | `engine/sim_risk.rs`、`services/sim_trading.rs` | `block_open` 禁止开仓，强平触发后写入 `sim_risk_events`。 |
| 测试覆盖 | `src-tauri/src/engine/sim_matching.rs`、`src-tauri/tests/sim_trading_test.rs` | 新增单元/集成测试覆盖三价取中、部分成交、OCO、止损/止盈/条件单/移动止损、风控。 |

---

### 8.5 B2：回放训练与账单导入（P1）

回放页面已实现基础步进训练，下一步连接真实复盘数据。

| 任务 | 关键文件 | 验收标准 |
|---|---|---|
| CSV/Excel 成交导入 | `services/replay_runner.rs`、新增导入命令 | 支持字段：时间、品种、方向、开平、价格、手数。 |
| 动态回放与标记 | `MarketReplayPage.tsx`、K 线标记 | 导入成交按时间顺序回放，K 线上标记开平仓点。 |
| 复盘日志关联 | `TradingReviewPage.tsx`、日记表 | 每笔导入成交可添加文字复盘和情绪标签。 |

---

### 8.6 B3：模拟盘绩效与风控深化（P1）

| 任务 | 关键文件 | 验收标准 |
|---|---|---|
| 高级绩效指标 | `services/sim_trading.rs` `get_performance` | 夏普、索提诺、最大回撤、月度归因、品种/时段贡献。 |
| 规则导入导出 | `SettingsPage.tsx` `SimulationRulesPanel.tsx` | 合约规则、风控规则可导出为 JSON 并导入。 |
| 模拟标识强化 | 全模拟盘页面 | 所有下单按钮、账户卡片、导出报告显著标注「模拟交易」。 |

---

### 8.7 C1：分析工作台联动（P1）

| 任务 | 关键文件 | 验收标准 |
|---|---|---|
| 因子溯源 | `FactorCenterPage.tsx`、`engine/dimensions.rs` | 因子卡片可下钻到关联新闻、日历、报告片段。 |
| 外盘联动 | `SymbolDetailPage.tsx`、新增外盘 reference 接口 | 原油/黄金/铜/农产品显示海外参考品种涨跌。 |
| 异动快评 | `AnomalyCenterPage.tsx`、`commands/analysis.rs` | 预警卡片一键生成 LLM 快评并归档为报告。 |
| 报告任务队列 | `OverviewPage.tsx` | 展示待生成、生成中、已完成、失败的报告任务。 |
| Copilot 跨报告追问 | `CopilotPage.tsx` | 可选择多份历史报告作为上下文进行追问。 |

---

### 8.8 D1：工程与质量（贯穿）

| 任务 | 关键文件 | 验收标准 |
|---|---|---|
| 数据质量标识统一 | 各前端页面 | 统一展示 `live/history/stale/error/pending/estimated/reference`。 |
| 本地数据导入导出 | `commands/data.rs`、`LocalDatabasePage.tsx` | 配置、报告、模拟交易、A 股数据可备份/恢复。 |
| Client live E2E | `scripts/e2e-client.sh` | 真实行情/LLM 下的关键链路 E2E 通过。 |
| 性能优化 | `db/sqlite.rs`、前端图表 | 1 万根 K 线查询 < 200ms，大表分页。 |
| 文档同步 | `docs/*.md` | 每完成一个阶段更新设计/需求/实现文档。 |

---

### 8.9 推荐启动顺序

1. **先 A5**：A 股已有完整骨架，补 K 线、财报、自选和数据同步后，用户日常可用。风险低、价值高。
2. **再 B1**：期货模拟盘的高级订单是当前最大体验缺口，行情驱动撮合是关键依赖。
3. **并行 C1/D1**：因子溯源、异动快评和数据质量标识可与 A5/B1 并行，提升整体产品一致性。
4. **最后 A6/B2/B3**：资金情绪、账单导入、高级绩效属于增强型功能，可在主链路稳定后迭代。

---

### 8.10 验收基线

每个阶段完成后仍需满足：

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml --test integration_test -- --nocapture
pnpm --dir frontend tsc
pnpm --dir frontend lint
pnpm --dir frontend test
pnpm test:e2e
```

新增功能必须附带：Rust 单元/集成测试、前端 Mock E2E、数据质量状态处理、LLM 免责声明（如涉及）。
