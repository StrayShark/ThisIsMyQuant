# A 股功能实现文档

> 版本：v0.1 · 2026-07-09  
> 依据：`docs/A_STOCK_MARKET_ANALYSIS_DESIGN.md`、`docs/A_STOCK_UI_INTERACTION_DESIGN.html`  
> 目标：把 A 股市场分析、股票筛选、本地数据库和模拟组合拆解为可实施的工程任务  
> 边界：只做研究、筛选、模拟组合和本地复盘，不接券商实盘交易

---

## 1. 实施目标

A 股功能不是一个独立应用，而是 ThisIsMyQuant 在现有“模拟盘 + 本地数据库 + 分析复盘工作台”上的新市场域。实现后用户应能完成：

```text
A 股总览 → 行业/概念下钻 → 个股工作台 → 股票筛选器 → 模拟组合 → 复盘与 LLM 总结
```

首阶段目标：

1. 建立 A 股本地数据库：股票目录、指数行情、日 K、行业/概念、财务摘要、因子快照、筛选结果。
2. 新增 A 股总览页：指数、市场宽度、涨跌停、成交额、行业/概念热力。
3. 新增行业/概念页：板块排行、成分股、领涨领跌、资金流、相关新闻。
4. 新增个股工作台：K 线、基础资料、财务摘要、估值、资金面、公告新闻、同行比较。
5. 新增股票筛选器：条件构建、结果表、模板保存、股票池快照。
6. 为后续模拟组合预留账户、订单、持仓和组合绩效接口。

---

## 2. 总体架构

### 2.1 模块边界

```text
adapters/stock_data.rs
  → services/stock_data_sync.rs
  → db/sqlite.rs stock_* tables
  → engine/stock_factors.rs
  → commands/stock.rs
  → frontend/src/pages/AStockPage.tsx
```

| 层 | 新增/扩展 | 职责 |
|---|---|---|
| Adapter | `adapters/stock_data.rs` | 封装 AKShare/Baostock/Tushare 可选数据源。 |
| Service | `services/stock_data_sync.rs` | 数据同步、缓存、任务进度、错误降级。 |
| Engine | `engine/stock_factors.rs` | 技术、财务、估值、资金、质量因子计算。 |
| DB | `db/sqlite.rs` | 创建 stock 表、读写股票数据和筛选快照。 |
| Commands | `commands/stock.rs` | A 股总览、个股详情、筛选器、财务查询。 |
| Frontend API | `frontend/src/api/client.ts` | 类型化 invoke 封装。 |
| Frontend Page | `frontend/src/pages/AStockPage.tsx` | A 股总览、行业、个股、筛选、财报、组合 tabs。 |

### 2.2 路由设计

短期采用单页多 tab，降低侧栏复杂度：

| 路由 | 页面 | 说明 |
|---|---|---|
| `/stocks` | A 股 | 默认进入 A 股总览。 |
| `/stocks?tab=industry` | 行业/概念 | 行业热力、概念排行和成分股。 |
| `/stocks?tab=detail&symbol=600000.SH` | 个股 | 个股工作台。 |
| `/stocks?tab=screener` | 筛选器 | 条件筛选和结果快照。 |
| `/stocks?tab=financials` | 财报 | 财报中心。 |
| `/stocks?tab=portfolio` | 模拟组合 | A 股纸面组合，后续阶段实现。 |

后续若 A 股功能变重，再拆成独立路由。

---

## 3. 数据模型

### 3.1 股票目录

```sql
CREATE TABLE IF NOT EXISTS stock_symbols (
    ts_code TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    exchange TEXT NOT NULL,
    market TEXT,
    industry TEXT,
    list_date TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_stock_symbols_industry ON stock_symbols(industry);
CREATE INDEX IF NOT EXISTS idx_stock_symbols_name ON stock_symbols(name);
```

字段约定：

