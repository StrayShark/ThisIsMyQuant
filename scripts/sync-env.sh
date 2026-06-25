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

# 同步 LLM Keys
for key in DOUBAO_API_KEY MINIMAX_API_KEY OPENAI_API_KEY DEEPSEEK_API_KEY QWEN_API_KEY JIN10_MCP_TOKEN; do
  val=$(extract "$key")
  if [ -n "$val" ]; then
    # 用 | 作为 sed 分隔符避免 Key 中的 / 冲突
    sed -i.bak "s|^${key}=.*|${key}=${val}|" "$TARGET" 2>/dev/null || \
    sed -i '' "s|^${key}=.*|${key}=${val}|" "$TARGET"
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

echo ""
echo "完成。请编辑 $TARGET 填入 CTP 仿真账号等其余配置。"
