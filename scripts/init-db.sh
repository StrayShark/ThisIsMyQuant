#!/bin/bash
# 确保数据目录存在；SQLite 表结构由 Rust 核心首次启动时自动创建。
set -euo pipefail
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DATA_DIR="$PROJECT_DIR/data"
mkdir -p "$DATA_DIR"
echo "数据目录: $DATA_DIR"
echo "表结构将在首次运行 pnpm tauri:dev 时由 Rust 核心自动初始化。"
