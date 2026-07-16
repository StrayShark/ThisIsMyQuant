# 国内期货模拟盘与本地数据库设计

> 版本：v0.3 · 2026-07-10
> 定位：国内期货模拟盘 + 本地数据库 + 分析复盘工作台
> 边界：仅虚拟资金和本地撮合，不连接实盘柜台，不提供真实下单能力
> 状态：已按 CMC 重构收敛为基础模拟盘优先；高级订单与回放训练为设置页实验区功能，不作为主业务入口。

---

## 1. 设计参考

主流模拟盘工具的能力可归纳为四类：

| 参考对象 | 可借鉴能力 |
|---|---|
| SimNow / CTP 仿真体系 | 按期货交易规则练习，贴近真实交易时间、合约和撮合行为。 |
| 文华财经模拟交易 | 虚拟资金、交易所规则、手续费/保证金、条件单、止盈止损、交易分析报告。 |
| 同花顺期货通模拟交易 | 行情、资讯、下单、夜盘、T+0、移动/PC 同步体验。 |
| TradingView Paper Trading | 无真实资金、接近真实市场、支持期货生命周期、费用和账户余额变化。 |

ThisIsMyQuant 不复制实盘交易终端，而是做一个更适合个人研究和复盘的本地模拟环境：**行情与分析解释为什么交易，模拟账户记录如何交易，本地数据库沉淀交易后的复盘证据**。

参考资料：

- 上海期货交易所投资者教育平台：SimNow 模拟仿真交易平台。
- 文华财经 WH6 使用说明：模拟交易、交易分析报告、条件单/止损止盈。
- 同花顺期货通 PC 版说明：行情资讯、模拟交易、PC 与 APP 同步。
- TradingView Help Center：Paper Trading main functionality。

## 2. 产品目标

1. 用虚拟资金练习国内期货主力品种交易。
2. 用本地撮合引擎模拟市价、限价等基础订单；止损、止盈、OCO、移动止损、条件单等高级订单已实现但收敛到实验区。
3. 按合约乘数、最小变动价位、保证金率、手续费规则计算账户权益。
4. 将订单、成交、持仓、资金曲线、交易计划、复盘记录全部写入 SQLite。
5. 把交易与行情、资讯、因子、日历、LLM 报告关联起来，形成可复盘的决策证据链。
6. 回放训练作为设置页实验区能力保留，不进入主导航。
7. 保持合规边界：只做模拟，不接实盘，不做代客交易。

## 3. 范围

### In Scope

- 虚拟账户与虚拟资金。
- 国内商品期货和航运运价主力品种的模拟交易。
- 开仓、平仓、平今、撤单、成交、持仓、盯市盈亏。
- 保证金、手续费、滑点、合约乘数、最小变动价位。
- 交易日记、交易计划、复盘标签、截图/报告引用。
- 条件单、止损止盈、OCO、移动止损（已实现，收敛到设置页实验区）。
- 历史行情回放训练和盘中模拟（已实现，收敛到设置页实验区）。
- 本地 SQLite 数据库、导入导出和备份。

### Out of Scope

- 真实交易、实盘下单、账户登录、银期转账。
- 连接期货公司交易柜台。
- 向交易所或经纪商发送委托。
- 多用户 SaaS 账户或云端同步。
- 高频交易撮合精度承诺。

## 4. 核心模块

| 模块 | 说明 |
|---|---|
| 模拟账户 | 账户列表、初始资金、可用资金、保证金占用、权益、风险度。 |
| 下单面板 | 买开、卖开、买平、卖平、平今、平昨、撤单。 |
| 撮合引擎 | 基于当前行情或回放 K 线模拟成交，支持滑点和部分成交策略。 |
| 订单管理 | 委托、撤单、已成交、已拒单、条件单状态流转。 |
| 持仓管理 | 多空持仓、均价、浮盈、保证金、手续费、平仓盈亏。 |
| 风控规则 | 资金不足拒单、最大手数、单品种风险、止损线、强平模拟。 |
| 复盘中心 | 交易日记、理由、执行评分、情绪标签、关联资讯/报告。 |
| 绩效分析 | 收益曲线、回撤、胜率、盈亏比、品种贡献、时段贡献。 |
| 回放训练 | 按交易日/品种回放行情，隐藏未来数据，下单练习（实验区）。 |
| 本地数据库 | 保存行情、订单、成交、持仓、资金、复盘和配置。 |

