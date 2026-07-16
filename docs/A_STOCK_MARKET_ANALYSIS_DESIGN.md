# A 股市场分析产品规划

> 版本：v0.1 · 2026-07-09
> 定位：A 股市场分析 + 本地数据库 + 模拟组合复盘
> 边界：只做行情、研究、筛选、模拟组合和复盘，不接券商实盘交易
> 交互稿：`docs/A_STOCK_UI_INTERACTION_DESIGN.html`（历史设计稿，CMC 重构后不再作为当前 UI 依据）
> 实现文档：`docs/A_STOCK_FEATURE_IMPLEMENTATION.md`

---

## 1. 参考开源实现

| 项目 | 可借鉴能力 | 本项目取舍 |
|---|---|---|
| AKShare | A 股行情、指数、行业、概念、资金流、财务等公开数据接口覆盖广。 | 作为 A 股 P0 数据主源，保持低频缓存和质量标识。 |
| Tushare / Tushare Pro | 股票、指数、财务、日历、复权、基础资料等标准化接口。 | 作为可选增强源，适合需要 token 的专业用户。 |
| Baostock | 免费 Python API，覆盖 A 股历史行情、财务和宏观等数据。 | 作为历史行情和财报备用源。 |
| Microsoft Qlib | AI-oriented quant 平台，强调数据处理、因子、模型、回测研究流水线。 | 借鉴其“数据集 - 因子 - 模型 - 回测 - 评估”分层，不直接嵌入完整 Python 平台。 |
| RQAlpha / backtrader | 事件驱动回测、账户、订单、绩效分析、可扩展数据源。 | 复用到本地模拟组合与回测语义，仍不接实盘。 |

设计原则：A 股模块不要做“大而全券商终端”，而是做一个适合本地研究的 **市场宽度观察、个股深度解释、财报因子筛选、模拟组合复盘** 工作台。

## 2. 产品目标

1. 覆盖 A 股主要指数、行业板块、概念题材和个股。
2. 提供市场宽度、涨跌家数、涨跌停、成交额、换手、风格轮动等盘面结构指标。
3. 为个股建立“行情 + 技术 + 资金 + 财务 + 公告 + 估值 + 同行业对比”的分析页。
4. 建立本地股票数据库，保存行情、财务、因子、公告、新闻、筛选结果和模拟组合。
5. 支持自选股、股票池、条件筛选、因子打分、组合观察和模拟交易复盘。
6. 使用 LLM 生成个股速览、财报摘要、行业对比、风险清单和复盘总结。

## 3. 市场范围

| 范围 | v1 规划 |
|---|---|
| 交易所 | 上交所、深交所、北交所。 |
| 标的 | A 股普通股、主要宽基指数、行业指数、概念板块。 |
| ETF | P1 支持宽基/行业 ETF 分析，作为指数替代和组合观察工具。 |
| 可转债 | P2 扩展，避免过早引入转股价、溢价率等复杂规则。 |
| 港股/美股 | 暂不进入 A 股 v1，可作为跨市场参考。 |
| 实盘交易 | Out of Scope，不接券商账户、不下真实委托。 |

## 4. 页面规划

| 页面 | 目标 | 关键模块 |
|---|---|---|
| A 股总览 | 看清当天市场结构 | 指数卡、涨跌家数、涨跌停、成交额、行业热力、风格轮动。 |
| 行业/概念 | 看清板块主线 | 行业热力、概念排名、板块成分、领涨领跌、资金流。 |
| 个股工作台 | 解释单只股票 | K 线、技术指标、资金流、财务摘要、公告新闻、估值与同行对比。 |
| 股票筛选器 | 从全市场找候选 | 条件筛选、因子打分、财务过滤、技术形态、结果保存。 |
| 财报中心 | 解读基本面 | 三大报表、盈利能力、成长、现金流、负债、杜邦拆解。 |
| 资金与情绪 | 观察交易结构 | 主力资金、北向资金、龙虎榜、融资融券、涨停梯队。 |
| 模拟组合 | 练习股票组合 | 虚拟资金、买卖、持仓、收益、回撤、仓位、行业暴露。 |
| A 股复盘 | 复盘交易和选股 | 组合归因、个股复盘、错过/错误交易、LLM 总结。 |

