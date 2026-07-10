# 代码规范（CODE STANDARDS）

> 版本：v2.3 · 2026-07-09 · Tauri 单体 Rust 核心

---

## 1. 通用原则

1. **可读性优先**：命名清晰、结构直白。
2. **单一职责**：函数/模块只做一件事。
3. **显式优于隐式**：避免魔法数字与隐藏副作用。
4. **类型安全**：Rust 与 TypeScript 均开启严格检查。
5. **注释解释「为什么」**，不重复代码本身。

---

## 2. Rust 核心规范（src-tauri）

### 2.1 工具链

| 工具 | 用途 |
|---|---|
| `cargo fmt` | 格式化 |
| `cargo clippy` | lint |
| `cargo test` | 单元 / 集成测试 |

### 2.2 分层

| 层 | 目录 | 职责 |
|---|---|---|
| IPC | `commands/` | Tauri 命令分组，参数校验与 `ApiResponse` 封装 |
| 编排 | `services/` | 轮询、分析任务、入库流程 |
| 领域 | `engine/` | K 线、指标、Prompt、维度、板块、异动、解析 |
| 模拟 | `engine/sim_*`、`services/sim_*` | 本地撮合、虚拟账户、保证金、手续费、绩效 |
| A 股 | `adapters/stock_*`、`engine/stock_*`、`services/stock_*` | 股票目录、行情、财务、行业概念、筛选器、因子 |
| 适配 | `adapters/` | AKShare/Sina、金十、日历、Yahoo、LLM HTTP |
| 持久化 | `db/` | SQLite schema 与查询 |

依赖方向：`commands → services → engine/adapters/db`，禁止反向依赖。

新增面向页面的接口时，优先在 Rust 层返回结构化视图，例如 `ProfessionalDashboardView`，避免前端跨多个底层接口拼装业务事实。

### 2.3 命名

| 对象 | 规则 | 示例 |
|---|---|---|
| 模块/文件 | snake_case | `analysis_runner.rs` |
| 类型 | PascalCase | `AnalysisReport` |
| 函数/变量 | snake_case | `run_analysis` |
| 常量 | UPPER_SNAKE | `DEFAULT_LIMIT` |

### 2.4 错误与日志

- 领域错误用 `thiserror` 定义，对外 IPC 返回 `ApiResponse::err(message)`
- 使用 `log` crate，关键路径：`bootstrap`、轮询、分析起止、外部 HTTP 失败
- 禁止在日志中输出 API Key 或完整 prompt 中的敏感信息

### 2.5 配置

- 所有运行时配置经 `config.rs` 从 `.env` 加载
- 业务代码不直接 `std::env::var`，统一走 `AppState.config`

### 2.6 测试

- 集成测试：`src-tauri/tests/integration_test.rs`
- 外部网络依赖的 live 测试需显式 feature 或环境变量，CI 默认 mock / 跳过

---

## 3. 前端规范（React + TS）

### 3.1 工具链

| 工具 | 用途 |
|---|---|
| TypeScript | `strict: true` |
| Tailwind CSS | 样式（设计 token 映射） |

### 3.2 命名

| 对象 | 规则 | 示例 |
|---|---|---|
| 组件 | PascalCase | `ChartPanel.tsx` |
| hook | `use` 前缀 | `useFuturesCatalog` |
| 变量/函数 | camelCase | `loadKlines` |
| 类型/接口 | PascalCase | `KLine` |

### 3.3 数据流

- **唯一数据入口**：`api/client.ts` → Tauri `invoke`
- **实时推送**：`ws/socket.ts` → Tauri `listen`（`kline-update` 等）
- 服务端状态用 `react-query`，全局 UI 用 `zustand`
- 禁止在前端直接请求 AKShare / 金十 / LLM

### 3.4 样式

- 颜色、间距走 `src/design/tokens.css` 与 `docs/DESIGN.md`
- UI 方向是专业期货分析工作台：低噪声、可扫描、数据质量明确
- 模拟盘入口必须清晰标注“模拟”，不得让用户误以为是真实下单
- 页面必须围绕五大商品板块组织；v1 不新增金融期货默认入口
- A 股页面必须显示股票代码、交易所、数据源、报告期和复权口径
- 所有行情、因子、资讯和报告组件都应显示来源、更新时间或数据质量状态

### 3.5 测试

- E2E：`frontend/e2e/`，`VITE_E2E_MOCK=true` 内存 Mock
- 指标计算等纯函数可单测

---

## 4. Git 规范

### 4.1 Commit Message（Conventional Commits）

```
<type>(<scope>): <subject>
```

- type：`feat / fix / docs / refactor / test / chore / perf`
- scope：`core / frontend / chart / tauri / llm / docs`

### 4.2 PR

- 必须通过：`pnpm --dir frontend exec tsc --noEmit`、`cargo test`（适用时）
- 描述含动机、改动点、测试方式

---

## 5. 领域命名约定

- **symbol**：主力连续大写，如 `RB0`、`AU0`
- **name**：中文品种名，如“螺纹钢”“黄金”，UI 默认展示中文名
- **sector**：五大商品板块之一：黑色建材、有色贵金属、农产品软商品、能源化工、航运运价
- **interval**：`1m/5m/15m/30m/1h/1d`
- **timestamp**：K 线传输用 ISO 8601 字符串或 Unix 秒（与 `models.rs` 一致）
- **data_quality**：`live/history/reference/estimated/pending/stale/error`
- **stock_code**：A 股标准代码，如 `600000.SH`、`000001.SZ`
- **report_period**：财务报告期，不能用行情日期替代
- **adjustment**：复权口径，至少区分 none / qfq / hfq
- **sim_account**：虚拟账户，不对应真实资金账户
- **sim_order**：模拟委托，状态包括 pending / filled / partial / canceled / rejected
- **sim_trade**：模拟成交，只由本地撮合生成
- **offset**：国内期货开平字段：open / close / close_today / close_yesterday

---

## 6. 禁止清单

- 禁止 `println!` / `console.log` 进生产路径（调试用完即删）
- 禁止硬编码密码/Key/URL
- 禁止前端绕过 Tauri 直接调外部 API
- 禁止把用户凭据写入 LLM prompt
- 禁止把估算、参考或陈旧数据展示成实时数据
- 禁止在 v1 默认导航、关注列表或批量分析中补充金融期货
- 禁止接入真实交易、保存交易密码、发送真实委托或把模拟成交标记为真实成交
- 禁止接入券商实盘交易、保存证券账户密码或把模拟组合标记为真实持仓
- 禁止在 A 股筛选器中隐藏报告期、复权口径或数据质量
- 禁止 `git push --force` 到 `main`

---

## 7. CI 检查项

1. Rust：`cargo fmt --check && cargo clippy && cargo test`
2. 前端：`pnpm --dir frontend exec tsc --noEmit && pnpm --dir frontend build`
3. E2E（可选）：`pnpm test:e2e`