| 字段 | 说明 |
|---|---|
| `ts_code` | 标准代码，如 `600000.SH`、`000001.SZ`、`BJxxxx.BJ`。 |
| `symbol` | 交易所原始代码，如 `600000`。 |
| `exchange` | `SH` / `SZ` / `BJ`。 |
| `industry` | 当前行业口径，来源可配置。 |
| `source` | `akshare` / `baostock` / `tushare`。 |

### 3.2 日 K 与指数 K 线

```sql
CREATE TABLE IF NOT EXISTS stock_daily_bars (
    ts_code TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    pre_close REAL,
    pct_chg REAL,
    volume REAL,
    amount REAL,
    turnover_rate REAL,
    adj_factor REAL,
    adjustment TEXT NOT NULL DEFAULT 'none',
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (ts_code, trade_date, adjustment)
);
CREATE INDEX IF NOT EXISTS idx_stock_daily_bars_date ON stock_daily_bars(trade_date);

CREATE TABLE IF NOT EXISTS stock_index_daily_bars (
    index_code TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    open REAL,
    high REAL,
    low REAL,
    close REAL,
    pct_chg REAL,
    volume REAL,
    amount REAL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (index_code, trade_date)
);
```

实现要求：

- P0 只做日线，P1 再考虑分钟线。
- 所有图表必须显示复权口径：`none`、`qfq`、`hfq`。
- 筛选器使用的行情日期必须写入结果快照。

### 3.3 行业/概念

```sql
CREATE TABLE IF NOT EXISTS stock_boards (
    board_code TEXT PRIMARY KEY,
    board_name TEXT NOT NULL,
    board_type TEXT NOT NULL, -- industry | concept | style
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stock_board_members (
    board_code TEXT NOT NULL,
    ts_code TEXT NOT NULL,
    weight REAL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (board_code, ts_code)
);

CREATE TABLE IF NOT EXISTS stock_board_snapshots (
    board_code TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    pct_chg REAL,
    amount REAL,
    turnover_rate REAL,
    net_flow REAL,
    up_count INTEGER,
    down_count INTEGER,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (board_code, trade_date)
);
```

### 3.4 财务与估值

```sql
CREATE TABLE IF NOT EXISTS stock_financial_metrics (
    ts_code TEXT NOT NULL,
    report_period TEXT NOT NULL,
    report_type TEXT,
    revenue REAL,
    revenue_yoy REAL,
    net_profit REAL,
    net_profit_yoy REAL,
    roe REAL,
    gross_margin REAL,
    debt_ratio REAL,
    operating_cash_flow REAL,
    eps REAL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (ts_code, report_period)
);

CREATE TABLE IF NOT EXISTS stock_valuation_snapshots (
    ts_code TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    pe_ttm REAL,
    pb REAL,
    ps_ttm REAL,
    dividend_yield REAL,
    market_cap REAL,
    float_market_cap REAL,
    pe_percentile REAL,
    pb_percentile REAL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (ts_code, trade_date)
);
```

要求：

- 财务指标必须显示 `report_period`。
- 估值必须显示口径：TTM、静态或源站默认。
- 缺少财务数据时 UI 使用 `pending`，不能用 0 代替。

### 3.5 因子快照与筛选结果

```sql
CREATE TABLE IF NOT EXISTS stock_factor_snapshots (
    ts_code TEXT NOT NULL,
    factor_date TEXT NOT NULL,
    momentum REAL,
    quality REAL,
    valuation REAL,
    growth REAL,
    volatility REAL,
    liquidity REAL,
    capital_flow REAL,
    score REAL,
    factor_version TEXT NOT NULL,
    source TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (ts_code, factor_date, factor_version)
);

CREATE TABLE IF NOT EXISTS stock_screen_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    criteria_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stock_screen_results (
    id TEXT PRIMARY KEY,
    template_id TEXT,
    name TEXT NOT NULL,
    criteria_json TEXT NOT NULL,
    result_json TEXT NOT NULL,
    trade_date TEXT,
    report_period TEXT,
    source_summary TEXT,
    created_at TEXT NOT NULL
);
```

