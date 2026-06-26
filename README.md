# ThisIsMyQuant

国内期货分析桌面应用（Tauri + Rust + React）。

## 开发环境

**前置：** Rust（stable）、Node.js 20+、pnpm

```bash
pnpm install
bash scripts/sync-env.sh   # 可选：同步金十 Token 等本地凭据
pnpm tauri:dev             # 启动桌面客户端（Rust 核心 + Vite 热更新）
bash scripts/install-githooks.sh   # 可选：推送前自动跑 CI 检查
```

首次启动在 Landing 页填写大模型 API Key；也可在设置页修改。Debug 构建会自动从 `.env` 或 `~/global_env/.env` 导入 LLM Key（若已配置）。

推送前运行 `pnpm test:ci` 确保与 GitHub Actions 一致；详见 [CONTRIBUTING.md](CONTRIBUTING.md)。
