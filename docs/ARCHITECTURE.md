# 架构设计（ARCHITECTURE）

> 版本：v2.4 · 2026-07-09  
> 形态：Tauri v2 桌面应用，Rust 单体核心 + React 前端

---

## 1. 总体架构

ThisIsMyQuant 采用本地桌面单体架构。Rust 负责期货与 A 股数据采集、模拟撮合、订单/持仓/资金计算、存储、分析编排、后台任务和 Tauri Commands；React 负责模拟盘、A 股工作台、图表、筛选、报告和交互状态。

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Tauri Rust Core                              │
│  bootstrap · AppState · SQLite · commands · background services      │
│                                                                     │
│  ┌──────────── adapters ────────────┐ ┌──────────── engine ────────┐ │
│  │ AKShare/Sina  Jin10  Calendar    │ │ sectors  dimensions        │ │
│  │ Yahoo StockData LLM Router       │ │ indicator analysis alerts  │ │
│  │                                  │ │ sim_order matching pnl     │ │
│  └──────────────────────────────────┘ └────────────────────────────┘ │
│                                                                     │
│  ┌────────────────────── services ─────────────────────────────────┐ │
│  │ quote_cache market_poll sim_trading news_poll news_ingest       │ │
│  │ matching_engine analysis_runner anomaly_watcher history_backfill│ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ Tauri invoke + events
┌──────────────────────────────▼──────────────────────────────────────┐
│                      React + Vite Frontend                           │
│  总览 期货 A股 报告 数据库 状态 设置                                  │
│  TanStack Query · Zustand · lightweight-charts · shadcn/ui           │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ local file
┌──────────────────────────────▼──────────────────────────────────────┐
│                       SQLite data/quant.db                           │
│  klines contracts stocks news reports sim_orders sim_trades positions│
│  equity_snapshots journal preferences credentials                    │
└─────────────────────────────────────────────────────────────────────┘
```

## 2. 产品边界对架构的影响

- 应用提供模拟盘能力，但只使用虚拟资金和本地撮合；没有真实交易网关，不连接期货公司柜台，不发送真实委托。
- 期货 v1 覆盖五大商品板块：黑色建材、有色贵金属、农产品软商品、能源化工、航运运价。
- A 股规划覆盖指数、行业/概念、个股、财报、资金情绪、筛选器和模拟组合。
- 金融期货不进入 v1 默认目录；代码中保留可扩展空间，但 UI、批量分析和默认关注列表不主动展示金融期货。
- 所有前端数据请求必须经 Tauri IPC 进入 Rust；前端不直接访问 AKShare、金十或 LLM。

## 3. Rust 核心分层

```
src-tauri/src/
├── lib.rs                    # bootstrap、状态初始化、注册 commands、启动后台任务
├── main.rs                   # Tauri 二进制入口
├── state.rs                  # AppState：DB、配置、适配器、缓存、任务句柄
├── models.rs                 # 前后端共享 DTO
├── commands/                 # Tauri IPC 命令分组
│   ├── data.rs               # 行情、资讯、日历、专业工作台聚合
│   ├── simulation.rs         # 虚拟账户、模拟下单、持仓、资金、复盘
│   ├── analysis.rs           # 报告、流式分析、追问、批量任务
│   ├── settings.rs           # 设置、Provider、偏好、凭据
│   ├── debug.rs              # 调试与 E2E 探针
│   └── mod.rs
├── adapters/
│   ├── akshare.rs            # AKShare/Sina 行情和历史
│   ├── stock_data.rs         # A 股股票、指数、行业、财务、资金面适配
│   ├── jinshi.rs             # 金十资讯
│   ├── jinshi_calendar.rs    # 金十财经日历
│   ├── llm.rs                # OpenAI 兼容 Provider 路由
│   └── yahoo.rs              # 海外参考行情
├── engine/
│   ├── sectors.rs            # 期货五大板块、主流品种、中文名、主力符号
│   ├── stock_factors.rs      # A 股技术、财务、估值、资金因子
│   ├── dimensions.rs         # 分析维度与关键词
│   ├── indicator.rs          # MA、BOLL、MACD、RSI 等
│   ├── analysis.rs           # Prompt 构建与上下文渲染
│   ├── report_parse.rs       # LLM 报告结构化解析
│   ├── liquidity.rs          # 流动性评分
│   ├── anomaly.rs            # 异动识别与信号生成
│   ├── sim_matching.rs       # 本地撮合、订单状态流转
│   ├── sim_account.rs        # 资金、保证金、手续费、风险度
│   └── sim_performance.rs    # 绩效指标、资金曲线、回撤
├── services/
│   ├── quote_cache.rs        # 行情缓存
│   ├── market_poll.rs        # 行情轮询与 kline-update 事件
│   ├── stock_data_sync.rs    # A 股目录、行情、财务、行业/概念同步
│   ├── sim_trading.rs        # 模拟交易编排、订单/成交/持仓落库
│   ├── replay_runner.rs      # 历史行情回放训练
│   ├── news_poll.rs          # 资讯轮询
│   ├── news_ingest.rs        # 资讯分类与入库
│   ├── analysis_runner.rs    # 报告生成、免责声明兜底
│   ├── analysis_followup.rs  # Copilot 追问
│   ├── batch_analysis.rs     # 批量分析
│   ├── schedule_runner.rs    # 定时任务
│   ├── daily_briefing.rs     # 每日简报
│   ├── anomaly_watcher.rs    # 异动监测
│   ├── history_backfill.rs   # 历史回填
│   └── data_maintenance.rs   # 数据维护
└── db/
    ├── sqlite.rs             # schema、查询、写入、偏好
    └── questdb.rs            # 可选时序库适配预留
