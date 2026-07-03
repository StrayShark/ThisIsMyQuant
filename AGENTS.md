# ThisIsMyQuant — Agent 项目指南

本文件面向需要在本仓库中编写、调试或审查代码的 AI 编码助手。阅读者被假设对项目一无所知，因此下文尽可能从目录结构、技术栈、构建流程、代码约定到测试与发布做完整说明。所有内容均基于仓库中的实际文件，不做假设性推断。

## 1. 项目概述

**ThisIsMyQuant** 是一款面向中国国内期货市场的分析型桌面应用，**不提供交易功能**，只做行情、资讯、技术分析与 LLM 研报生成。

- 产品形态：Tauri v2 桌面客户端（Rust 核心 + React 前端）。
- 目标平台：macOS、Windows、Linux（分别打包为 dmg、msi、appimage）。
- 数据端：K 线与行情主要来自 AKShare；资讯、财经日历来自金十数据（Jin10）。
- 分析端：基于规则 + 多维度 LLM Prompt pipeline 生成日报、异动提醒、深度品种研报等。
- 存储：本地 SQLite（`data/quant.db`），Rust 启动时自动建表，无独立迁移框架。

项目 README 见 [`README.md`](README.md)；更详细的设计与需求文档见 [`docs/`](docs/)。

## 2. 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面壳 | Tauri v2 | Rust 提供 command + event，前端为 WebView。 |
| 后端/核心 | Rust (edition 2021, MSRV 1.77.2) | 全部业务逻辑位于 `src-tauri/src/`。 |
| 前端框架 | React 18 + TypeScript 5 | 严格模式，Vite 构建。 |
| 构建工具 | Vite 6 | 开发端口 `5173`，`@/` 指向 `frontend/src/`。 |
| UI 组件 | shadcn/ui (New York) + Radix UI | 原子组件位于 `frontend/src/components/ui/`。 |
| 样式 | Tailwind CSS 3 + `tailwindcss-animate` | 暗色主题，token 见 `frontend/src/design/tokens.css`。 |
| 路由 | React Router 6 | 使用 `HashRouter`，路由定义在 `frontend/src/App.tsx`。 |
| 状态管理 | Zustand + TanStack Query | Zustand 管纯 UI 状态，React Query 管服务端/异步状态。 |
| 图表 | lightweight-charts 5 | K 线、技术指标等。 |
| HTTP (Rust) | reqwest | 调用 AKShare、Jin10、LLM 接口。 |
| 异步运行时 | tokio | 多线程 runtime + 定时任务。 |
| 数据库 | SQLite via `rusqlite` (bundled) | 文件 `data/quant.db`，WAL 模式。 |
| LLM 接入 | OpenAI 兼容协议 | 支持 Doubao、MiniMax、OpenAI、DeepSeek、Qwen，可配置 fallback。 |
| 测试 | Vitest + Playwright + cargo test | 前端单元、Playwright mock/live E2E、Rust 单元/集成测试。 |
| 包管理 | pnpm 9 | 根 workspace + `frontend/` 子 workspace。 |
| CI/CD | GitHub Actions + 本地 Docker | CI、夜间 live E2E、tag 发布。 |

## 3. 目录结构