## 5. 功能需求

### 5.1 市场总览

| ID | 需求 | 优先级 |
|---|---|---|
| F-A-MKT-01 | 上证、深成、创业板、科创 50、北证 50、沪深 300、中证 500/1000 指数概览 | P0 |
| F-A-MKT-02 | 涨跌家数、涨跌停、炸板、连板、成交额、量能变化 | P0 |
| F-A-MKT-03 | 行业/概念热力图，支持按涨幅、成交额、资金流排序 | P0 |
| F-A-MKT-04 | 风格轮动：大盘/小盘、价值/成长、周期/消费/科技/金融 | P1 |
| F-A-MKT-05 | 盘后市场结构日报 | P1 |

### 5.2 个股工作台

| ID | 需求 | 优先级 |
|---|---|---|
| F-A-STK-01 | 个股基础资料：代码、名称、交易所、行业、总市值、流通市值 | P0 |
| F-A-STK-02 | K 线、多周期、成交量、均线、MACD、RSI、BOLL | P0 |
| F-A-STK-03 | 资金面：主力资金、北向持股/流入、融资融券、龙虎榜 | P1 |
| F-A-STK-04 | 财务摘要：收入、利润、毛利率、ROE、现金流、负债率 | P0 |
| F-A-STK-05 | 估值：PE、PB、PS、股息率、历史分位、同行比较 | P1 |
| F-A-STK-06 | 公告/新闻时间线，自动归因到财报、并购、减持、政策、风险 | P1 |
| F-A-STK-07 | LLM 个股速览：业务、财务、技术、资金、风险、观察点 | P1 |

### 5.3 股票筛选器

| ID | 需求 | 优先级 |
|---|---|---|
| F-A-SCR-01 | 条件筛选：市值、行业、涨跌幅、成交额、换手率、估值 | P0 |
| F-A-SCR-02 | 财务筛选：营收增速、利润增速、ROE、毛利率、经营现金流 | P0 |
| F-A-SCR-03 | 技术筛选：均线多头、突破、回撤、放量、波动收敛 | P1 |
| F-A-SCR-04 | 因子打分：动量、质量、估值、成长、波动、流动性 | P1 |
| F-A-SCR-05 | 保存筛选模板和结果快照 | P0 |

### 5.4 模拟组合与复盘

| ID | 需求 | 优先级 |
|---|---|---|
| F-A-PAPER-01 | A 股虚拟组合账户，与期货模拟账户隔离 | P1 |
| F-A-PAPER-02 | 支持买入、卖出、撤单、成交、持仓和现金 | P1 |
| F-A-PAPER-03 | 支持 T+1、涨跌停、最小交易单位、手续费/印花税/过户费 | P1 |
| F-A-PAPER-04 | 组合收益、回撤、行业暴露、个股贡献 | P1 |
| F-A-PAPER-05 | 组合复盘：买入理由、卖出理由、错误归因、LLM 总结 | P1 |

## 6. 数据源规划

| 数据 | P0 来源 | P1/P2 来源 | 说明 |
|---|---|---|---|
| 股票基础资料 | AKShare | Tushare Pro | 代码、名称、交易所、行业、上市日期。 |
| 实时行情 | AKShare 东方财富/新浪接口 | pytdx / efinance | 只做分析和模拟，不承诺交易级实时。 |
| 历史 K 线 | AKShare / Baostock | Tushare Pro | 日线 P0，分钟线 P1。 |
| 指数行情 | AKShare | Tushare Pro | 宽基、行业、主题指数。 |
| 行业/概念 | AKShare 东方财富/同花顺公开数据 | Tushare Pro | 板块热力和成分股。 |
| 财务报表 | Baostock / AKShare | Tushare Pro | 三大报表、财务指标、业绩快报。 |
| 资金流 | AKShare | Tushare Pro | 主力资金、北向、融资融券、龙虎榜。 |
| 公告新闻 | AKShare / 巨潮公开页 | Tushare Pro | 低频缓存，保留来源。 |

