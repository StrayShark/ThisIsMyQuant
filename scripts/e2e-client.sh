#!/bin/bash
# 启动 Tauri 客户端并运行 Live E2E（各业务模块 + LLM 明日/短期分析）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> 同步环境变量"
bash scripts/sync-env.sh

echo "==> 清理占用端口"
lsof -ti:5173 2>/dev/null | xargs kill -9 2>/dev/null || true
lsof -ti:17845 2>/dev/null | xargs kill -9 2>/dev/null || true

echo "==> Playwright 客户端 Live E2E"
cd frontend
pnpm exec playwright test -c playwright.client.config.ts --project=client-live "$@"
