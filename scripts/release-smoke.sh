#!/bin/bash
# Release 构建冒烟：确认 release 配置可编译（无 E2E HTTP、无 env 自动导入依赖）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo build --release"
cargo build --manifest-path src-tauri/Cargo.toml --release

echo "==> frontend production build"
pnpm --dir frontend run build

echo "==> OK: release artifacts ready (E2E HTTP / env import are debug-only)"