## 7. 本地数据库草案

```sql
CREATE TABLE stock_symbols (
    ts_code TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    exchange TEXT NOT NULL,
    industry TEXT,
    list_date TEXT,
    status TEXT,
    updated_at TEXT NOT NULL
);

CREATE TABLE stock_daily_bars (
    ts_code TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    volume REAL,
    amount REAL,
    adj_factor REAL,
    PRIMARY KEY (ts_code, trade_date)
);

CREATE TABLE stock_financial_metrics (
    ts_code TEXT NOT NULL,
    report_period TEXT NOT NULL,
    revenue REAL,
    net_profit REAL,
    roe REAL,
    gross_margin REAL,
    debt_ratio REAL,
    operating_cash_flow REAL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (ts_code, report_period)
);

CREATE TABLE stock_factor_snapshots (
    ts_code TEXT NOT NULL,
    factor_date TEXT NOT NULL,
    momentum REAL,
    quality REAL,
    valuation REAL,
    growth REAL,
    volatility REAL,
    liquidity REAL,
    score REAL,
    PRIMARY KEY (ts_code, factor_date)
);

CREATE TABLE stock_screen_results (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    criteria_json TEXT NOT NULL,
    result_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE stock_watchlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    symbols_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## 8. 分析维度

| 维度 | 说明 |
|---|---|
| 技术面 | 趋势、均线、成交量、波动、突破/回撤。 |
| 资金面 | 主力资金、北向、龙虎榜、融资融券、成交额。 |
| 基本面 | 收入、利润、ROE、毛利率、现金流、负债。 |
| 估值 | PE、PB、PS、分位数、行业相对估值。 |
| 行业景气 | 行业涨跌、盈利周期、政策、上下游。 |
| 事件驱动 | 公告、财报、减持、并购、监管、舆情。 |
| 风险 | ST/退市风险、商誉、负债、现金流恶化、监管处罚。 |

## 9. UI 导航建议

现有导航可扩展为：

```
总览
期货
  行情 / 模拟盘 / 复盘 / 回放 / 因子 / 资讯 / 日历 / 异动
A股
  A股总览 / 行业概念 / 个股 / 筛选器 / 财报 / 资金情绪 / 模拟组合
报告
数据库
状态
设置
```

若短期不做二级导航，可先新增顶层入口：`A股`，进入后内部 tabs 切换总览、行业、个股、筛选、财报、组合。

## 10. 开发路线

| 阶段 | 内容 | 状态 |
|---|---|---|
| A1 | A 股数据源适配：股票目录、指数、日 K、行业/概念 | 下一阶段 |
| A2 | A 股总览与行业概念页 | 下一阶段 |
| A3 | 个股工作台：K 线、资料、财务摘要、新闻公告 | 下一阶段 |
| A4 | 股票筛选器与因子快照 | 后续 |
| A5 | 财报中心与估值/同行对比 | 后续 |
| A6 | 模拟组合、T+1、费用、组合绩效 | 后续 |
| A7 | Qlib 风格因子研究流水线和回测报告 | 后续 |

## 11. 合规与边界

- A 股模块仅用于行情研究、筛选、模拟组合和本地复盘。
- 不连接券商交易接口，不保存证券账户、交易密码或验证码。
- 模拟组合必须标记为模拟，不与真实持仓混用。
- 数据来自公开或用户授权接口，需保留来源、更新时间和数据质量。
- LLM 输出仅供参考，不构成投资建议。