```

依赖方向：

```
commands → services → engine/adapters/db
```

`engine` 保持领域纯度，尽量不直接依赖 Tauri；`adapters` 只处理外部接口；`services` 负责编排和错误降级。

## 4. 前端结构

```
frontend/src/
├── api/
│   ├── client.ts             # invoke 统一入口
│   └── e2e-mock.ts           # Playwright mock 数据
├── components/
│   ├── AppShell.tsx          # 侧栏、顶部、布局
│   └── ui/                   # shadcn/ui 原子组件
├── features/
│   ├── chart/                # K 线与指标
│   ├── analysis/             # 报告、流式分析、时间线
│   ├── market/               # 品种、行情、板块
│   └── news/                 # 资讯与分类展示
├── pages/
│   ├── OverviewPage.tsx      # 专业分析工作台
│   ├── DashboardPage.tsx     # 行情
│   ├── SimulationPage.tsx    # 模拟盘
│   ├── TradingReviewPage.tsx # 交易复盘
│   ├── MarketReplayPage.tsx  # 回放训练
│   ├── AStockPage.tsx        # A 股总览、行业、个股、筛选、财报、组合
│   ├── FactorCenterPage.tsx  # 因子中心
│   ├── NewsDecisionPage.tsx  # 资讯决策中心
│   ├── MacroCalendarPage.tsx # 日历与宏观
│   ├── AnomalyCenterPage.tsx # 异动预警中心
│   ├── CopilotPage.tsx       # Copilot 研究助手
│   ├── ReportsPage.tsx       # 报告归档
│   ├── SymbolsPage.tsx       # 品种目录
│   ├── SymbolDetailPage.tsx  # 品种详情
│   ├── LocalDatabasePage.tsx # 本地数据库
│   ├── StatusPage.tsx        # 数据源和模拟引擎状态
│   └── SettingsPage.tsx      # 设置
├── app/store.ts              # Zustand UI 状态
├── ws/socket.ts              # Tauri event 监听兼容层
└── App.tsx                   # HashRouter 路由
```

## 5. 专业工作台聚合

首页和多个新页面共用 `get_professional_dashboard` 聚合视图，后端返回：

| 字段 | 用途 |
|---|---|
| `decision_flow` | 行情、资讯、日历、异动、报告节点组成的决策时间线。 |
| `factors` | 品种/板块因子快照，包含方向、强度、置信度和数据质量。 |
| `alerts` | 异动预警与风险提示。 |
| `report_workflow` | 报告任务状态、触发类型、完成度。 |
| `overseas_links` | 原油、黄金、铜、农产品等海外参考品种。 |
| `sim_account` | 当前模拟账户权益、可用资金、风险度、今日盈亏。 |
| `sim_positions` | 当前模拟持仓和持仓风险。 |

该接口面向首屏体验，目标是一次请求拿到工作台主要卡片，减少前端多接口瀑布。

## 6. 启动流程

1. 前端加载 `BootstrapLoader`。
2. Rust `bootstrap()` 加载 `.env` 和用户偏好。
3. 初始化 SQLite 并执行 additive schema 初始化。
4. 初始化 AKShare/Sina、金十、日历、Yahoo、LLM Router 和模拟交易配置。
5. 启动行情、模拟撮合、资讯、历史回填、定时分析、日历、异动等后台任务。
6. `emit("app-ready")`，前端开始请求 `get_health` 和工作台数据。

## 7. 数据流

### 行情链路

```
AKShare/Sina → market_poll → quote_cache / SQLite → kline-update event → ChartPanel
```

### 资讯链路

```
Jin10 → news_poll → news_ingest → rule/LLM classify → news_items/news_classifications → 决策流/Prompt
```

### 报告链路

```
用户/调度/异动触发 → analysis_runner → LLM Router → reports 入库 → analysis events → 报告页/Copilot
```

### 专业工作台链路

```
SQLite + caches + services state → professional_dashboard_view → get_professional_dashboard → 总览/因子/资讯/异动
```

### A 股链路

```
AKShare/Baostock/Tushare → stock_data_sync
  → stock_symbols / stock_daily_bars / stock_financial_metrics / stock_factor_snapshots
  → get_a_stock_dashboard / get_stock_detail / run_stock_screener
  → A 股总览、行业概念、个股工作台、筛选器、财报中心
