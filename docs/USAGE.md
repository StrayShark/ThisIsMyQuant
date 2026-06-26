# 使用方式（USAGE）

> 版本：v2.1 · Tauri 桌面应用

---

## 1. 环境准备

| 依赖 | 版本 | 说明 |
|---|---|---|
| Rust | stable | [rustup.rs](https://rustup.rs/) |
| Node.js | 20+ | |
| pnpm | 9+ | |

### 配置

```bash
bash scripts/sync-env.sh   # 从 ~/global_env 同步金十 Token、LLM Key 等到 .env
```

| 变量 | 必填 | 说明 |
|---|---|---|
| `JIN10_MCP_TOKEN` | 推荐 | 金十资讯 / 财经日历 |
| `DOUBAO_API_KEY` 等 | 推荐 | 本地 debug 可写入 `.env` 并自动导入 SQLite |
| `DATABASE_URL` | 否 | 默认 `sqlite:///data/quant.db` |

### LLM API Key

| 场景 | 配置方式 |
|---|---|
| **Release 安装包** | 首次启动 Landing 页或 **设置 → 配置 LLM API Key** |
| **本地 debug** | `sync-env.sh` 后 debug 构建自动导入 SQLite（DB 无凭据时） |

运营项在 **设置页** 编辑并持久化，无需改 `.env`。

---

## 2. 开发与打包

```bash
pnpm install
pnpm tauri:dev
pnpm tauri:build
bash scripts/release-smoke.sh
```

---

## 3. 测试

| 命令 | 说明 |
|---|---|
| `pnpm test:e2e` | UI Mock 冒烟 |
| `pnpm test:e2e:client` | 启动 Tauri + LLM 明日/短期 Live E2E |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib` | Rust 单元测试 |

---

## 4. AI 分析

- Copilot：**手动** / **明日展望** / **短期研判**
- 定时任务：设置页配置周期与分析类型
- 每日简报：默认 17:00 对 watchlist 跑「明日展望」
- 异动：价格异动触发分析

报告页可按触发类型筛选。

---

## 5. 脚本

| 脚本 | 作用 |
|---|---|
| `scripts/sync-env.sh` | 同步 global_env |
| `scripts/e2e-client.sh` | 客户端 Live E2E |
| `scripts/release-smoke.sh` | Release 编译冒烟 |

---

## 6. 排障

| 现象 | 排查 |
|---|---|
| 进 Landing | Release 需配置 LLM Key |
| 分析失败 | Key、设置页大模型状态 |
| Live E2E 超时 | Key 与端口 5173/17845 |
