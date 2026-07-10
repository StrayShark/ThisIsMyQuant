# 模块通信机制（COMMUNICATION）

> 版本：v2.4 · 2026-07-09  
> 通信模型：Tauri IPC + Tauri Events + 测试专用 HTTP 探针

---

## 1. 通信总览

```
前端 ── invoke(command, args) ──▶ Rust commands/*
前端 ◀── emit(event, payload) ──  Rust services / analysis_runner

Rust 内部：
  commands ──▶ services / db
  services ──▶ adapters (AKShare, Jin10, Yahoo, StockData, LLM)
  services ──▶ engine (分析、指标、维度、异动、模拟撮合、A股因子)
  services ──▶ db (SQLite)
```

生产应用不暴露业务 REST 服务。`src-tauri/src/testing/` 中的 HTTP 探针只用于 client live E2E 和调试就绪检查。

## 2. 响应格式

前端统一通过 `frontend/src/api/client.ts` 调用 Tauri command。业务响应格式：

```json
{ "code": 0, "message": "ok", "data": {} }
```

`code != 0` 表示业务错误；`api.unwrap()` 会转为前端 Error，供 React Query 和页面错误态处理。

## 3. Tauri Commands

### 3.1 状态与目录

| Command | 说明 |
|---|---|
| `get_health` | 应用、数据库、数据源、LLM、后台任务健康检查。 |
| `get_settings` | 当前配置快照。 |
| `get_status_dashboard` | 状态页聚合视图。 |
| `list_products` | 五大板块主流品种目录，使用中文名和主力符号。 |
| `list_contracts` | 合约列表和主力信息。 |
| `list_dimensions` | 分析维度字典。 |
| `list_dimension_facts` | 已沉淀的维度事实。 |

### 3.2 行情与专业工作台

| Command | 说明 |
|---|---|
| `get_klines` | 查询历史 K 线。 |
| `market_subscribe` | 订阅行情轮询。 |
| `get_professional_dashboard` | 专业分析工作台聚合视图，包含决策流、因子、异动、报告任务和外盘联动。 |
| `trigger_data_fetch_cycle` | 手动触发数据抓取/回填周期。 |

### 3.3 模拟盘与本地数据库

| Command | 说明 |
|---|---|
| `list_sim_accounts` | 查询模拟账户。 |
| `create_sim_account` | 创建虚拟账户。 |
| `reset_sim_account` | 重置模拟账户。 |
| `get_sim_account_snapshot` | 查询账户权益、可用资金、保证金和风险度。 |
| `place_sim_order` | 提交模拟委托。 |
| `cancel_sim_order` | 撤销模拟委托。 |
| `list_sim_orders` | 查询模拟委托。 |
| `list_sim_trades` | 查询模拟成交。 |
| `list_sim_positions` | 查询模拟持仓。 |
| `list_sim_equity_curve` | 查询资金曲线。 |
| `save_sim_journal_entry` | 保存交易复盘记录。 |
| `list_sim_journal_entries` | 查询交易复盘记录。 |
| `update_sim_contract_rules` | 更新合约保证金、手续费、滑点等模拟规则。 |
| `start_market_replay` / `stop_market_replay` | 启停历史行情回放训练。 |
| `export_local_database` / `import_local_database` | 导出/导入本地行情、报告、模拟交易和复盘数据。 |

### 3.4 资讯与日历

| Command | 说明 |
|---|---|
| `list_news` | 查询资讯列表。 |
| `list_calendar_events` | 查询财经日历。 |
| `trigger_news_reclassify` | 手动触发资讯分类重跑。 |

### 3.5 A 股市场

