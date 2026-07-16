#!/usr/bin/env bash
# Linux apt 重试与镜像回退（Docker 网络抖动 / 官方源不可达）。
set -euo pipefail

apt_sudo() {
  if [[ "$(id -u)" -eq 0 ]]; then
    "$@"
  else
    sudo "$@"
  fi
}

apt_configure() {
  apt_sudo tee /etc/apt/apt.conf.d/80-docker-ci >/dev/null <<'EOF'
Acquire::Retries "5";
Acquire::http::Timeout "120";
Acquire::https::Timeout "120";
EOF
}

apt_use_mirror() {
  local mirror="${1:?mirror url}"
  echo "使用 apt 镜像: ${mirror}" >&2
  if [[ -f /etc/apt/sources.list.d/debian.sources ]]; then
    apt_sudo sed -i "s|http://deb.debian.org/debian|${mirror}|g" /etc/apt/sources.list.d/debian.sources
    apt_sudo sed -i "s|http://deb.debian.org/debian-security|${mirror}-security|g" /etc/apt/sources.list.d/debian.sources || true
  elif [[ -f /etc/apt/sources.list.d/ubuntu.sources ]]; then
    apt_sudo sed -i "s|http://archive.ubuntu.com/ubuntu|${mirror}|g" /etc/apt/sources.list.d/ubuntu.sources
    apt_sudo sed -i "s|http://security.ubuntu.com/ubuntu|${mirror}|g" /etc/apt/sources.list.d/ubuntu.sources
  elif [[ -f /etc/apt/sources.list ]]; then
    local host="${mirror#http://}"
    host="${host#https://}"
    apt_sudo sed -i "s|deb.debian.org|${host%%/*}|g" /etc/apt/sources.list
    apt_sudo sed -i "s|security.debian.org|${host%%/*}|g" /etc/apt/sources.list || true
  fi
}

apt_retry() {
  local max="${APT_RETRY_MAX:-5}"
  local wait="${APT_RETRY_WAIT_SECS:-10}"
  local attempt=1
  while true; do
    if "$@"; then
      return 0
    fi
    if [[ $attempt -ge $max ]]; then
      echo "ERROR: apt 失败（已重试 ${max} 次）: $*" >&2
      return 1
    fi
    echo "apt 重试 ${attempt}/${max}: $*" >&2
    attempt=$((attempt + 1))
    sleep "$wait"
  done
}

apt_get() {
  apt_retry apt_sudo apt-get "$@"
}

apt_update() {
  apt_configure
  local mirror="${APT_MIRROR:-}"
  if apt_get update "$@"; then
    return 0
  fi
  if [[ -n "$mirror" ]]; then
    apt_use_mirror "$mirror"
    apt_get update "$@"
    return $?
  fi
  # 官方源失败时依次尝试常见镜像
  for fallback in \
    "http://mirrors.aliyun.com/ubuntu" \
    "http://mirrors.aliyun.com/debian"; do
    echo "WARN: 官方 apt 源不可用，尝试 ${fallback} ..." >&2
    apt_use_mirror "$fallback"
    if apt_get update "$@"; then
      return 0
    fi
  done
  return 1
}
