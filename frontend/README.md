# ThisIsMyQuant 前端

React + Vite + TypeScript + Lightweight Charts。数据通过 **Tauri invoke** 与 Rust 核心通信。

## 开发

桌面应用请在**仓库根目录**运行：

```bash
pnpm tauri:dev
```

前端单独 `pnpm dev` 仅用于 UI 调试，**无 Rust 核心时 API 不可用**（E2E 使用 `VITE_E2E_MOCK=true`）。

## 脚本

| 命令 | 作用 |
|---|---|
| `pnpm tsc` | 类型检查 |
| `pnpm build` | 生产构建（Tauri 打包时自动调用） |
| `pnpm test:e2e` | Playwright（Mock 模式） |

## 设计

样式见 `src/design/tokens.css` 与 `docs/DESIGN.md`。
