#!/usr/bin/env bash
# push 前完整 CI：Mac 本地检查 + Docker Linux Rust 构建。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo ""
echo "▶ pre-push [1/2]: Mac 本地 CI（frontend + e2e + Rust）…"
bash "$ROOT/scripts/ci-local.sh"

echo ""
echo "▶ pre-push [2/2]: Docker Linux Rust 构建（对齐 CI rust job）…"
bash "$ROOT/scripts/ci-linux-docker.sh"

echo ""
echo "pre-push 全部检查通过，开始推送。"