| Command | 说明 |
|---|---|
| `list_stock_symbols` | 查询 A 股股票目录。 |
| `get_a_stock_dashboard` | A 股总览：指数、市场宽度、行业热力、涨跌停、成交额。 |
| `get_stock_klines` | 查询个股 K 线。 |
| `get_stock_detail` | 查询个股基础资料、行情、财务摘要、估值、资金面和事件。 |
| `list_stock_industries` | 查询行业/概念板块。 |
| `get_stock_industry_detail` | 查询行业/概念成分、涨跌、资金流和领涨领跌。 |
| `run_stock_screener` | 按条件运行股票筛选器。 |
| `save_stock_screen` | 保存筛选模板或结果快照。 |
| `list_stock_financials` | 查询财报和财务指标。 |
| `list_stock_factor_snapshots` | 查询股票因子快照和打分。 |
| `trigger_stock_data_sync` | 手动同步 A 股目录、行情、行业、财务或资金数据。 |

### 3.6 分析、报告与 Copilot

| Command | 说明 |
|---|---|
| `trigger_analysis` | 同步触发单品种分析。 |
| `stream_analysis` | 异步流式分析，结果通过 `analysis-*` 事件推送。 |
| `trigger_batch_analysis` | 按关注列表或板块批量分析。 |
| `list_reports` | 查询报告列表。 |
| `get_report` | 查询报告详情。 |
| `list_followups` | 查询 Copilot 追问历史。 |
| `analysis_followup` | 异步追问，结果通过 `followup-*` 事件推送。 |

### 3.7 设置与维护

| Command | 说明 |
|---|---|
| `save_preferences` | 保存用户偏好。 |
| `get_llm_catalog` | 查询 LLM Provider 与模型目录。 |
| `save_llm_credentials` | 保存加密凭据。 |
| `test_llm_provider` | 测试 LLM Provider 连通性。 |
| `export_data` / `import_data` | 数据导出/导入。 |

实际命令以 `src-tauri/src/commands/` 注册表为准；新增页面必须先通过 `api/client.ts` 增加类型化封装，再在页面调用。

## 4. Tauri Events

| 事件 | 方向 | 说明 |
|---|---|---|
| `app-ready` | Rust → 前端 | 核心初始化完成。 |
| `kline-update` | Rust → 前端 | 行情/K 线增量，供图表实时更新。 |
| `analysis-delta` | Rust → 前端 | LLM 分析流式文本片段。 |
| `analysis-done` | Rust → 前端 | 分析完成，包含报告 ID。 |
| `analysis-error` | Rust → 前端 | 分析失败。 |
| `followup-delta` | Rust → 前端 | Copilot 追问流式文本片段。 |
| `followup-done` | Rust → 前端 | 追问完成。 |
| `followup-error` | Rust → 前端 | 追问失败。 |
| `notification` | Rust → 前端 | 报告完成、任务失败、异动等系统通知。 |
| `sim-order-update` | Rust → 前端 | 模拟委托状态变化。 |
| `sim-trade-update` | Rust → 前端 | 模拟成交生成。 |
| `sim-account-update` | Rust → 前端 | 模拟账户权益、资金、风险度变化。 |
| `market-replay-tick` | Rust → 前端 | 历史行情回放推进。 |
| `stock-sync-progress` | Rust → 前端 | A 股数据同步进度。 |
| `stock-screener-done` | Rust → 前端 | 股票筛选器完成并返回结果快照。 |
| `analysis-error` | Rust → 前端 | 分析失败。 |
| `followup-delta` | Rust → 前端 | Copilot 追问流式文本片段。 |
| `followup-done` | Rust → 前端 | 追问完成。 |
| `followup-error` | Rust → 前端 | 追问失败。 |
| `notification` | Rust → 前端 | 报告完成、任务失败、异动等系统通知。 |
| `sim-order-update` | Rust → 前端 | 模拟委托状态变化。 |
| `sim-trade-update` | Rust → 前端 | 模拟成交生成。 |
| `sim-account-update` | Rust → 前端 | 模拟账户权益、资金、风险度变化。 |
| `market-replay-tick` | Rust → 前端 | 历史行情回放推进。 |

前端事件监听集中在 `frontend/src/ws/socket.ts` 和相关 feature hook 中。