## 5. 主用户路径（基础模拟盘）

```
市场发现
  → 标的详情
  → 基础模拟盘
  → 交易复盘（可选）
```

基础模拟盘主能力覆盖：市价/限价、买卖、开平、手数、费用估算、持仓、委托、成交、资金流水、账户重置。

完整模拟交易工作流：

```
选择虚拟账户
  → 选择品种/合约
  → 查看行情、因子、资讯、报告
  → 填写交易计划
  → 下单
  → 撮合成交/挂单
  → 更新持仓与资金
  → 盘后复盘
  → 绩效分析与 LLM 复盘总结
```

## 6. 订单与成交规则

### 6.1 订单类型

主业务订单类型：

| 类型 | 说明 |
|---|---|
| 市价单 | 按当前可用价格成交，加入可配置滑点。 |
| 限价单 | 价格达到或穿越时成交。 |

实验/后续能力（设置页实验区可用）：

| 类型 | 说明 |
|---|---|
| 止损单 / 止损限价 | 触发价达到后转市价或限价平仓。 |
| 止盈单 / 止盈限价 | 触发价达到后转市价或限价平仓。 |
| 移动止损 | 随盈利扩大动态收紧触发价，回撤指定 tick 后平仓。 |
| 条件单 | 按价格触发，支持 `>=` / `<=` 两个方向。 |
| OCO | 止盈与止损二选一，任一成交后取消另一单。 |

### 6.2 开平规则

国内期货需要显式处理方向和开平：

| 字段 | 取值 |
|---|---|
| `side` | buy / sell |
| `offset` | open / close / close_today / close_yesterday |
| `position_side` | long / short |

平仓时优先按交易所规则和用户设置处理平今/平昨。若数据源无法区分昨仓，模拟盘可按“先平今/先平昨”配置执行，并在成交记录中标记 `estimated`。

### 6.3 成交价格

| 模式 | 说明 |
|---|---|
| 盘中模拟 | 使用最新 quote 或 K 线 close 作为基础价。 |
| 历史回放 | 当前回放 bar 内按 high/low 判断限价是否触发。 |
| 保守撮合 | 买入按更不利价格，卖出按更不利价格。 |
| 自定义滑点 | 按 tick 数或百分比加入滑点。 |

## 7. 保证金与手续费

每个合约需要维护规则：

| 字段 | 说明 |
|---|---|
| `contract_multiplier` | 合约乘数。 |
| `price_tick` | 最小变动价位。 |
| `margin_rate_long` / `margin_rate_short` | 多/空保证金率。 |
| `commission_open` | 开仓手续费。 |
| `commission_close` | 平仓手续费。 |
| `commission_close_today` | 平今手续费。 |
| `commission_mode` | 按手数、按成交额比例或混合。 |

公式示例：

```text
成交额 = 成交价 × 合约乘数 × 手数
保证金 = 成交额 × 保证金率
手续费 = 按手数费用或成交额比例费用
账户权益 = 可用资金 + 保证金占用 + 浮动盈亏
风险度 = 保证金占用 / 账户权益
```

规则来源优先级：

1. 本地内置默认规则。
2. 用户自定义规则。
3. 后续公开数据源同步。

## 8. 本地数据库 Schema 草案

