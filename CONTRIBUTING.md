# 贡献规范

## 推送前必做（与 GitHub CI 一致）

`pnpm install` 会自动配置 pre-push 钩子，**每次 `git push` 前自动运行**：

```bash
pnpm test:ci   # 即 scripts/ci-local.sh
```

手动重装钩子：

```bash
bash scripts/install-githooks.sh
```

紧急跳过（不推荐）：`git push --no-verify`

### 检查项说明

| 步骤 | 命令 | 对应 CI job |
|------|------|-------------|
| Rust 单元测试 | `cargo test --manifest-path src-tauri/Cargo.toml --lib` | rust |
| 前端类型（与 build 相同） | `pnpm --dir frontend exec tsc -b` | frontend |
| ESLint | `pnpm --dir frontend run lint` | frontend |
| 生产构建 | `pnpm --dir frontend run build` | frontend / e2e |
| Mock E2E | `VITE_E2E_MOCK=true pnpm --dir frontend exec playwright test --project=ui-mock` | e2e |

首次跑 E2E 需安装浏览器：`pnpm --dir frontend exec playwright install chromium`

### 常见踩坑

- **`.gitignore` 中的 `data/`** 会忽略所有名为 `data` 的目录，包括 `frontend/src/data/`。应使用 `/data/` 仅忽略仓库根目录运行时数据。
- 修改 `frontend/package.json` 后须提交 `frontend/pnpm-lock.yaml`。
- Rust CI 若因 crates.io 网络抖动失败，本地 `cargo test --lib` 通过即可重跑 CI；workflow 已含 `cargo fetch` 重试。

## 远程分支保护

仓库管理员在 GitHub **Settings → Branches → main** 启用 **Require status checks**，勾选 `rust`、`frontend`、`e2e`。
