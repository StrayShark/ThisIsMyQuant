# 贡献规范

## 推送前必做

`pnpm install` 会自动配置 pre-push 钩子。**每次 `git push` 前自动运行以下两项**（约 5–20 分钟）：

```bash
pnpm test:ci          # [1] Mac：frontend + e2e + Rust
pnpm test:ci:linux    # [2] Docker：Ubuntu Tauri 系统库 + cargo test --lib
```

手动一次性跑完：

```bash
pnpm test:ci:all
```

**前置：** Docker Desktop 已启动；E2E 首次需 `pnpm --dir frontend exec playwright install chromium`

重装钩子：`bash scripts/install-githooks.sh`

紧急跳过（不推荐）：`git push --no-verify`

### 检查项

| 命令 | 内容 | 对应 CI |
|------|------|---------|
| `test:ci` | gitignore 自检、cargo test、tsc、lint、build、e2e mock | frontend + e2e + rust（macOS） |
| `test:ci:linux` | Ubuntu 容器、Tauri apt 依赖、cargo test --lib | rust（Linux） |

### 常见踩坑

- **`.gitignore` 的 `data/`** 会误伤 `frontend/src/data/`，应写 `/data/`
- 修改 `frontend/package.json` 后须提交 `frontend/pnpm-lock.yaml`
- Mac 上 `test:ci` 不能替代 `test:ci:linux`（GTK/WebKit 仅 Linux 编译链）

## 远程分支保护

GitHub **Settings → Branches → main** 启用 **Require status checks**：`rust`、`frontend`、`e2e`