```sql
CREATE TABLE sim_accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'CNY',
    initial_balance REAL NOT NULL,
    cash_balance REAL NOT NULL,
    equity REAL NOT NULL,
    margin_used REAL NOT NULL DEFAULT 0,
    realized_pnl REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE sim_contract_rules (
    symbol TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    exchange TEXT NOT NULL,
    contract_multiplier REAL NOT NULL,
    price_tick REAL NOT NULL,
    margin_rate_long REAL NOT NULL,
    margin_rate_short REAL NOT NULL,
    commission_mode TEXT NOT NULL,
    commission_open REAL NOT NULL,
    commission_close REAL NOT NULL,
    commission_close_today REAL NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE sim_orders (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    side TEXT NOT NULL,
    offset TEXT NOT NULL,
    order_type TEXT NOT NULL,
    price REAL,
    trigger_price REAL,
    stop_loss_price REAL,
    take_profit_price REAL,
    oco_group_id TEXT,
    parent_order_id TEXT,
    tif TEXT,
    condition_operator TEXT,
    trailing_distance_ticks REAL,
    trailing_reference_price REAL,
    quantity INTEGER NOT NULL,
    filled_quantity INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    reason TEXT,
    source TEXT NOT NULL DEFAULT 'manual',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE sim_trades (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    offset TEXT NOT NULL,
    price REAL NOT NULL,
    quantity INTEGER NOT NULL,
    commission REAL NOT NULL,
    slippage REAL NOT NULL DEFAULT 0,
    realized_pnl REAL NOT NULL DEFAULT 0,
    traded_at TEXT NOT NULL
);

CREATE TABLE sim_positions (
    account_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    position_side TEXT NOT NULL,
    today_qty INTEGER NOT NULL DEFAULT 0,
    history_qty INTEGER NOT NULL DEFAULT 0,
    avg_price REAL NOT NULL,
    margin REAL NOT NULL,
    unrealized_pnl REAL NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (account_id, symbol, position_side)
);

CREATE TABLE sim_equity_snapshots (
    account_id TEXT NOT NULL,
    snapshot_at TEXT NOT NULL,
    equity REAL NOT NULL,
    cash_balance REAL NOT NULL,
    margin_used REAL NOT NULL,
    realized_pnl REAL NOT NULL,
    unrealized_pnl REAL NOT NULL,
    risk_ratio REAL NOT NULL,
    PRIMARY KEY (account_id, snapshot_at)
);

CREATE TABLE sim_journal_entries (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    symbol TEXT,
    trade_id TEXT,
    report_id TEXT,
    title TEXT NOT NULL,
    thesis TEXT,
    execution_review TEXT,
    emotion_tags TEXT,
    score INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## 9. UI 页面

| 页面 | 关键内容 |
|---|---|
| 模拟盘 | 账户权益、基础下单面板、持仓、委托、成交、风险度。 |
| 交易复盘 | 交易日记、收益曲线、品种/时段统计、LLM 复盘。 |
| 回放训练 | 选择日期和品种，播放历史行情，隐藏未来数据，模拟下单（设置页实验区）。 |
| 规则设置 | 合约乘数、保证金、手续费、滑点、风控阈值。 |
| 本地数据库 | 行情、交易、报告、资讯、复盘数据的导入导出和备份。 |

现有市场发现、标的详情、行情、因子、资讯、日历、异动、助手、报告页面应增加与基础模拟盘的联动入口。交易复盘可通过设置页实验区或报告页访问。

## 10. Command 设计

| Command | 说明 |
|---|---|
| `list_sim_accounts` | 查询模拟账户。 |
| `create_sim_account` | 创建虚拟账户。 |
| `reset_sim_account` | 重置指定账户，需确认。 |
| `place_sim_order` | 提交模拟委托。 |
| `cancel_sim_order` | 撤销模拟委托。 |
| `list_sim_orders` | 查询委托。 |
| `list_sim_trades` | 查询成交。 |
| `list_sim_positions` | 查询持仓。 |
| `get_sim_account_snapshot` | 查询权益、资金、风险度。 |
| `list_sim_equity_curve` | 查询资金曲线。 |
| `save_sim_journal_entry` | 保存交易复盘。 |
| `list_sim_journal_entries` | 查询交易复盘。 |
| `update_sim_contract_rules` | 更新手续费/保证金规则。 |
| `start_market_replay` | 启动历史回放（实验区）。 |

## 11. 风控与合规

- 所有下单按钮必须标明“模拟”。
- 模拟账户与实盘账户在命名、颜色、状态上明确区分。
- 不允许配置期货公司账号、资金账号、交易密码。
- 不实现真实交易网关，不发送真实委托。
- 导出的交易记录需标注为模拟交易。
- LLM 复盘只能评价执行质量和风险，不给出确定性收益承诺。

## 12. 开发阶段

| 阶段 | 内容 |
|---|---|
| S1 | SQLite schema、虚拟账户、手动下单、委托/成交/持仓/资金。 |
| S2 | 保证金/手续费规则、风控、撤单、止损止盈、条件单。 |
| S3 | 与行情图表联动、图表下单标记、交易日记。 |
| S4 | 绩效分析、资金曲线、LLM 复盘、导入导出。 |
| S5 | 历史行情回放训练、策略沙盒、模拟竞赛/评分（实验区）。 |
