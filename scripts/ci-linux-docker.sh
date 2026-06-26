#!/usr/bin/env bash
# 在 Docker Ubuntu 中复现 CI rust job（Tauri Linux 系统依赖 + cargo test --lib）。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE="${CI_LINUX_IMAGE:-ubuntu:24.04}"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: 需要安装 Docker Desktop 或 docker CLI" >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "ERROR: Docker 未运行，请先启动 Docker Desktop" >&2
  exit 1
fi

echo "==> Linux CI (Docker ${IMAGE})"
echo "    挂载: ${ROOT}"

docker run --rm \
  -v "${ROOT}:/app" \
  -w /app \
  -e DEBIAN_FRONTEND=noninteractive \
  "${IMAGE}" \
  bash -lc '
set -euo pipefail

echo "==> apt: base tools"
apt-get update -qq
apt-get install -y -qq curl ca-certificates git >/dev/null

echo "==> Tauri Linux dependencies"
bash scripts/tauri-linux-deps.sh

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> rustup (stable)"
  curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

echo "==> cargo test (lib)"
cargo test --manifest-path src-tauri/Cargo.toml --lib

echo ""
echo "Linux Docker CI passed."
'

echo ""
echo "本地 Linux 构建验证通过。"
