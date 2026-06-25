# 国内期货多维度分析与资讯分类 — 产品设计

> 版本：v1.0 · 2026-06-25  
> 状态：部分已实现（Rust 核心）  
> 关联：`src-tauri/src/engine/sectors.rs`、`src-tauri/src/engine/analysis.rs`

---

## 1. 目标

1. 为每个流动性合格的期货品种建立 **可配置分析维度**（季节性、天气、海外上游、财报等）。
2. 实时金十资讯 **自动分类** 到维度 + 品种，供 LLM 分析与历史检索。
3. 期货列表 **仅保留高流动性品种**（量化筛选 + 人工白名单）。
4. 分析结果结构化落库，支持按维度回溯与报告生成。

---

## 2. 核心分析因素（通用框架）

| 维度 code | 名称 | 适用说明 | 典型数据源 |
|---|---|---|---|
| `seasonality` | 季节性 | 农产品/能源消费旺季、检修季 | 历史价量、日历事件 |
| `weather` | 天气 | 产区降水/霜冻/干旱、航运干扰 | 气象 API、新闻 |
| `overseas_upstream` | 海外上游 | 原油、LME、CBOT、铁矿发运 | 外盘行情、进口数据 |
| `domestic_supply` | 国内供给 | 开工率、产量、库存、检修 | 产业周报、新闻 |
| `demand` | 需求 | 地产/基建/消费/出口 | 宏观数据、新闻 |
| `inventory` | 库存 | 社会库存、港口库存、仓单 | 库存周报 |
| `spread_arb` | 价差套利 | 跨期/跨品种价差、进口利润 | 行情计算 |
| `policy` | 政策监管 | 收储抛储、环保限产、关税 | 新闻 |
| `macro` | 国内宏观 | 中国 PMI、社融、LPR、政策预期 | 宏观数据、央行、统计局 |
| `overseas_finance` | 国外金融环境 | 美国 CPI/PPI/PCE、非农、美联储/FOMC 利率决策、美债与美元指数 | 金十宏观快讯、海外日历 |
| `geopolitics` | 地缘 | 制裁、航道、战争 | 新闻 |
| `earnings` | 企业财报 | 龙头矿商/钢企/油企业绩指引 | 公告、新闻 |
| `technical` | 技术面 | 趋势、关键位、持仓量 | K 线（已有） |
| `flow` | 资金持仓 | 主力净多净空、仓单变化 | 交易所持仓（后续） |

**板块默认维度权重**（分析 prompt 按此排序）：

| 板块 | 优先维度 |
|---|---|
| 黑色建材 | demand, domestic_supply, inventory, overseas_finance, overseas_upstream, policy |
| 有色贵金属 | macro, overseas_finance, overseas_upstream, inventory, earnings, geopolitics |
| 农产品 | weather, seasonality, overseas_finance, overseas_upstream, inventory, demand |
| 能源化工 | overseas_upstream, overseas_finance, domestic_supply, inventory, spread_arb, policy |
| 航运 | geopolitics, demand, overseas_finance, seasonality, overseas_upstream |
| 金融 | macro, overseas_finance, policy, flow, technical |

---

## 3. 品种级分析维度配置

在 `src-tauri/src/engine/sectors.rs` 与 `dimensions.rs` 维护品种与维度：

```rust
pub struct AnalysisDimension {
    pub code: &'static str,
    pub label: &'static str,
    pub keywords: &'static [&'static str],
    pub prompt_hint: &'static str,
}

pub struct FutureProduct {
    pub code: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub exchange: &'static str,
    pub dimensions: &'static [&'static str],
    pub liquidity_tier: &'static str, // "core" | "watch" | "excluded"
}
```

### 3.1 示例：螺纹钢 RB0

| 维度 | 关键词示例 |
|---|---|
| demand | 地产、基建、开工率、销售面积 |
| domestic_supply | 铁水、高炉、钢厂、减产 |
| inventory | 社会库存、厂库、五大材 |
| overseas_upstream | 铁矿、普氏、发运、澳洲、巴西 |
| policy | 环保限产、平控、碳排放 |
| spread_arb | 卷螺差、期现基差 |

### 3.2 示例：豆粕 M0

| 维度 | 关键词示例 |
|---|---|
| weather | 美国中西部、巴西降雨、干旱 |
| seasonality | 种植进度、收割、压榨旺季 |
| overseas_upstream | CBOT、美豆、巴西大豆、到港 |
| inventory | 港口库存、油厂库存 |
| demand | 养殖、生猪、饲料 |
| spread_arb | 豆菜粕价差、进口大豆榨利 |

完整品种表见附录 A（基于现有 40 品种裁剪后约 **32 个 core**）。

---

## 4. 流动性筛选（仅保留高流动性品种）

### 4.1 规则

每日收盘后（或启动时）对 `*0` 主力连续计算 **20 日滚动指标**：

```
liquidity_score = 0.5 * norm(volume_20d) + 0.5 * norm(oi_proxy_20d)
```

- `volume_20d`：20 日日均成交量（手）
- `oi_proxy_20d`：若无持仓数据，用 `volume * turnover` 或最近日 K `volume` 峰值代理