### 3.6 自选股与股票池

```sql
CREATE TABLE IF NOT EXISTS stock_watchlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    symbols_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

## 4. 后端实现

### 4.1 Adapter trait

```rust
#[async_trait]
pub trait StockDataProvider: Send + Sync {
    async fn list_symbols(&self) -> AppResult<Vec<StockSymbol>>;
    async fn list_index_bars(&self, req: StockBarsRequest) -> AppResult<Vec<StockBar>>;
    async fn list_stock_bars(&self, req: StockBarsRequest) -> AppResult<Vec<StockBar>>;
    async fn list_boards(&self) -> AppResult<Vec<StockBoard>>;
    async fn list_board_members(&self, board_code: &str) -> AppResult<Vec<StockBoardMember>>;
    async fn list_financial_metrics(&self, ts_code: &str) -> AppResult<Vec<StockFinancialMetric>>;
    async fn list_valuation_snapshots(&self, ts_code: &str) -> AppResult<Vec<StockValuationSnapshot>>;
}
```

P0 可先实现 `AkshareStockProvider`，保留 `BaostockStockProvider` 和 `TushareStockProvider` 的配置位。

### 4.2 同步服务

`services/stock_data_sync.rs` 负责：

| 方法 | 说明 |
|---|---|
| `sync_symbols()` | 同步股票目录。 |
| `sync_indices()` | 同步主要指数日 K。 |
| `sync_daily_bars(symbols, range)` | 同步个股日 K。 |
| `sync_boards()` | 同步行业/概念与成分股。 |
| `sync_financials(symbols)` | 同步财务指标。 |
| `sync_market_snapshot()` | 生成市场宽度、涨跌停、行业热力快照。 |

同步策略：

1. 启动时只同步轻量元数据，不阻塞首屏。
2. 用户进入 A 股页时触发 `get_a_stock_dashboard`，优先读缓存，必要时后台刷新。
3. 数据同步通过 `stock-sync-progress` 事件推送进度。
4. 任一数据源失败时保留旧数据，并返回 `stale/error` 状态。

### 4.3 Command 清单

新增 `src-tauri/src/commands/stock.rs`：

| Command | 入参 | 出参 | P0/P1 |
|---|---|---|---|
| `list_stock_symbols` | `{ query?, industry?, limit? }` | `Vec<StockSymbol>` | P0 |
| `get_a_stock_dashboard` | `{ trade_date? }` | `AStockDashboardView` | P0 |
| `get_stock_klines` | `{ ts_code, interval, adjustment, limit }` | `Vec<StockBar>` | P0 |
| `get_stock_detail` | `{ ts_code }` | `StockDetailView` | P0 |
| `list_stock_industries` | `{ board_type? }` | `Vec<StockBoardView>` | P0 |
| `get_stock_industry_detail` | `{ board_code }` | `StockBoardDetailView` | P0 |
| `run_stock_screener` | `StockScreenerCriteria` | `StockScreenResultView` | P0 |
| `save_stock_screen` | `{ name, criteria, result? }` | `StockScreenTemplate` | P0 |
| `list_stock_financials` | `{ ts_code }` | `Vec<StockFinancialMetric>` | P1 |
| `list_stock_factor_snapshots` | `{ ts_code?, date? }` | `Vec<StockFactorSnapshot>` | P1 |
| `trigger_stock_data_sync` | `{ scope, symbols? }` | `TaskStatus` | P0 |

### 4.4 视图模型

```rust
pub struct AStockDashboardView {
    pub trade_date: String,
    pub data_quality: DataQuality,
    pub indices: Vec<StockIndexCard>,
    pub breadth: MarketBreadthView,
    pub boards: Vec<StockBoardHeatItem>,
    pub limit_up_down: LimitUpDownView,
    pub mainlines: Vec<MarketMainlineItem>,
    pub portfolio: Option<StockPaperPortfolioSummary>,
}

