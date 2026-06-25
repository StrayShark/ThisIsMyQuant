# 架构设计（ARCHITECTURE）

> 版本：v2.0 · Tauri 单体 Rust 核心

---

## 1. 总体架构

ThisIsMyQuant 为 **Tauri 桌面应用**：Rust 承担全部业务逻辑（行情、K 线、资讯、分析、持久化）；React 前端负责 UI 与图表；二者通过 **Tauri IPC + 事件** 通信。

```
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri 应用 (src-tauri, Rust)                    │
│  bootstrap · 轮询任务 · 分析调度 · SQLite · Tauri Commands        │
│  ┌──────────── adapters ────────────┐  ┌──── engine ──────────┐ │
│  │ AkshareClient  JinshiClient      │  │ kline / indicator    │ │
│  │ LlmRouter                        │  │ analysis / dimensions│ │
│  └──────────────────────────────────┘  └──────────────────────┘ │
│  ┌──────────── services ────────────────────────────────────────┐ │
│  │ market_poll · news_poll · analysis_runner · liquidity_job   │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────┬────────────────────────────────────┘
                             │ invoke + emit (Tauri IPC / Events)
┌────────────────────────────▼────────────────────────────────────┐
│              前端 (Vite + React + lightweight-charts)              │
│  ChartPanel · AiPanel · NewsPanel · Reports · Settings            │
└─────────────────────────────────────────────────────────────────┘
                             │
                             ▼
                    本地 SQLite (data/quant.db)
```

---

## 2. Rust 核心分层

```
src-tauri/src/
├── lib.rs              # 启动 bootstrap、注册 commands、后台任务
├── commands.rs         # Tauri IPC 命令（对外 API）
├── config.rs           # 读取 .env
├── state.rs            # AppState（共享依赖）
├── adapters/
│   ├── akshare.rs      # AKShare HTTP 客户端（K 线、合约）
│   ├── jinshi.rs       # 金十资讯
│   └── llm.rs          # 多 Provider LLM 路由与流式
├── engine/
│   ├── kline.rs        # K 线工具
│   ├── kline_agg.rs    # 周期聚合
│   ├── indicator.rs    # MA / MACD 等（分析用）
│   ├── analysis.rs     # Prompt 构建、上下文渲染
│   ├── report_parse.rs # LLM 报告 JSON 解析
│   ├── dimensions.rs   # 分析维度定义
│   ├── sectors.rs      # 品种板块目录
│   └── liquidity.rs    # 流动性分层
├── services/
│   ├── market_poll.rs  # 实时轮询 → emit kline-update
│   ├── news_poll.rs    # 资讯轮询入库
│   ├── news_ingest.rs  # 资讯分类落库
│   ├── analysis_runner.rs
│   ├── analysis_scheduler.rs
│   ├── analysis_followup.rs
│   └── liquidity_job.rs
└── db/
    └── sqlite.rs       # Schema、K 线缓存、报告、facts
```

---

## 3. 前端结构

```
frontend/src/
├── api/client.ts       # invoke 封装（唯一数据入口）
├── ws/socket.ts        # listen kline-update / notification
├── lib/tauri-bridge.ts # waitForAppReady
├── features/
│   ├── chart/          # K 线 + 指标
│   ├── analysis/       # Copilot 流式分析
│   ├── market/         # 品种列表
│   └── news/           # 资讯面板
└── pages/              # Dashboard / Reports / Settings
```

---

## 4. 启动流程

1. Tauri 加载 WebView，前端显示 `BootstrapLoader`
2. Rust `bootstrap()`：打开 SQLite、`init_schema`、连接 AKShare/金十/LLM
3. 启动 `market_poll`、`news_poll`、`analysis_scheduler`、`liquidity_job`
4. `emit("app-ready")` → 前端渲染主界面
5. 前端 `invoke("get_health")` 等拉取初始状态

---

## 5. 技术选型

| 领域 | 选型 | 说明 |
|---|---|---|
| 桌面壳 | Tauri 2 | 原生窗口、Rust 业务核心 |
| 持久化 | SQLite + rusqlite | 单文件、易备份 |
| HTTP | reqwest | AKShare / 金十 / LLM |
| 前端 | React + Vite + shadcn/ui | Cursor 极简深色 UI |
| 图表 | lightweight-charts v5 | 双/多 pane、指标叠加 |
| LLM | OpenAI 兼容 API | 多 Provider 可切换 |

---

## 6. 配置

所有运行时配置来自项目根目录 `.env`，由 `config.rs` 加载。修改后需重启应用。

关键变量：`DATABASE_URL`、`WATCHLIST`、`DEFAULT_LLM_PROVIDER`、各 `*_API_KEY`。

详见 [USAGE.md](./USAGE.md)。
