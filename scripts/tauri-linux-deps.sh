#!/usr/bin/env bash
# Tauri 2 在 Linux 上编译所需的系统库（CI / Release Ubuntu 矩阵）。
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  exit 0
fi

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