```
.
├── .cursor/                 # Cursor IDE 规则
├── .githooks/               # 自定义 Git hooks（pre-push）
├── .github/workflows/       # CI/CD：ci.yml、e2e-nightly.yml、release.yml
├── .vscode/                 # VS Code 工作区设置
├── data/                    # SQLite 数据文件、WAL/SHM、e2e 就绪标记
├── docs/                    # 项目文档（架构、设计、需求、数据源等）
├── frontend/                # React + Vite + TypeScript 前端
│   ├── e2e/                 # Playwright E2E 测试
│   ├── public/              # 静态资源
│   └── src/
│       ├── api/             # Tauri invoke 统一客户端 / E2E mock
│       ├── app/             # Zustand 全局状态
│       ├── components/      # 通用与布局组件（含 ui/ 原子组件）
│       ├── data/            # 静态期货、日历、维度数据
│       ├── design/          # 主题 token / CSS 变量
│       ├── features/        # 按业务域组织的模块
│       ├── hooks/           # 自定义 React Hooks
│       ├── lib/             # 工具函数与平台桥接
│       ├── pages/           # 页面级路由组件
│       ├── ws/              # WebSocket 相关
│       ├── App.tsx          # 根组件、路由、Provider
│       ├── main.tsx         # React 入口（先 BootstrapLoader）
│       └── types.ts         # 共享 TS 类型
├── scripts/                 # 开发、CI、构建、图标生成等脚本
│   └── docker/              # Linux CI 镜像 Dockerfile
├── src-tauri/               # Tauri + Rust 核心
│   ├── capabilities/        # Tauri 权限配置
│   ├── icons/               # 应用图标
│   ├── src/
│   │   ├── adapters/        # 外部数据源客户端（AKShare、Jin10、LLM 等）
│   │   ├── commands.rs      # Tauri command 处理器
│   │   ├── config.rs        # 主配置 + .env 加载
│   │   ├── config/          # 偏好设置、LLM 目录辅助
│   │   ├── crypto/          # AES-GCM 凭据加密
│   │   ├── db/              # SQLite / QuestDB 适配器
│   │   ├── engine/          # 分析、指标、分类、板块等核心逻辑
│   │   ├── error.rs         # 统一错误类型
│   │   ├── lib.rs           # 库入口、bootstrap、run()
│   │   ├── logging.rs       # trace 感知日志辅助
│   │   ├── main.rs          # 二进制入口
│   │   ├── models.rs        # 共享 DTO
│   │   ├── services/        # 后台轮询、任务调度服务
│   │   ├── state.rs         # AppState 定义
│   │   └── testing/         # 调试/测试辅助（含 E2E HTTP 探针）
│   ├── tests/               # Rust 集成测试
│   ├── Cargo.toml           # Rust manifest
│   ├── tauri.conf.json      # Tauri 应用配置
│   └── build.rs             # Tauri build hook
├── .env / .env.example      # 运行时配置与模板
├── package.json             # 根 workspace 脚本
└── pnpm-lock.yaml           # pnpm 锁文件
```

## 4. 构建与开发命令

前置要求：Rust stable、Node.js 20+、pnpm 9+；macOS 上 `pnpm install` 会自动配置 pre-push 钩子。

### 首次设置

```bash
pnpm install               # 安装依赖并启用 .githooks/pre-push
bash scripts/sync-env.sh   # 可选：从 ~/global_env/.env 同步 LLM/数据源凭据到 .env
```

### 常用开发命令

```bash
pnpm dev                   # 等价于 pnpm tauri:dev，启动桌面客户端（Rust + Vite 热更新）
pnpm tauri:dev             # Tauri dev 模式
pnpm tauri:build           # Tauri 生产构建
```

### 前端独立开发

```bash
pnpm --dir frontend dev    # 仅启动 Vite（端口 5173）
pnpm --dir frontend build  # tsc + vite build
pnpm --dir frontend lint   # ESLint
pnpm --dir frontend tsc    # 类型检查
pnpm --dir frontend test   # Vitest 单元测试
```

