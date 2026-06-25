# ThisIsMyQuant

> 自托管的国内期货分析桌面应用：实时行情 + Lightweight Charts K 线 + 大模型走势分析。

---

## 项目简介

ThisIsMyQuant 面向个人期货交易者与量化爱好者，提供「看盘 → 理解 → 决策」的闭环：

- **K 线数据**：AKShare 历史/分钟 K 线，轮询驱动图表增量更新
- **资讯数据**：金十财经期货新闻，纳入 LLM 分析上下文
- **K 线图表**：Lightweight Charts，多周期、技术指标、实时增量
- **AI 分析**：定时 / 手动调用大模型生成走势分析报告与 Copilot 追问
- **自托管隐私**：SQLite 本地存储，LLM Key 自带，数据不出本机
- **仅分析不交易**：规避合规风险，专注信息呈现与辅助决策

> ⚠️ 本项目生成的分析报告仅供参考，不构成投资建议。

---

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面壳 + 业务核心 | Tauri 2 (Rust) |
| 数据源 | AKShare（K 线）· 金十（资讯） |
| 数据库 | SQLite |
| 前端 | React 18 · TypeScript · Vite · shadcn/ui · Tailwind CSS |
| 图表 | TradingView `lightweight-charts` v5 |
| LLM | Doubao / MiniMax / OpenAI / DeepSeek / Qwen（OpenAI 兼容） |

前后端通信：**Tauri IPC（invoke）+ 事件推送**，无独立 HTTP 后端。

---

## 目录结构

```
ThisIsMyQuant/
├── frontend/               # React 前端
│   ├── src/
│   │   ├── features/       # chart / market / analysis / news
│   │   ├── api/client.ts   # Tauri invoke 封装
│   │   └── ws/socket.ts    # Tauri 事件（kline-update 等）
│   └── package.json
├── src-tauri/              # Rust 核心
│   ├── src/
│   │   ├── lib.rs          # 启动、调度、Tauri 命令注册
│   │   ├── commands.rs     # IPC API
│   │   ├── adapters/       # AKShare、金十、LLM
│   │   ├── engine/         # K 线、指标、分析、维度
│   │   ├── services/       # 轮询、分析任务、资讯入库
│   │   └── db/             # SQLite
│   └── tauri.conf.json
├── docs/
├── scripts/
├── data/                   # SQLite（git 忽略）
├── .env / .env.example
└── package.json            # pnpm tauri:dev / tauri:build
```

---

## 快速开始

### 1. 环境

- Rust（stable）、Node 20+、pnpm
- macOS / Windows / Linux（Tauri 目标平台）

### 2. 配置

```bash
bash scripts/sync-env.sh   # 或 cp .env.example .env 后编辑 LLM Key
```

### 3. 开发

```bash
pnpm install
pnpm tauri:dev      # 推荐：Rust 核心 + Vite 热更新
```

或：

```bash
bash scripts/dev.sh
```

### 4. 打包

```bash
pnpm tauri:build
```

---

## 文档

| 文档 | 内容 |
|---|---|
| [架构设计](docs/ARCHITECTURE.md) | Rust 核心分层、模块职责 |
| [通信机制](docs/COMMUNICATION.md) | Tauri IPC 与事件协议 |
| [UI 设计](docs/DESIGN.md) | Cursor 极简深色主题 |
| [使用方式](docs/USAGE.md) | 配置、排障 |
| [需求文档](docs/REQUIREMENTS.md) | 功能需求与里程碑 |

---

## 许可

MIT
