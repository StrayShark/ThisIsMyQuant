#!/usr/bin/env bash
# 在 Docker 中复现 CI rust job（Tauri Linux 系统依赖 + cargo test --lib）。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PLATFORM="${CI_LINUX_PLATFORM:-linux/amd64}"
IMAGE="${CI_LINUX_IMAGE:-thisismyquant-ci-linux:local}"
DOCKERFILE="${ROOT}/scripts/docker/Dockerfile.ci-linux"
APT_MIRROR="${APT_MIRROR:-http://mirrors.aliyun.com/debian}"
HOST_CARGO="${CARGO_HOME_HOST:-${HOME}/.cargo}"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: 需要安装 Docker Desktop 或 docker CLI" >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "ERROR: Docker 未运行，请先启动 Docker Desktop" >&2
  exit 1
fi

if ! docker image inspect "${IMAGE}" >/dev/null 2>&1; then
  echo "==> 首次运行：构建 CI Linux 镜像（含 Tauri 依赖，约 3–10 分钟）"
  echo "    platform=${PLATFORM} mirror=${APT_MIRROR}"
  docker build \
    --platform "${PLATFORM}" \
    --build-arg "APT_MIRROR=${APT_MIRROR}" \
    -f "${DOCKERFILE}" \
    -t "${IMAGE}" \
    "${ROOT}"
fi

CARGO_MOUNTS=()
if [[ -d "${HOST_CARGO}/registry" ]]; then
  echo "    复用本机 Cargo 缓存: ${HOST_CARGO}/registry"
  CARGO_MOUNTS+=(-v "${HOST_CARGO}/registry:/usr/local/cargo/registry")
fi
if [[ -d "${HOST_CARGO}/git" ]]; then
  CARGO_MOUNTS+=(-v "${HOST_CARGO}/git:/usr/local/cargo/git")
fi

echo "==> Linux CI (Docker ${IMAGE}, platform=${PLATFORM})"
echo "    挂载: ${ROOT}"
echo "    重建镜像: pnpm test:ci:linux:rebuild"

docker run --rm \
  --platform "${PLATFORM}" \
  -v "${ROOT}:/app" \
  "${CARGO_MOUNTS[@]}" \
  -w /app \
  -e CARGO_TERM_COLOR=always \
  -e CARGO_HOME=/usr/local/cargo \
  "${IMAGE}" \
  bash -c '
set -euo pipefail
export PATH="/usr/local/cargo/bin:${PATH}"
mkdir -p "${CARGO_HOME}"
cp scripts/docker/cargo-config.toml "${CARGO_HOME}/config.toml"

echo "==> cargo fetch (locked)"
cargo fetch --manifest-path src-tauri/Cargo.toml --locked

echo "==> cargo test (lib)"
cargo test --manifest-path src-tauri/Cargo.toml --lib

echo ""
echo "Linux Docker CI passed."
'

echo ""
echo "本地 Linux 构建验证通过。"