### Rust 相关

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml --test integration_test -- --nocapture
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
```

## 5. 测试说明

项目测试分为三层：前端单元测试、Playwright E2E、Rust 单元/集成测试。

### 前端单元测试

- 框架：Vitest 2
- 配置：`frontend/vitest.config.ts`
- 匹配：`frontend/src/**/*.test.{ts,tsx}`
- 示例：`frontend/src/features/overview/treemap-layout.test.ts`
- 运行：`pnpm --dir frontend test`

### Playwright E2E

- 主配置：`frontend/playwright.config.ts`
- 客户端真实后端配置：`frontend/playwright.client.config.ts`
- 截图配置：`frontend/playwright.screenshot.config.ts`

| 项目 | 命令 | 说明 |
|------|------|------|
| ui-mock | `pnpm test:e2e` | mock API 下的无头 E2E，CI 默认运行。 |
| client-live | `pnpm test:e2e:client` | 连接真实 Tauri 后端，需要 LLM/数据源凭据。 |
| UI 模式 | `pnpm --dir frontend test:e2e:ui` | Playwright UI 调试模式。 |

Mock 机制：设置 `VITE_E2E_MOCK=true` 后，前端 `api/client.ts` 会替换为 `api/e2e-mock.ts`，不调用 Rust command。

### Rust 测试

- 库测试：`cargo test --lib`
- 集成测试：`src-tauri/tests/integration_test.rs`，覆盖 SQLite roundtrip、AKShare/Jin10 实时接口、LLM health 等。
- 客户端 E2E：`src-tauri/tests/client_e2e.rs`，启动完整 `AppState` 并跑 client E2E suite。

### CI / 推送前检查

```bash
pnpm test:ci               # Mac 本地：gitignore 自检、cargo test、tsc、lint、build、mock E2E
pnpm test:ci:linux         # Docker Ubuntu：Tauri 系统依赖 + cargo test --lib
pnpm test:ci:all           # 依次跑 ci + ci:linux（即 pre-push 钩子内容）
pnpm test:ci:linux:rebuild # 重建 CI Docker 镜像后运行 ci:linux
```

紧急跳过 pre-push：`git push --no-verify`（不推荐）。

## 6. 代码风格与约定

### 通用原则

- 可读性优先、单一职责、显式代码、严格类型。
- 禁止在正式路径使用 `println!` / `console.log`；使用 `log` crate 或前端日志封装。
- 禁止硬编码密钥；凭据走 `.env` 或 SQLite 加密凭据表。
- 前端禁止直接调用外部 API，所有后端请求统一通过 `src/api/client.ts` 走 Tauri invoke。
- 提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/)。
- `main` 分支禁止 force push。

### 前端约定

**文件命名**

| 类型 | 示例 |
|------|------|
| 页面/业务组件 | `OverviewPage.tsx`、`AppShell.tsx`（PascalCase） |
| 原子 UI 组件 | `filter-pill.tsx`、`panel-skeleton.tsx`（kebab-case） |
| Hooks | `useAppearance.ts` |
| 工具/纯函数 | `utils.ts`、`tauri-bridge.ts`（camelCase） |
| 测试 | `*.test.ts(x)` / `*.spec.ts(x)` |

**组件与样式**

- 原子组件放在 `components/ui/`，使用 `class-variance-authority` 定义变体，使用 `cn()` 合并 Tailwind 类名。
- 颜色主题通过 CSS 变量（`design/tokens.css`、`themes.css`）驱动，Tailwind 配置映射到 `hsl(var(--...))`。
- 默认暗色主题：`index.html` 中 `<html class="dark">`。

**API 调用**

- 统一通过 `src/api/client.ts` 的 `api` 对象调用。
- 运行时检测 `__TAURI_INTERNALS__`，动态 import `@tauri-apps/api/core` 调用 Rust command。
- 接口统一返回 `ApiResponse<T>`，通过 `unwrap()` 校验 `code === 0`。

**状态管理**

- 服务端/异步数据优先使用 TanStack Query。
- 纯客户端 UI 状态使用 Zustand（`src/app/store.ts`）。

### Rust 约定

**命名与格式**

- 文件/模块：snake_case；类型：PascalCase。
- 必须能通过 `cargo fmt` 与 `cargo clippy`。

**Command 层**

- `#[tauri::command]` 函数通常为 `pub async fn`。
- 共享状态通过 `State<'_, Arc<AppState>>` 注入，需要事件发射时接收 `AppHandle`。
- 返回类型几乎总是 `Result<ApiResponse<T>, String>`；错误通常包装为 `ApiResponse::err(...)`。

**错误处理**

- 核心错误类型：`error::AppError`（`Msg`、`Http`、`Db`、`Json`），基于 `thiserror`。
- 别名：`AppResult<T> = Result<T, AppError>`。
- Command 层把错误转换为 `String` 或 `ApiResponse::err(e.to_string())`。

**状态管理**

- 单个 `Arc<AppState>` 由 Tauri 管理：`app.manage(state.clone())`。
- 可变共享状态用 `std::sync::RwLock`（配置、LLM router、用户偏好、行情缓存）或 `tokio::sync::Mutex`（异步客户端/轮询句柄）。
- 锁中毒时通过 `.unwrap_or_else(|e| e.into_inner())` 恢复。

**日志**

- 使用 `log` crate + `tauri-plugin-log`。
- 结构化 trace 辅助：`logging::log_trace(trace_id, level, msg)`。

## 7. 运行时架构

1. **启动流程**（`src-tauri/src/lib.rs` 的 `bootstrap`）：
   - 加载 `.env` 与用户偏好；
   - 打开 SQLite 并执行 `init_schema()`（`CREATE TABLE IF NOT EXISTS` + 必要的 `ALTER TABLE`）；
   - 初始化 AKShare/Jin10/LLM router 适配器；
   - 启动后台服务（行情轮询、资讯轮询、历史回填、定时综合分析、日历提醒等）；
   - 构建 `AppState` 并注册 Tauri commands；
   - 向前端发送 `app-ready` 事件。

2. **前后端通信**：
   - 请求/响应：前端 `invoke`，Rust `commands.rs` 处理。
   - 服务端推送：Rust 通过 `AppHandle` 发送事件，如 `kline-update`、`analysis-delta` / `analysis-done` / `analysis-error`、`notification`。

