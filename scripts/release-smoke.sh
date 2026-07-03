#!/bin/bash
# Release 构建冒烟：确认 release 配置可编译（无 E2E HTTP、无 env 自动导入依赖）
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo build --release"
cargo build --manifest-path src-tauri/Cargo.toml --release

echo "==> frontend production build"
pnpm --dir frontend run build

BUNDLE_DIR="src-tauri/target/release/bundle"
if [[ -d "$BUNDLE_DIR" ]]; then
  echo "==> validate bundle artifacts"
  found=0
  for pattern in "$BUNDLE_DIR"/*/*.dmg "$BUNDLE_DIR"/*/*.msi "$BUNDLE_DIR"/*/*.AppImage; do
    if [[ -f "$pattern" ]]; then
      echo "  found: $pattern"
      found=1
    fi
  done
  if [[ "$found" -eq 0 ]]; then
    echo "WARNING: no .dmg / .msi / .AppImage found under $BUNDLE_DIR"
    echo "  (cargo build --release does not produce installers; run pnpm tauri:build for full bundles)"
  fi
else
  echo "INFO: bundle dir not found: $BUNDLE_DIR (expected for cargo build --release; tauri build creates it)"
fi

BIN="src-tauri/target/release/thisismyquant"
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
  BIN="${BIN}.exe"
fi
if [[ -x "$BIN" ]]; then
  echo "==> binary exists and is executable: $BIN"
else
  echo "WARNING: release binary not found: $BIN"
fi

echo "==> OK: release artifacts ready (E2E HTTP / env import are debug-only)"