**准入门槛（建议初值，可配置）：**

| 指标 | 阈值 |
|---|---|
| 20 日日均成交量 | ≥ 5,000 手（金融/贵金属可单独设） |
| 20 日日均成交额 | ≥ 5 亿元（小合约如 TS 除外） |
| 连续 5 日零成交 | 自动 excluded |

**Tier：**

- `core`：通过门槛 → 出现在 UI 列表、默认 watchlist 候选
- `watch`：边缘流动性 → 可搜索但不默认展示
- `excluded`：不展示、不分析

### 4.2 建议保留 core 列表（32）

剔除流动性长期偏弱或高度重叠品种：**FG0 玻璃、SA0 纯碱、AO0 氧化铝、UR0 尿素、TF0/TS0 短端国债** 等进入 watch 或 excluded（以实际回测为准）。

### 4.3 实现位置

```
src-tauri/src/engine/liquidity.rs     # 计算 + tier
src-tauri/src/services/liquidity_job.rs  # 日更任务
contracts 表增加 liquidity_tier, liquidity_score, scored_at
```

---

## 5. 资讯分类 pipeline

```
金十 poll → news_items 入库 → 规则分类 → (低置信) LLM 分类 → news_dimensions 关联
                                    ↓
                            分析时按 symbol + dimension 取 Top-N
```

### 5.1 规则分类（第一层，零成本）

```rust
fn classify_rule(news: &NewsItem, product: &FutureProduct) -> Vec<(Dimension, f32)> {
    // 1. 板块 jin10 category_id 命中 → +0.3
    // 2. 品种 keywords 命中 title/summary → +0.5
    // 3. 各 dimension.keywords 命中 → +0.4 each
    // 返回 score >= 0.5 的维度
}
```

### 5.2 LLM 分类（第二层，批量）

对规则未命中或 multi-label 的资讯，每 5 分钟批量调用：

**System：** 你是期货资讯分类器，只输出 JSON。

**User：**
```json
{
  "news": {"id": "...", "title": "...", "summary": "..."},
  "candidates": [
    {"symbol": "RB0", "dimensions": ["demand", "inventory", ...]},
    ...
  ]
}
```

**Output schema：**
```json
{
  "labels": [
    {"symbol": "RB0", "dimension": "demand", "confidence": 0.85, "reason": "..."}
  ]
}
```

- 模型：`doubao-lite` / 小模型即可，temperature=0
- 批量 size：10 条/请求

### 5.3 去重

- `content_hash = sha256(normalize(title + summary))`
- 24h 内重复 hash 跳过

---

## 6. 数据库设计

### 6.1 新增表

```sql
-- 维度字典
CREATE TABLE analysis_dimensions (
    code TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    description TEXT
);

-- 品种-维度配置（也可继续放 sectors 代码里，DB 做 override）
CREATE TABLE product_dimensions (
    symbol TEXT NOT NULL,
    dimension_code TEXT NOT NULL,
    keywords TEXT,           -- JSON array
    prompt_hint TEXT,
    PRIMARY KEY (symbol, dimension_code)
);

-- 资讯持久化
CREATE TABLE news_items (
    id TEXT PRIMARY KEY,
    source TEXT DEFAULT 'jin10',
    category_id INTEGER,
    title TEXT NOT NULL,
    summary TEXT,
    url TEXT,
    display_time TEXT NOT NULL,
    content_hash TEXT UNIQUE,
    raw_json TEXT,
    ingested_at TEXT NOT NULL
);
CREATE INDEX idx_news_time ON news_items(display_time);
CREATE INDEX idx_news_hash ON news_items(content_hash);

-- 资讯分类结果
CREATE TABLE news_classifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    news_id TEXT NOT NULL REFERENCES news_items(id),
    symbol TEXT NOT NULL,
    dimension_code TEXT NOT NULL,
    confidence REAL NOT NULL,
    method TEXT NOT NULL,    -- 'rule' | 'llm'
    created_at TEXT NOT NULL,
    UNIQUE(news_id, symbol, dimension_code)
);
CREATE INDEX idx_nc_symbol_dim ON news_classifications(symbol, dimension_code, created_at);

-- 流动性快照
CREATE TABLE liquidity_snapshots (
    symbol TEXT NOT NULL,
    scored_at TEXT NOT NULL,
    volume_20d REAL,
    turnover_20d REAL,
    score REAL,
    tier TEXT,
    PRIMARY KEY (symbol, scored_at)
);

-- 维度事实摘录（LLM 从资讯/分析中抽取的结构化要点，可选 Phase 2）
CREATE TABLE dimension_facts (
    id TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    dimension_code TEXT NOT NULL,
    fact TEXT NOT NULL,
    source_news_id TEXT,
    source_report_id TEXT,
    valid_until TEXT,
    created_at TEXT NOT NULL
);
```

### 6.2 reports 表扩展

```sql
ALTER TABLE reports ADD COLUMN dimension_summary TEXT;  -- JSON: {dimension: bullet_points}
ALTER TABLE reports ADD COLUMN news_ids TEXT;           -- JSON array，引用的资讯 id
```