pub struct StockDetailView {
    pub symbol: StockSymbol,
    pub latest_bar: Option<StockBar>,
    pub financial: Option<StockFinancialMetric>,
    pub valuation: Option<StockValuationSnapshot>,
    pub factors: Option<StockFactorSnapshot>,
    pub peer_comparison: Vec<StockPeerItem>,
    pub events: Vec<StockEventItem>,
    pub data_quality: DataQuality,
}

pub struct StockScreenerCriteria {
    pub market_cap_min: Option<f64>,
    pub market_cap_max: Option<f64>,
    pub industries: Vec<String>,
    pub pct_chg_min: Option<f64>,
    pub amount_min: Option<f64>,
    pub pe_max: Option<f64>,
    pub roe_min: Option<f64>,
    pub revenue_yoy_min: Option<f64>,
    pub net_profit_yoy_min: Option<f64>,
    pub technical_flags: Vec<String>,
}
```

---

## 5. 因子与筛选逻辑

### 5.1 P0 筛选条件

| 条件组 | 字段 | 说明 |
|---|---|---|
| 基础 | 市场、行业、市值、上市状态 | 从 `stock_symbols` 和估值表读取。 |
| 行情 | 涨跌幅、成交额、换手率、近 N 日涨幅 | 从 `stock_daily_bars` 计算。 |
| 估值 | PE、PB、PS、股息率 | 从估值快照读取。 |
| 财务 | ROE、毛利率、营收增速、利润增速、经营现金流 | 从财务指标读取。 |
| 技术 | 均线多头、突破 N 日高点、回撤幅度 | 从日 K 计算。 |

筛选结果必须包含：

- 命中条件摘要。
- 使用的交易日。
- 使用的最新报告期。
- 数据源摘要。
- 数据质量状态。

### 5.2 因子打分

P1 开始实现打分：

```text
score = 0.20 * momentum
      + 0.25 * quality
      + 0.20 * valuation
      + 0.20 * growth
      + 0.10 * liquidity
      + 0.05 * capital_flow
