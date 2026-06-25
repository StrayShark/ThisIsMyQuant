# UI 设计语言（DESIGN）

> 版本：v3.0 · 2026-06-24
> **源规范**：`~/global_design/awesome-design-md/design-md/vercel/DESIGN.md`（Vercel-Inspired，已纳入下文）
> **项目变体**：深色仪表盘（参考 [Vercel Dashboard](https://vercel.com/dashboard)），非 marketing 浅色站
> **组件体系**：[shadcn/ui](https://ui.shadcn.com/) New York + 项目 token（`frontend/src/design/tokens.css`）

---

## 0. 规范层级

| 层级 | 说明 |
|---|---|
| **L1 — Vercel 源规范** | 下文 YAML + Overview/Do-Don't，来自 global_design，默认浅色 marketing |
| **L2 — 深色仪表盘映射** | 本项目将 L1 token 映射为 dashboard 深色值（§1） |
| **L3 — 产品扩展** | AI 时间线五色、行情涨跌色、220px 侧栏等业务规则（§2–§7） |
| **L4 — 合规审查** | 代码对照 L2+L3 的审查结论（§9） |

---

## 1. Vercel 源规范（global_design）

> 完整 prose 见 `~/global_design/awesome-design-md/design-md/vercel/DESIGN.md`。
> 以下为 machine-readable token 块与核心原则摘要。

```yaml
---
version: alpha
name: Vercel-Inspired-design-analysis
description: Vercel design language — stark black-and-ink on near-white canvas, mesh gradient decoration, Geist sans + Geist Mono.

colors:
  primary: "#171717"
  on-primary: "#ffffff"
  ink: "#171717"
  body: "#4d4d4d"
  mute: "#888888"
  hairline: "#ebebeb"
  hairline-strong: "#a1a1a1"
  canvas: "#ffffff"
  canvas-soft: "#fafafa"
  canvas-soft-2: "#f5f5f5"
  link: "#0070f3"
  link-deep: "#0761d1"
  link-bg-soft: "#d3e5ff"
  success: "#0070f3"
  error: "#ee0000"
  error-soft: "#f7d4d6"
  error-deep: "#c50000"
  warning: "#f5a623"
  warning-soft: "#ffefcf"
  warning-deep: "#ab570a"
  violet: "#7928ca"
  cyan: "#50e3c2"
  cyan-soft: "#aaffec"
  highlight-pink: "#ff0080"
  gradient-develop-start: "#007cf0"
  gradient-develop-end: "#00dfd8"
  gradient-preview-start: "#7928ca"
  gradient-preview-end: "#ff0080"
  gradient-ship-start: "#ff4d4d"
  gradient-ship-end: "#f9cb28"
  selection-bg: "#171717"
  selection-fg: "#f2f2f2"

typography:
  display-xl: { fontFamily: "Geist, Inter, system-ui", fontSize: 48px, fontWeight: 600, letterSpacing: -2.4px }
  display-lg: { fontFamily: "Geist, Inter, system-ui", fontSize: 32px, fontWeight: 600, letterSpacing: -1.28px }
  display-md: { fontSize: 24px, fontWeight: 600 }
  body-md: { fontSize: 16px, fontWeight: 400, lineHeight: 24px }
  body-sm: { fontSize: 14px, fontWeight: 400, lineHeight: 20px, letterSpacing: -0.28px }
  caption: { fontSize: 12px, fontWeight: 400 }
  caption-mono: { fontFamily: "Geist Mono, ui-monospace", fontSize: 12px }
  code: { fontFamily: "Geist Mono", fontSize: 13px }
  button-md: { fontSize: 14px, fontWeight: 500 }
  button-lg: { fontSize: 16px, fontWeight: 500 }

rounded:
  xs: 4px
  sm: 6px
  md: 8px
  lg: 12px
  xl: 16px
  pill-sm: 64px
  pill: 100px
  full: 9999px

spacing:
  xxs: 4px
  xs: 8px
  sm: 12px
  md: 16px
  lg: 24px
  xl: 32px
  2xl: 40px
  3xl: 48px
  4xl: 64px
  5xl: 96px
  6xl: 128px
  section: 192px

components:
  nav-bar: { height: 64px, padding: "12px 24px" }
  form-input: { height: 40px, rounded: 6px }
  form-input-sm: { height: 32px }
  form-input-lg: { height: 48px }
  button-primary: { rounded: pill, typography: button-lg }
  button-primary-sm: { rounded: pill, typography: button-md }
  nav-cta-signup: { height: 28px, rounded: 6px }
  card-marketing: { rounded: 8px, padding: 24px }
  ex-app-shell-row: { activeIndicator: primary, rounded: 6px }
---
```

### Vercel 核心原则（源规范）

- **色彩**：Ink `#171717` 为 marketing 主 CTA；Link Blue `#0070f3` 为链接/成功语义；Cyan `#50e3c2`、Error `#ee0000` 为品牌梯度/语义色。
- **字体**：Geist（400/500/600）+ Geist Mono（技术层）；Display 负字距；正文不用 mono。
- **圆角**：Marketing CTA 用 `pill` 100px；应用内控件用 `sm` 6px / `md` 8px。
- **间距**：4px 基准；section 192px。
- **Elevation**：浅色站用 stacked shadow + inset hairline；**本项目 L2 刻意禁用阴影**。
- **Do**：单一 ink 主 CTA、分层 surface、mono 仅技术标签。
- **Don't**：不要 weight 700+、不要 icon 级渐变、不要 heavy drop-shadow。

---

## 2. 深色仪表盘 Token 映射（L1 → L2）

本项目为 **in-app dashboard**，采用 Vercel Dashboard 极性翻转，而非 marketing 浅色站。

| Vercel 源 (L1) | 本项目 (L2) | CSS 变量 | 说明 |
|---|---|---|---|
| `canvas` `#ffffff` | `#000000` | `--color-canvas` | 页面真黑底 |
| `canvas-soft` `#fafafa` | `#0a0a0a` | `--color-canvas-soft` | 面板/图表区 |
| `primary` `#171717` (ink CTA) | `#0070f3` | `--color-primary` | Dashboard 主 CTA 用 Link Blue |
| `ink` `#171717` | `#ededed` | `--color-ink` | 标题/强调正文 |
| `body` `#4d4d4d` | `#a1a1a1` | `--color-body` | 次要正文 |
| `mute` `#888888` | `#888888` | `--color-muted` | 一致 |
| `hairline` `#ebebeb` | `#333333` | `--color-hairline` | 深色描边 |
| `hairline-strong` `#a1a1a1` | `#404040` | `--color-hairline-strong` | 强描边 |
| `link` `#0070f3` | `#0070f3` | `--color-link` | 一致 |
| `link-deep` `#0761d1` | `#0761d1` | `--color-primary-active` | 一致 |
| `link-bg-soft` `#d3e5ff` | `#0d2847` | `--color-link-bg-soft` | 深色浅底 |
| `cyan` `#50e3c2` | `#50e3c2` | `--color-up` | 涨/成功 |
| `error` `#ee0000` | `#ee0000` | `--color-down` | 跌/错误 |
| stacked shadow | **无** | — | Dashboard 仅用 1px 细线 |
| `nav-bar` 64px | TopBar 48px (`h-12`) | — | 紧凑仪表盘 |
| marketing `pill` CTA | shadcn `rounded-md` 6–8px | — | in-app 尺度 |

### 有意保留的 L3 扩展

| Token | 值 | 用途 |
|---|---|---|
| `--color-timeline-*` | 五色柔和 | 仅 AI 分析时间线 |
| `--color-surface-elevated` | `#171717` | 下拉/浮层（对应 Vercel ink） |
| 侧栏宽 | 220px | AppShell 固定宽度 |

---

## 3. 设计原则（L2 + L3）

1. **仪表盘克制**：像 Vercel / Linear 控制台，信息分层清晰。
2. **单一电压色**：Vercel Blue 用于主 CTA、链接、焦点环。
3. **细线即深度**：禁止投影；1px `#333` + `#0a0a0a` 卡片 vs `#000` 底。
4. **数字用等宽**：价格、合约、K 线坐标用 Geist Mono / JetBrains Mono。
5. **涨跌色受限**：`#50e3c2` / `#ee0000` 仅用于价格与 K 线，不用于按钮/链接。
6. **AI 时间线**：五色药丸仅用于分析步骤，不作系统操作色。

---

## 4. 字体

| 层级 | Vercel token | 本项目 | 用途 |
|---|---|---|---|
| 页面标题 | display-md 24px/600 | 24px/600 | 页头 |
| 区块标题 | display-sm 20px/600 | 18px/600 | 卡片组 |
| 正文 | body-sm 14px | 14px（body 默认） | 内容 |
| 注释 | caption 12px | 11–12px | 元信息 |
| 代码/价格 | code 13px mono | 13px mono | OHLCV、合约 |

- **Sans 回退**：Geist → Inter → system-ui（`index.css` 现加载 Inter）
- **Mono 回退**：Geist Mono → JetBrains Mono

---

## 5. 布局

### AppShell（220px 侧栏）

```
┌──────────┬────────────────────────────────────────────┐
│ ThisIsMyQuant │  TopBar（搜索 h-12）                     │
│          │────────────────────────────────────────────│
│ 行情/报告 │              主内容区                       │
│ …        │                                            │
│ ● 数据源  │                                            │
└──────────┴────────────────────────────────────────────┘
  220px              flex-1
```

### 间距（对齐 Vercel spacing）

`xxs 4 · xs 8 · sm 12 · md 16 · lg 24 · xl 32 · 2xl 40 · 3xl 48 · section 80`（dashboard 节距较 marketing 收紧）

### 圆角（in-app 尺度）

| Token | 值 | 用途 |
|---|---|---|
| `--radius-sm` | 6px | 按钮、输入、导航项 |
| `--radius-md` | 8px | 卡片（shadcn `--radius: 0.5rem`） |
| `--radius-lg` | 12px | 大面板 |
| `--radius-pill` | 9999px | 时间线 badge、状态点 |

---

## 6. 组件对照（Vercel primitive → 项目实现）

| Vercel 组件 | 项目实现 | 文件 |
|---|---|---|
| `ex-app-shell-row` | NavLink ghost + active `bg-muted/40` | `AppShell.tsx` |
| `form-input-sm` (32px) | Input `h-9` (36px) | `TopBar.tsx`, `input.tsx` |
| `button-primary-sm` | Button default `bg-primary` | `button.tsx` |
| `card-marketing` | Card / `.panel` 无 shadow | `card.tsx`, `index.css` |
| `tab-ghost` | TabsList muted 底 | `tabs.tsx` |
| `badge-secondary` | Badge secondary | `badge.tsx` |
| `code-editor-mockup` | ChartPanel 深色底 | `ChartPanel.tsx` |

---

## 7. Do / Don't（项目）

### Do
- 保持真黑 `#000` 页面底
- 主操作只用 Vercel Blue（L2 映射后的 primary）
- 价格/代码用等宽字体
- 图表/K 线颜色引用 CSS 变量
- 用 shadcn 组件保持一致性

### Don't
- 不要引入第二个品牌动作色
- 不要给卡片加投影（与 Vercel marketing 不同，dashboard 变体）
- 不要把涨跌色用在按钮/链接上
- 不要把时间线柔和色用在非 AI 场景
- 不要在 TS/Canvas 中硬编码 hex（应读 token）

---

## 8. Token 落地

| 文件 | 职责 |
|---|---|
| `frontend/src/design/tokens.css` | L2 CSS 变量 + shadcn HSL |
| `frontend/src/index.css` | 字体、`.panel`、时间线 pill |
| `frontend/tailwind.config.ts` | Tailwind 颜色/radius 映射 |
| `frontend/components.json` | shadcn New York / neutral |

---

## 9. UI 合规审查（2026-06-24）

对照 **L2 深色映射 + L3 产品规则** 审查 `frontend/src/`。

### 9.1 符合 ✅

| 项 | 证据 |
|---|---|
| 真黑页面底 | `tokens.css` `--background: 0 0% 0%`；`AppShell` `bg-background` |
| 无卡片阴影 | 全库无 `shadow-*`（src 内）；`.panel` 仅 border |
| Vercel Blue 主 CTA | `--color-primary: #0070f3`；Button `bg-primary` |
| Link / 焦点环 | `--ring: 211 100% 48%` |
| 细线分割 | `border-border` → `--color-hairline #333` |
| 侧栏 220px | `AppShell` `w-[220px]` |
| 涨跌色值正确 | token `#50e3c2` / `#ee0000` 与 Vercel cyan/error 一致 |
| AI 时间线专用色 | `AnalysisTimeline` + `.pill-*` 类 |
| 卡片圆角 8–12px | Card `rounded-lg`；符合 in-app `md/lg` |
| 字体回退链 | `--font-sans` / `--font-mono` 定义正确 |
| 深色模式固定 | shadcn HSL 变量为 dark 值 |

### 9.2 部分符合 ⚠️

| 项 | 规范 | 现状 | 建议 |
|---|---|---|---|
| 主 CTA 色语义 | L1 ink `#171717`；L2 映射为 blue | 已按 L2 执行 | 文档已说明，无需改 |
| 输入框高度 | `form-input-sm` 32px / `form-input` 40px | Input 默认 `h-9`(36px)；TopBar 同 | 改为 `h-8`(32) 或 `h-10`(40) 对齐 Vercel 档位 |
| TopBar 高度 | Vercel nav 64px | `h-12`(48px) | 可接受（dashboard 紧凑）；若需严格对齐改为 `h-16` |
| 按钮圆角 | in-app `rounded.sm` 6px | shadcn `rounded-md` (~8px) | 可改为 `rounded-[6px]` 或 `--radius-sm` |
| 导航激活态 | `ex-app-shell-row` 左缘 primary 指示条 | 整行 `bg-muted/40` | 可选：加 `border-l-2 border-primary` |
| 字体加载 | Geist 本体 | 仅 Google Inter + JetBrains | 接入 `geist` npm 或 `@fontsource` |
| 间距 token | Vercel `md:16px` | `--space-md:20px` | 将 `--space-md` 改为 `16px` 与源规范一致 |
| Badge 圆角 | Vercel `rounded.full` | `rounded-md` | 元数据 badge 可改 `rounded-full` |
| 文档状态文案 | 数据源描述 | 侧栏仍写「AKShare」 | 更新为「新浪 · 金十」等与后端一致 |

### 9.3 不符合 ❌

| 项 | 规范 | 现状 | 修复 |
|---|---|---|---|
| ChartPanel 硬编码色 | 应使用 CSS 变量 | `#0a0a0a`、`#888`、`#262626`、`#333`、`#50e3c2`、`#ee0000` 内联 | 读 `getComputedStyle(document.documentElement).getPropertyValue('--color-*')` 或集中 `chartTheme` 常量 |
| Badge up/down 硬编码 | token 衍生 | `bg-[#0d3329]`、`bg-[#3d0a0a]` | 新增 `--color-up-bg` / `--color-down-bg` |
| index.css pill 文字色 | 应用 token | `color: #171717` 硬编码 | 改为 `var(--color-ink)` 或专用 `--color-pill-fg` |
| DESIGN 侧栏示意 | 「● CTP」 | 实际为 AKShare/金十 | 更新文档 ASCII 图（§5 已修正方向） |

### 9.4 合规评分

| 维度 | 得分 | 说明 |
|---|---|---|
| 色彩系统 | 85% | token 完整；ChartPanel/Badge 有硬编码 |
| 字体排版 | 75% | 层级合理；未加载 Geist 本体 |
| 布局间距 | 80% | 侧栏/面板 OK；`space-md` 偏差 |
| 组件形态 | 78% | shadcn 一致；圆角/输入高度略偏 |
| 深度/Elevation | 95% | 无阴影，符合 L2 |
| **综合** | **82%** | 核心仪表盘气质达标；细节 token 化待补 |

### 9.5 优先修复清单

1. **P0** — `ChartPanel.tsx`：K 线/网格/背景色改为读取 `tokens.css` 变量
2. **P1** — `badge.tsx`：up/down 背景色 token 化
3. **P1** — `tokens.css`：`--space-md: 16px`；可选增加 `--color-up-bg` / `--color-down-bg`
4. **P2** — 加载 Geist 字体；Input 高度对齐 32/40px 档
5. **P2** — `AppShell` 导航 active 左缘指示条；侧栏状态文案更新

---

## 10. 可访问性

- 正文对比：`#ededed` on `#000` ≈ 15:1（AAA）
- 主 CTA 高度：Button default `h-9`(36px)，lg `h-10`(40px) — 建议 primary 用 lg
- 焦点环：`ring-primary`（Vercel Blue）
- 时间线药丸带文字标签，不依赖颜色单独传达信息
