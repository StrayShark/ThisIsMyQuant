#!/bin/bash
# ============================================================
# ThisIsMyQuant — 从 ~/global_env/.env 同步生成项目 .env
# ============================================================
set -euo pipefail

GLOBAL_ENV="$HOME/global_env/.env"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="$PROJECT_DIR/.env"

if [ ! -f "$GLOBAL_ENV" ]; then
  echo "ERROR: 未找到 $GLOBAL_ENV"
  echo "请确认全局环境文件存在。"
  exit 1
fi

echo "同步来源: $GLOBAL_ENV"
echo "目标:     $TARGET"

# 读取全局 .env 中已配置的 LLM Key（仅非空），写入项目 .env 模板
# 使用 .env.example 作为模板，替换其中的占位
TEMPLATE="$PROJECT_DIR/.env.example"
if [ ! -f "$TEMPLATE" ]; then
  echo "ERROR: 未找到模板 $TEMPLATE"
  exit 1
fi

cp "$TEMPLATE" "$TARGET"

# 从全局环境提取已配置的 Key 并写入
extract() {
  local key="$1"
  local val
  val=$(grep -E "^${key}=" "$GLOBAL_ENV" 2>/dev/null | head -1 | cut -d'=' -f2- || true)
  echo "$val"
}

# 同步 LLM Key（本地调试用）与金十 Token；项目 .env 已有非空值时保留
for key in DOUBAO_API_KEY DOUBAO_BASE_URL DOUBAO_MODEL \
  MINIMAX_API_KEY MINIMAX_BASE_URL MINIMAX_MODEL \
  OPENAI_API_KEY OPENAI_BASE_URL OPENAI_MODEL \
  DEEPSEEK_API_KEY DEEPSEEK_BASE_URL DEEPSEEK_MODEL \
  QWEN_API_KEY QWEN_BASE_URL QWEN_MODEL \
  KIMI_API_KEY KIMI_BASE_URL KIMI_MODEL \
  MOONSHOT_API_KEY MOONSHOT_BASE_URL MOONSHOT_MODEL \
  DEFAULT_LLM_PROVIDER JIN10_MCP_TOKEN; do
  val=$(extract "$key")
  if [ -n "$val" ]; then
    if grep -q "^${key}=" "$TARGET" 2>/dev/null; then
      current=$(grep -E "^${key}=" "$TARGET" 2>/dev/null | head -1 | cut -d'=' -f2- || true)
      if [ -n "$current" ]; then
        continue
      fi
    fi
    if grep -q "^${key}=" "$TARGET" 2>/dev/null; then
      sed -i.bak "s|^${key}=.*|${key}=${val}|" "$TARGET" 2>/dev/null || \
      sed -i '' "s|^${key}=.*|${key}=${val}|" "$TARGET"
    else
      echo "${key}=${val}" >> "$TARGET"
    fi
    echo "  ✓ 已同步 $key"
  fi
done
rm -f "$TARGET.bak"

# 生成 ENCRYPTION_KEY（若为空）
if grep -q "^ENCRYPTION_KEY=$" "$TARGET"; then
  KEY=$(openssl rand -base64 32)
  sed -i.bak "s|^ENCRYPTION_KEY=$|ENCRYPTION_KEY=${KEY}|" "$TARGET" 2>/dev/null || \
  sed -i '' "s|^ENCRYPTION_KEY=$|ENCRYPTION_KEY=${KEY}|" "$TARGET"
  rm -f "$TARGET.bak"
  echo "  ✓ 已生成 ENCRYPTION_KEY"
fi

echo "完成。行情源固定为 MARKET_FEED=akshare_poll。LLM Key 已同步至 .env（本地 debug 会自动导入 SQLite）；也可在应用内 Landing/设置页配置。"