```

因子版本写入 `factor_version`，方便后续回溯。

---

## 6. 前端实现

### 6.1 文件结构

```text
frontend/src/
├── pages/
│   └── AStockPage.tsx
├── features/stocks/
│   ├── AStockDashboard.tsx
│   ├── StockIndustryView.tsx
│   ├── StockDetailWorkspace.tsx
│   ├── StockScreener.tsx
│   ├── StockFinancialCenter.tsx
│   ├── StockPaperPortfolio.tsx
│   ├── components/
│   │   ├── MarketBreadthStrip.tsx
│   │   ├── IndexQuoteCard.tsx
│   │   ├── IndustryHeatmap.tsx
│   │   ├── StockHeader.tsx
│   │   ├── FinancialSnapshot.tsx
│   │   ├── StockScreenerBuilder.tsx
│   │   └── ScreenResultTable.tsx
│   └── types.ts
```

短期也可以先把组件放在 `pages/AStockPage.tsx` 中，等功能稳定后拆分。

### 6.2 A 股页 tabs

| Tab | 组件 | 数据 |
|---|---|---|
| 总览 | `AStockDashboard` | `get_a_stock_dashboard` |
| 行业概念 | `StockIndustryView` | `list_stock_industries`、`get_stock_industry_detail` |
| 个股 | `StockDetailWorkspace` | `get_stock_detail`、`get_stock_klines` |
| 筛选器 | `StockScreener` | `run_stock_screener`、`save_stock_screen` |
| 财报 | `StockFinancialCenter` | `list_stock_financials` |
| 模拟组合 | `StockPaperPortfolio` | P1 实现 |

### 6.3 关键交互

#### 行业热力下钻

1. 用户点击总览页行业热力块。
2. 前端切换到 `tab=industry&board=<code>`。
3. 调用 `get_stock_industry_detail`。
4. 成分股表、板块 K 线、资金流和事件列表同步更新。

#### 个股搜索

1. 顶部搜索输入代码、中文名或拼音首字母。
2. 调用 `list_stock_symbols({ query })`。
3. 选择个股后进入 `tab=detail&symbol=<ts_code>`。
4. 并行加载 `get_stock_detail` 和 `get_stock_klines`。

#### 筛选器

1. 用户在左侧设置条件。
2. 点击“运行筛选”调用 `run_stock_screener`。
3. 结果表显示命中原因、分数、报告期、数据源。
4. 用户可保存模板或保存结果为股票池。

### 6.4 UI 状态

| 状态 | 展示 |
|---|---|
| Loading | 表格骨架、热力图 skeleton、按钮 disabled。 |
| Empty | 说明缺少数据源、报告期或筛选条件过严。 |
| Stale | 保留旧数据，显示旧数据时间和同步失败原因。 |
| Error | 显示数据源、错误摘要、重试按钮。 |
| Simulation | 模拟组合所有交易和持仓显示“模拟”。 |

---

## 7. A 股模拟组合设计（P1）

股票模拟组合和期货模拟盘共用“虚拟账户/订单/成交/持仓/资金曲线”的思想，但规则不同。

### 7.1 规则

| 规则 | A 股模拟组合 |
|---|---|
| 最小交易单位 | 100 股。 |
| 买入卖出 | 普通股 T+1，买入当日不可卖出。 |
| 涨跌停 | 普通股票 10%，创业板/科创板 20%，ST 5%，北交所按规则配置。 |
| 费用 | 佣金、印花税、过户费，可配置。 |
| 现金 | 买入时扣现金，卖出时增加现金。 |

### 7.2 数据表

可新建独立表，避免和期货 `sim_orders` 混淆：

```sql
CREATE TABLE IF NOT EXISTS stock_paper_accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    initial_cash REAL NOT NULL,
    cash REAL NOT NULL,
    equity REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stock_paper_orders (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    ts_code TEXT NOT NULL,
    side TEXT NOT NULL, -- buy | sell
    order_type TEXT NOT NULL,
    price REAL,
    quantity INTEGER NOT NULL,
    filled_quantity INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stock_paper_positions (
    account_id TEXT NOT NULL,
    ts_code TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    available_quantity INTEGER NOT NULL,
    avg_price REAL NOT NULL,
    market_value REAL NOT NULL,
    unrealized_pnl REAL NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (account_id, ts_code)
);
```

P1 不要求实时撮合，先按最新行情或回放日 K 的 close 成交，并明确标记模拟。

---

## 8. LLM 能力

### 8.1 个股速览 Prompt 输入

```text
个股基础资料：
{name, ts_code, industry, market_cap}

价格与技术：
{latest_bar, 20d_return, 60d_return, volume_change, trend}

财务：
{revenue_yoy, net_profit_yoy, roe, gross_margin, debt_ratio, operating_cash_flow, report_period}

估值：
{pe_ttm, pb, percentile, peer_comparison}

事件：
{announcements, news, abnormal_flow}

要求：
输出业务概览、短期价格行为、财务质量、估值位置、风险清单、观察点。
必须注明数据报告期和数据源。
```

### 8.2 筛选结果总结

对筛选结果生成：

- 共同特征。
- 行业集中度。
- 风险暴露。
- 需要二次排除的样本。
- 可加入模拟组合观察的理由。

LLM 输出必须附免责声明。

---

## 9. 测试计划

### 9.1 Rust 测试

| 测试 | 内容 |
|---|---|
| `stock_symbols_roundtrip` | 股票目录写入/查询。 |
| `stock_daily_bars_roundtrip` | 日 K 多复权口径写入/查询。 |
| `stock_financial_metrics_roundtrip` | 财务指标按报告期查询。 |
| `stock_screener_filters` | 市值、行业、ROE、PE、涨跌幅筛选。 |
| `stock_factor_scoring` | 因子打分稳定性。 |

### 9.2 前端测试

| 测试 | 内容 |
|---|---|
| A 股总览 mock E2E | 页面可访问，指数卡、市场宽度、行业热力存在。 |
| 个股页 mock E2E | 搜索并进入个股，K 线区、财务摘要、资金面存在。 |
| 筛选器 mock E2E | 配置条件、运行筛选、保存模板。 |
| 状态测试 | empty/stale/error 数据质量展示。 |

### 9.3 验收命令

```bash
pnpm --dir frontend tsc
pnpm --dir frontend lint
pnpm --dir frontend test
pnpm test:e2e
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

---

## 10. 分阶段任务清单

### A1：数据源与数据库

| 任务 | 文件 | 验收 |
|---|---|---|
| 新增 stock DTO | `src-tauri/src/models.rs` | 类型可序列化到前端。 |
| 新增 stock schema | `src-tauri/src/db/sqlite.rs` | 启动自动建表。 |
| 新增 stock adapter | `src-tauri/src/adapters/stock_data.rs` | mock/AKShare 数据可返回。 |
| 新增 sync service | `src-tauri/src/services/stock_data_sync.rs` | 可同步目录和日 K。 |
| 注册命令模块 | `src-tauri/src/commands/stock.rs`、`mod.rs` | `get_a_stock_dashboard` 可调用。 |

### A2：A 股总览与行业

| 任务 | 文件 | 验收 |
|---|---|---|
| API client | `frontend/src/api/client.ts` | 类型化封装完成。 |
| Mock 数据 | `frontend/src/api/e2e-mock.ts` | E2E 可跑。 |
| A 股页路由 | `frontend/src/App.tsx`、`AppShell.tsx` | 侧栏出现 A 股入口。 |
| 总览 UI | `frontend/src/pages/AStockPage.tsx` | 指数、宽度、热力图显示。 |
| 行业详情 | `AStockPage.tsx` 或 `features/stocks` | 点击行业可下钻。 |

### A3：个股与筛选器

| 任务 | 文件 | 验收 |
|---|---|---|
| 个股详情命令 | `commands/stock.rs` | 返回 `StockDetailView`。 |
| K 线查询 | `get_stock_klines` | 图表可展示日 K。 |
| 个股工作台 UI | `AStockPage.tsx` | 展示价格、财务、估值、资金面。 |
| 筛选器 engine | `engine/stock_factors.rs` | 条件过滤正确。 |
| 筛选器 UI | `StockScreener` | 可运行、保存模板和结果。 |

### A4：模拟组合与 LLM

| 任务 | 文件 | 验收 |
|---|---|---|
| 组合表 | `db/sqlite.rs` | 账户、订单、持仓表可用。 |
| 组合 commands | `commands/stock.rs` | 买卖、持仓、权益可查询。 |
| 模拟组合 UI | `StockPaperPortfolio` | T+1、费用、模拟标识可见。 |
| 个股速览 | `commands/analysis.rs` 或 `stock.rs` | LLM 输出含引用和免责声明。 |
| 组合复盘 | `StockPaperPortfolio` / `Reports` | 组合归因可生成。 |

---

## 11. 关键验收标准

1. A 股模块有独立入口，不破坏期货模拟盘现有流程。
2. A 股总览能展示指数、市场宽度、行业/概念热力和数据更新时间。
3. 个股工作台展示股票代码、交易所、行业、K 线、财务报告期、估值口径和数据源。
4. 股票筛选结果包含命中原因、报告期、快照时间和数据质量。
5. 本地数据库保存 A 股核心数据，可重复打开应用后读取。
6. A 股模拟组合必须标记为模拟，不接券商实盘，不保存证券账户信息。
7. LLM 个股速览和复盘报告必须包含免责声明。
