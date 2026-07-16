# ThisIsMyQuant

ThisIsMyQuant 是一款面向中国国内期货与 A 股市场的 **模拟盘 + 本地数据库 + 分析复盘工作台**，基于 **Tauri v2 + Rust + React** 构建。项目支持虚拟资金、模拟下单、持仓复盘、行情资讯分析和 LLM 研报，**不连接实盘交易柜台，不发送真实委托，不托管真实账户**。

期货产品规划聚焦五大商品板块：黑色建材、有色贵金属、农产品软商品、能源化工、航运运价。A 股产品规划聚焦指数、行业/概念、个股、财报、资金情绪、股票筛选和模拟组合。v1 不把金融期货纳入默认产品目录、关注列表或批量分析范围；如后续接入股指/国债期货，会作为独立规划处理。

## 核心能力

| 模块 | 说明 |
|---|---|
| 总览 | 聚合五大期货板块、模拟账户、决策流、报告任务和数据源状态。 |
| 市场 | 统一发现期货与 A 股资产，支持筛选、排序、排行榜、自选和详情跳转。 |
| 标的详情 | 展示 K 线、关键指标、相关事件、模拟持仓和引用式 AI 摘要。 |
| 自选 | 管理关注列表、分组、备注、提醒和自选异动。 |
| 基础模拟盘 | 虚拟账户、市价/限价下单、委托/成交/持仓、资金流水、保证金和风险度。 |
| 事件资讯 | 聚合金十资讯、财经日历、公告、财报和异动事件。 |
| 本地数据库 | 管理行情、资讯、日历、报告、模拟交易、自选和设置等本地数据域。 |
| 引用式 AI | 基于本地行情、事件、报告和模拟持仓生成带来源、日期与免责声明的分析。 |
| 状态与设置 | 管理 LLM Provider、数据源健康、定时任务、实验区功能和本地凭据。 |

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

最新 CMC/Coinbase 风格视觉设计稿位于 `docs/CMC_PRODUCT_VISUAL_DESIGN.html`，作为本地设计产物已加入 `.gitignore`。