3. **后台服务**（`src-tauri/src/services/`）：
   - `market_poll`：行情轮询；
   - `news_poll` / `news_ingest`：资讯抓取与分类；
   - `history_backfill`：历史 K 线回补；
   - `schedule_runner`：定时综合分析；
   - `daily_briefing`：日报生成；
   - `anomaly_watcher`：异动监测；
   - `liquidity_job`：流动性快照；
   - `data_maintenance`：数据清理维护。

4. **分析流水线**：
   - 资讯分类（规则 + 可选 LLM）→ 去重（SHA256）→ 维度摘要 → 综合研报。
   - 维度定义见 `docs/FUTURES_ANALYSIS_DESIGN.md` 与 `src-tauri/src/engine/dimensions.rs`。

## 8. 配置与凭据

- 模板文件：`.env.example`。
- 本地凭据：`.env`（不提交）。
- 同步脚本：`scripts/sync-env.sh` 会从 `~/global_env/.env` 抽取需要的键并写入项目 `.env`，同时为空 `ENCRYPTION_KEY` 生成随机值。
- Rust 启动时会将 LLM key 等敏感信息加密后存入 SQLite `llm_credentials` 表；运行时通过 `crypto/credentials.rs` 的 AES-GCM 加密/解密读取。
- 用户偏好在 SQLite `user_preferences` 中维护，可在设置页修改。

## 9. 数据库

- 主数据库：`data/quant.db`（路径由 `DATABASE_URL` 决定）。
- 模式初始化：`src-tauri/src/db/sqlite.rs` 在应用启动时创建所有表，使用 `CREATE TABLE IF NOT EXISTS` 与 additive `ALTER TABLE`，**没有独立迁移框架**。
- WAL 模式已启用：`PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;`。
- 可选 QuestDB 适配器：`src-tauri/src/db/questdb.rs`（当前主要为 stub/可选集成）。

## 10. 安全与合规

- **没有交易功能**：应用仅做行情展示与分析，不连接任何交易柜台。
- **凭据加密**：LLM API key 等敏感配置使用 AES-GCM 加密后落盘，不在日志中明文打印。
- **外部网络**：前端 CSP 仅允许 `localhost` / `127.0.0.1` 与若干 LLM 服务商域名（Volces、MiniMax、OpenAI、DeepSeek、DashScope）。
- **Tauri 权限**：权限集中在 `src-tauri/capabilities/default.json`，包括 `core:default`、窗口拖拽/最大化、`shell:allow-open`、`process:allow-exit`/`allow-restart`。
- **禁止泄露 secrets**：`.env`、加密 key、数据库文件均已在 `.gitignore` 中；新增含 secrets 的文件时必须同步更新 gitignore 与凭据加密逻辑。
- **合规提示**：数据来自 AKShare、金十等公开或授权接口；使用 LLM 分析时注意不要上传非公开敏感信息，遵守各 LLM 服务商与数据源的条款。

## 11. 发布与部署

- 本地发布构建：`pnpm tauri:build` 或 `pnpm release:smoke`（仅构建、无网络/E2E）。
- GitHub Actions 发布：`.github/workflows/release.yml`，在 `v*` tag 推送时触发，构建 macOS (aarch64) 与 Linux 安装包。
- 产物：macOS `.dmg`、Windows `.msi`、Linux `.appimage`。
- 没有 Kubernetes、docker-compose 或服务器部署流程；这是一款纯本地桌面应用。

## 12. 快速参考：常用命令速查

```bash
# 开发
pnpm install
pnpm tauri:dev

# 前端
pnpm --dir frontend dev
pnpm --dir frontend build
pnpm --dir frontend lint
pnpm --dir frontend tsc
pnpm --dir frontend test

# 测试
pnpm test:rust
pnpm test:e2e
pnpm test:e2e:client
pnpm test:ci
pnpm test:ci:linux
pnpm test:ci:all

# Rust 格式/检查
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml

# 图标生成
pnpm icon:gen
```

## 13. 参考文档

- [`README.md`](README.md) — 快速开始
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — 贡献与 pre-push 规范
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — 架构设计
- [`docs/DESIGN.md`](docs/DESIGN.md) — UI/UX 设计规范
- [`docs/CODE_STANDARDS.md`](docs/CODE_STANDARDS.md) — 代码规范
- [`docs/FUTURES_ANALYSIS_DESIGN.md`](docs/FUTURES_ANALYSIS_DESIGN.md) — 期货分析维度设计
- [`docs/DATA_SOURCES.md`](docs/DATA_SOURCES.md) — 数据源说明
- [`docs/USAGE.md`](docs/USAGE.md) — 使用与调测说明
- [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md) — 需求与里程碑
