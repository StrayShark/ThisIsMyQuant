#!/usr/bin/env bash
# 配置 Git 使用仓库内 hooks；pnpm install 时自动调用。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  echo "install-githooks: 非 git 仓库，跳过" >&2
  exit 0
fi

git -C "$ROOT" config core.hooksPath .githooks
chmod +x "$ROOT/.githooks/pre-push" "$ROOT/scripts/ci-local.sh" 2>/dev/null || true

if [[ "${CI:-}" == "true" || "${CI:-}" == "1" ]]; then
  exit 0
fi

echo "Git hooks 已启用：push 前将自动运行 pnpm test:ci"
