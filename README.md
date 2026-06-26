# ThisIsMyQuant

国内期货分析桌面应用（Tauri + Rust + React）。

## 开发环境

**前置：** Rust（stable）、Node.js 20+、pnpm

```bash
pnpm install               # 自动安装 pre-push 钩子（push 前跑 CI）
bash scripts/sync-env.sh   # 可选：同步金十 Token 等本地凭据
pnpm tauri:dev             # 启动桌面客户端（Rust 核心 + Vite 热更新）
```

`git push` 前会自动执行 `pnpm test:ci`（与 GitHub Actions 一致）。验证 Linux Rust 构建：`pnpm test:ci:linux`（需 Docker）。详见 [CONTRIBUTING.md](CONTRIBUTING.md)。