## 5. 核心时序

### 5.1 启动

```
Tauri bootstrap
  → SQLite init
  → adapters init
  → services start
  → emit app-ready
  → frontend get_health + get_professional_dashboard
```

### 5.2 专业工作台

```
前端 OverviewPage
  → invoke get_professional_dashboard
  → Rust 汇总 reports/news/calendar/quotes/alerts/factors/sim_account
  → 返回 ProfessionalDashboardView
  → 总览、模拟盘、因子、资讯、异动页面复用局部数据
```

### 5.3 模拟下单

```
前端 invoke place_sim_order
  → Rust 校验账户、合约规则、资金、风险阈值
  → sim_orders 入库
  → matching_engine 判断是否立即成交
  → 更新 sim_trades / sim_positions / sim_accounts
  → emit sim-order-update / sim-trade-update / sim-account-update
```

### 5.4 历史回放训练

```
前端 invoke start_market_replay
  → Rust 加载指定日期 K 线
  → replay_runner 推进 bar
  → emit market-replay-tick
  → matching_engine 用回放价格撮合模拟订单
```

### 5.5 行情更新

```
market_poll 定时拉取 AKShare/Sina
  → quote_cache 更新
  → SQLite 写入或合成 K 线
  → emit kline-update
  → ChartPanel 更新最后一根 candle/volume
```

### 5.6 A 股总览

```
前端 invoke get_a_stock_dashboard
  → Rust 汇总指数、市场宽度、行业/概念、涨跌停、成交额
  → 返回 AStockDashboardView
  → A 股总览和行业概念页展示
```

### 5.7 股票筛选

```
前端 invoke run_stock_screener({ criteria })
  → Rust 查询 stock_daily_bars / financial_metrics / factor_snapshots
  → 生成结果快照
  → 可选写入 stock_screen_results
```

### 5.8 资讯分类

```
news_poll 拉取金十
  → news_ingest 去重
  → 规则分类 + 可选 LLM 分类
  → news_items/news_classifications 入库
  → 专业工作台与 Prompt 可读取
```

### 5.9 流式分析

```
前端 invoke stream_analysis({ symbol, trigger })
  → Rust spawn analysis_runner
  → emit analysis-delta 多次
  → reports 入库，兜底免责声明
  → emit analysis-done + notification
```

### 5.10 Copilot 追问

```
前端 invoke analysis_followup({ report_id, question })
  → Rust 加载报告与上下文
  → LLM Router 流式调用
  → emit followup-delta / followup-done
  → followups 入库
```

## 6. 错误与降级

- 行情失败：返回 `stale/error` 数据质量，不阻塞页面。
- 金十失败：资讯/日历模块显示错误状态，行情和报告仍可用。
- LLM 失败：报告任务标记失败，保留错误信息和重试入口。
- 工作台聚合失败：尽量返回部分数据，并在对应卡片显示空态或错误态。
- 模拟下单失败：返回拒单原因，如资金不足、超最大手数、合约规则缺失、回放未启动。
- 撮合失败：订单保留为挂起或拒单，并记录错误原因。
- A 股同步失败：保留上次数据，页面显示 `stale/error` 和具体数据源。
- 股票筛选失败：返回字段缺失、报告期不足或数据源错误，不能返回半真半假的排名。
- 流式任务失败：通过 `analysis-error` 或 `followup-error` 推送。

## 7. E2E 通信模式

| 模式 | 说明 |
|---|---|
| UI mock E2E | `VITE_E2E_MOCK=true`，前端 `api/e2e-mock.ts` 模拟 Tauri 响应。 |
| Client live E2E | 启动真实 Tauri 后端，通过测试 HTTP 探针确认 ready，再由浏览器访问 Vite。 |
| Rust 集成测试 | 直接构造 `AppState` 或调用适配器/DB 函数。 |

测试专用 HTTP 探针不属于产品 API，不应被前端生产代码依赖。
