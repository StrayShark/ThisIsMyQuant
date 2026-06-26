#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
git -C "$ROOT" config core.hooksPath .githooks
chmod +x "$ROOT/.githooks/pre-push" "$ROOT/scripts/ci-local.sh"
echo "Git hooks installed (core.hooksPath=.githooks)"