---

## 7. LLM 通信设计

### 7.1 三阶段调用

| 阶段 | 触发 | 模型 | 输出 |
|---|---|---|---|
| **C1 资讯分类** | news_poll 每批 | 小模型 | JSON labels |
| **C2 维度摘要** | 分析前 | 中模型 | 每维度 2-3 条要点 JSON |
| **C3 完整报告** | daily/realtime/manual | 主模型 | Markdown 报告 + dimension_summary JSON |

### 7.2 C3 Prompt 结构（升级现有 `render_prompt`）

```
[System] 现有 SYSTEM_PROMPT + 输出格式要求

[User]
## 品种与维度
{product_name} ({symbol}) — 请按以下维度逐一分析：
- [demand] 需求：...
- [inventory] 库存：...

## 技术面（60 日 K）
{indicator block — 已有}

## 分维度资讯（近 24h，已分类）
### demand
1. [2026-06-25] 标题 — 摘要 (conf 0.9)
### inventory
...

## 输出要求
1. 先输出 ```json dimension_summary``` 块
2. 再输出 Markdown 报告，按维度分节
```

### 7.3 流式协议（保持现有 Tauri Event）

```
stream_analysis(symbol, trigger)
  → analysis-delta { text }
  → analysis-done { report_id, dimension_summary }
  → analysis-error { message }
```

解析：流结束后从 content 中提取 ` ```json ` 块写入 `reports.dimension_summary`。

### 7.4 Copilot 追问（Phase 2）

`AiPanel` textarea → `invoke("analysis_followup", { report_id, question })`  
上下文 = 原 report + 最近 dimension_facts + 相关 news。

---

## 8. 服务架构

```
┌─────────────┐     poll      ┌──────────────┐
│ Jinshi API  │ ────────────► │ NewsIngest   │──► news_items
└─────────────┘               └──────┬───────┘
                                     │
                              ┌──────▼───────┐
                              │ NewsClassifier│──► news_classifications
                              │ rule + llm   │
                              └──────┬───────┘
                                     │
┌─────────────┐   daily cron  ┌──────▼───────┐     stream     ┌────────┐
│ AKShare K线 │ ────────────► │ AnalysisRunner│ ─────────────►│ LLM    │
└─────────────┘               │ build_context │                └────────┘
                              │ + dim news    │──► reports
                              └──────────────┘

LiquidityJob (daily) ──► liquidity_snapshots ──► filter UI product list
```

### 8.1 新增 Rust 模块

| 模块 | 职责 |
|---|---|
| `engine/dimensions.rs` | 维度字典 + 品种配置 |
| `engine/liquidity.rs` | 流动性打分 |
| `services/news_ingest.rs` | 入库 + 去重 |
| `services/news_classifier.rs` | 规则 + LLM 分类 |
| `engine/analysis.rs` | 扩展 build_context：按维度聚合资讯 |

### 8.2 Tauri Commands

| Command | 说明 |
|---|---|
| `list_products` | 仅 `core` tier，含 dimensions |
| `list_news_by_dimension` | symbol + dimension + limit |
| `get_liquidity_ranks` | 排行榜/筛选依据 |
| `reclassify_news` | 手动触发重分类（调试） |

---

## 9. 前端变更

| 页面 | 变更 |
|---|---|
| SymbolsPage | 仅展示 core；显示 liquidity_score |
| AiPanel | 报告按维度折叠展示；Copilot 接 followup |
| 新增 NewsPanel | 按维度筛选资讯流（可选） |
| `@/data/futures` | 从 `list_products` 动态加载，替代静态缺失文件 |

---

## 10. 实施分期

| 阶段 | 内容 | 预估 |
|---|---|---|
| **P0** | 流动性筛选 + core 列表 + `list_products` + 补 futures.ts | 2d |
| **P1** | news_items 入库 + 规则分类 + news_classifications | 3d |
| **P2** | LLM 分类 + 分析 prompt 按维度重组 | 3d |
| **P3** | dimension_summary 落库 + 前端维度 UI | 2d |
| **P4** | dimension_facts + Copilot followup | 3d |

---

## 附录 A：建议 core 品种（32）

| 板块 | core 品种 |
|---|---|
| 黑色 | RB0 HC0 I0 J0 JM0 |
| 有色 | CU0 AL0 ZN0 NI0 AU0 AG0 LC0 |
| 农产 | M0 Y0 P0 C0 SR0 CF0 LH0 |
| 能化 | SC0 FU0 BU0 TA0 MA0 PP0 RU0 |
| 航运 | EC0 |
| 金融 | IF0 IH0 IC0 IM0 T0 TL0 |

watch：AP0 FG0 SA0 AO0 UR0 TF0 TS0

---

## 附录 B：配置项（.env）

```env
LIQUIDITY_MIN_VOLUME_20D=5000
LIQUIDITY_MIN_TURNOVER_20D=5e8
NEWS_CLASSIFY_LLM=doubao-lite
NEWS_CLASSIFY_BATCH=10
NEWS_RETENTION_DAYS=30
```
