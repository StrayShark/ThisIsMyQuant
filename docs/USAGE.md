# 使用方式（USAGE）

> 版本：v2.0 · Tauri 桌面应用

---

## 1. 环境准备

| 依赖 | 版本 | 说明 |
|---|---|---|
| Rust | stable | [rustup.rs](https://rustup.rs/) |
| Node.js | 20+ | |
| pnpm | 9+ | |

### 配置 .env

```bash
bash scripts/sync-env.sh
# 或 cp .env.example .env
```

| 变量 | 必填 | 说明 |
|---|---|---|
| `DEFAULT_LLM_PROVIDER` | 是 | `doubao` / `minimax` / `openai` 等 |
| `DOUBAO_API_KEY` / `MINIMAX_API_KEY` | 至少一个 | LLM 分析 |
| `DATABASE_URL` | 否 | 默认 `sqlite:///data/quant.db` |
| `WATCHLIST` | 否 | 轮询品种，逗号分隔 |
| `AKSHARE_ENABLED` | 否 | 默认 true |
| `JINSHI_ENABLED` | 否 | 金十资讯 |
| `DAILY_ANALYSIS_CRON` | 否 | 如 `"0 17"` 每日 17:00 |
| `REALTIME_ANALYSIS_INTERVAL` | 否 | 秒，0 关闭盘中分析 |

修改 `.env` 后需**重启应用**。

---

## 2. 开发与打包

```bash
pnpm install
pnpm tauri:dev      # 开发
pnpm tauri:build    # 打包 .app / .exe / .AppImage
```

等价于 `bash scripts/dev.sh`。

首次启动时 Rust 核心会自动创建 SQLite 表结构；也可预先执行：

```bash
bash scripts/init-db.sh   # 仅创建 data/ 目录
```

---

## 3. 功能使用

### K 线与指标

- 侧栏「行情」进入工作台，切换周期 Tab（默认日 K）
- 图表下方工具栏开关 MA / BOLL / MACD / RSI

### AI 分析

- Copilot 面板输入问题或点击发送，触发流式分析
- 「报告」页查看历史报告；报告详情可追问（历史持久化）

### 资讯

- 工作台右侧 NewsPanel 展示与当前品种相关的金十资讯

---

## 4. 手动触发分析

界面：Copilot 发送即触发 `stream_analysis`。

无 HTTP API；调试可用 Rust 集成测试或 UI。

---

## 5. 脚本

| 脚本 | 作用 |
|---|---|
| `scripts/sync-env.sh` | 从 `~/global_env/.env` 同步 |
| `scripts/dev.sh` | 启动 `pnpm tauri:dev` |
| `scripts/init-db.sh` | 创建 `data/` 目录 |

---

## 6. 排障

| 现象 | 排查 |
|---|---|
| 启动白屏过久 | 查看终端 Rust 日志；8s 后前端仍会渲染 |
| K 线空白 | AKShare 网络；设置页查看数据源状态 |
| 分析失败 | 确认 LLM Key；终端 `LLM` 相关错误 |
| 实时 K 线不更新 | `WATCHLIST` 是否包含当前品种；`market_subscribe` |

健康状态：设置页只读展示，或侧栏底部数据源指示点。

---

## 7. 常见问题

**Q：还需要 Python 吗？**  
A：不需要。业务逻辑已全部在 Rust 核心中。

**Q：能接 Ollama 吗？**  
A：配置 `OPENAI_BASE_URL=http://localhost:11434/v1`，`OPENAI_API_KEY=ollama`，`DEFAULT_LLM_PROVIDER=openai`。

**Q：数据存在哪？**  
A：默认 `data/quant.db`（SQLite），报告与 K 线缓存均在本地。
