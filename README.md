# ThisIsMyQuant

ThisIsMyQuant 是一款面向中国国内期货与 A 股市场的 **模拟盘 + 本地数据库 + 分析复盘工作台**，基于 **Tauri v2 + Rust + React** 构建。项目支持虚拟资金、模拟下单、持仓复盘、行情资讯分析和 LLM 研报，**不连接实盘交易柜台，不发送真实委托，不托管真实账户**。

期货产品规划聚焦五大商品板块：黑色建材、有色贵金属、农产品软商品、能源化工、航运运价。A 股产品规划聚焦指数、行业/概念、个股、财报、资金情绪、股票筛选和模拟组合。v1 不把金融期货纳入默认产品目录、关注列表或批量分析范围；如后续接入股指/国债期货，会作为独立规划处理。

## 核心能力

| 模块 | 说明 |
|---|---|
| 专业分析工作台 | 聚合板块热度、决策流、因子快照、异动信号、报告工作流与外盘联动。 |
| 行情工作台 | 使用主力连续合约展示 K 线、成交量、技术指标和行情状态。 |
| 模拟盘 | 虚拟账户、模拟下单、委托/成交/持仓、保证金、手续费、风险度和资金曲线。 |
| 交易复盘 | 交易日记、计划执行、盈亏归因、品种贡献、LLM 复盘总结。 |
| A 股市场分析 | 指数、行业概念、个股、财报、资金流、筛选器、模拟组合和本地股票数据库。 |
| 因子中心 | 按板块/品种跟踪库存、供需、宏观、外盘、政策、价差等分析因子。 |
| 资讯决策中心 | 接入金十期货资讯与财经日历，按品种和维度分类，形成可追溯的决策流。 |
| 日历与宏观 | 展示重要宏观数据、事件影响和待观察窗口，注入 LLM 分析上下文。 |
| 异动预警中心 | 基于涨跌幅、波动、成交量、数据质量状态生成盘中异动信号。 |
| 报告与 Copilot | 支持手动、短期、明日展望、批量分析、追问和报告归档。 |
| 状态与设置 | 管理 LLM Provider、数据源健康、定时任务、关注列表和本地凭据。 |

## 数据与分析范围

- 行情与 K 线：AKShare / Sina 主力连续合约轮询与历史回填。
- A 股数据：AKShare、Baostock、Tushare Pro 可选，覆盖股票目录、指数、日 K、行业概念、财务、资金流和公告新闻。
- 资讯与日历：金十数据期货分类、快讯和财经日历。
- 外盘参考：Yahoo Finance 等日级数据，用于内外盘联动提示。
- 本地存储：SQLite `data/quant.db`，保存行情、资讯、报告、模拟订单、成交、持仓、资金曲线和复盘记录。
- LLM：OpenAI 兼容协议，支持 Doubao、MiniMax、OpenAI、DeepSeek、Qwen 等 Provider。
- 数据质量：UI 和报告需区分 `live`、`history`、`estimated`、`pending`、`stale`、`error`，避免把估算数据伪装成实时数据。

## 开发环境

前置依赖：Rust stable、Node.js 20+、pnpm 9+。

```bash
pnpm install
bash scripts/sync-env.sh
pnpm tauri:dev
```

常用检查：

```bash
pnpm --dir frontend tsc
pnpm --dir frontend lint
pnpm --dir frontend test
pnpm test:e2e
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

完整推送前检查：

```bash
pnpm test:ci:all
```

`pnpm install` 会安装 pre-push 钩子；`git push` 前默认运行 macOS CI 与 Linux Docker CI。详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 文档索引

- [需求文档](docs/REQUIREMENTS.md)
- [架构设计](docs/ARCHITECTURE.md)
- [通信机制](docs/COMMUNICATION.md)
- [数据源说明](docs/DATA_SOURCES.md)
- [UI 设计语言](docs/DESIGN.md)
- [期货分析设计](docs/FUTURES_ANALYSIS_DESIGN.md)
- [模拟盘设计](docs/SIMULATION_TRADING_DESIGN.md)
- [A 股市场分析设计](docs/A_STOCK_MARKET_ANALYSIS_DESIGN.md)
- [A 股功能实现文档](docs/A_STOCK_FEATURE_IMPLEMENTATION.md)
- [使用方式](docs/USAGE.md)

最新 A 股交互与 UI 设计稿位于 `docs/A_STOCK_UI_INTERACTION_DESIGN.html`，作为本地设计产物已加入 `.gitignore`。
