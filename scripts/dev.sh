#!/bin/bash
# 启动 Tauri 桌面应用（Rust 核心 + Vite 前端）
set -euo pipefail
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"
pnpm install
pnpm tauri:dev
