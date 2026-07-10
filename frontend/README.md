# 前端

React + Vite + TypeScript 前端，负责 ThisIsMyQuant 的国内期货与 A 股模拟盘、本地数据库和专业分析工作台 UI。

页面矩阵包括：总览、期货行情、模拟盘、复盘、回放、因子、资讯、日历、异动、A 股总览、行业概念、个股、筛选器、财报、模拟组合、助手、报告、品种、数据库、状态、设置。所有数据请求必须通过 `frontend/src/api/client.ts` 调用 Tauri command；前端不直接访问 AKShare、金十、Tushare 或 LLM，也不得实现真实交易接口。

常用命令：

```bash
pnpm --dir frontend dev
pnpm --dir frontend tsc
pnpm --dir frontend lint
pnpm --dir frontend test
```

完整应用请在仓库根目录执行：

```bash
pnpm tauri:dev
```

更多说明见根目录 [README](../README.md) 与 [USAGE](../docs/USAGE.md)。
