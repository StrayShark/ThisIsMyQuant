#!/usr/bin/env bash
# 本地复现 GitHub CI；pre-push 钩子会调用此脚本。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "ERROR: $*" >&2; exit 1; }

# 防止 .gitignore 误伤源码目录（如 frontend/src/data/）
for f in frontend/src/data/futures.ts frontend/src/data/dimensions.ts; do
  if [[ ! -f "$f" ]]; then
    fail "缺少 $f"
  fi
  if git check-ignore -q "$f" 2>/dev/null; then
    fail "$f 被 .gitignore 忽略，CI 将无法编译"
  fi
done

echo "==> cargo fmt check"
cargo fmt --manifest-path src-tauri/Cargo.toml --check

echo "==> cargo clippy"
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

echo "==> cargo test (lib)"
cargo test --manifest-path src-tauri/Cargo.toml --lib

echo "==> frontend install"
# 非 TTY（pre-push / CI）下须禁用 modules  purge 确认，否则会静默失败
CI=true pnpm --dir frontend install --frozen-lockfile --config.confirmModulesPurge=false

echo "==> frontend typecheck (tsc -b，与 build 一致)"
pnpm --dir frontend exec tsc -b

echo "==> frontend lint"
pnpm --dir frontend run lint

echo "==> frontend unit tests"
pnpm --dir frontend run test

echo "==> frontend build"
pnpm --dir frontend run build

echo "==> e2e mock"
VITE_E2E_MOCK=true pnpm --dir frontend exec playwright test --project=ui-mock

echo ""
echo "CI local checks passed. 可以 push。"
