# 贡献规范

## CI 门禁

推送至 `main` / `master` 前须通过 CI。本地检查：

```bash
bash scripts/ci-local.sh
```

安装 pre-push 钩子（推送前自动运行上述检查）：

```bash
bash scripts/install-githooks.sh
```

## 修改前端依赖

`frontend/pnpm-lock.yaml` 已纳入版本控制。更新依赖后：

```bash
cd frontend && pnpm install
git add frontend/package.json frontend/pnpm-lock.yaml
```

## GitHub Actions

工作流定义见 `.github/workflows/ci.yml`。远程仓库建议为 `main` 分支启用 **Require status checks to pass**（需仓库管理员在 GitHub Settings → Branches 配置）。
