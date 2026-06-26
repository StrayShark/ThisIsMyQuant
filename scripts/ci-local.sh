#!/usr/bin/env bash
# 本地复现 GitHub CI 核心检查；pre-push 钩子会调用此脚本。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo test (lib)"
cargo test --manifest-path src-tauri/Cargo.toml --lib

echo "==> frontend tsc"
pnpm --dir frontend exec tsc --noEmit

echo "==> frontend lint"
pnpm --dir frontend run lint

echo "==> frontend build"
pnpm --dir frontend run build

echo "==> e2e mock"
VITE_E2E_MOCK=true pnpm --dir frontend exec playwright test --project=ui-mock

echo "CI local checks passed."