```

### 模拟交易链路

```
用户下单 → place_sim_order → 风控校验 → sim_orders
  → matching_engine 根据 quote/replay bar 撮合
  → sim_trades / sim_positions / sim_accounts / sim_equity_snapshots
  → sim-order-update / sim-account-update events
```

### 回放训练链路

```
选择品种和日期 → start_market_replay
  → replay_runner 按 bar 推进行情
  → matching_engine 使用回放行情撮合
  → 训练记录与交易复盘入库
```

## 8. 技术选型

| 领域 | 选型 | 说明 |
|---|---|---|
| 桌面壳 | Tauri v2 | 原生桌面窗口与 Rust IPC。 |
| 后端 | Rust 2021 | 数据、分析、任务调度统一在本地核心。 |
| 前端 | React 18 + Vite + TypeScript | 桌面优先的单页应用。 |
| UI | shadcn/ui + Tailwind | 克制、低噪声的专业金融工作台。 |
| 图表 | lightweight-charts 5 | K 线、成交量、技术指标。 |
| 状态 | TanStack Query + Zustand | 异步数据与 UI 状态分离。 |
| 存储 | SQLite | 本地单文件数据库，WAL 模式。 |
| LLM | OpenAI 兼容协议 | 多 Provider 路由与 fallback。 |
| 模拟撮合 | Rust engine/service | 本地虚拟账户、订单、成交、持仓、保证金、手续费。 |
| A 股数据 | AKShare/Baostock/Tushare 可选 | 股票、指数、行业概念、财务、资金流和公告。 |

## 9. 配置与安全

- `.env` 用于本地开发凭据和默认配置，生产偏好存入 SQLite。
- LLM Key 等敏感凭据加密后落库。
- 日志禁止输出 API Key、Token、完整凭据或用户隐私。
- 报告必须保留免责声明：仅供参考，不构成投资建议。
- 模拟盘必须保留“模拟交易”标识，不得出现真实下单、实盘账户、交易密码配置入口。
- Tauri 权限集中在 `src-tauri/capabilities/default.json`。

## 10. 测试架构

| 层级 | 说明 |
|---|---|
| Vitest | 前端纯函数、布局算法、数据适配。 |
| Playwright mock E2E | `VITE_E2E_MOCK=true`，覆盖页面和交互。 |
| Client live E2E | 启动真实 Tauri 后端，覆盖健康检查、专业工作台、LLM 报告。 |
| Rust tests | SQLite、适配器、分析、命令和服务逻辑。 |

详见 [USAGE.md](./USAGE.md)。
