# 模块通信机制（COMMUNICATION）

> 版本：v2.0 · Tauri IPC + Events

---

## 1. 通信总览

```
前端 ── invoke(command, args) ──▶ Rust commands.rs
前端 ◀── emit(event, payload) ──  Rust services / analysis_runner

Rust 内部：
  services ──▶ adapters (HTTP 拉取)
  services ──▶ db (SQLite)
  analysis_runner ──▶ llm (流式) ──▶ emit analysis-delta / analysis-done
  market_poll ──▶ emit kline-update
```

无 REST / WebSocket 服务端；E2E 测试使用 `VITE_E2E_MOCK=true` 的内存 Mock。

---

## 2. Tauri Commands（请求/响应）

封装于 `frontend/src/api/client.ts`，统一响应：

```json
{ "code": 0, "message": "ok", "data": { } }
```

| Command | 说明 |
|---|---|
| `get_health` | 健康检查、数据源与轮询状态 |
| `list_products` | 品种目录（按 tier 过滤） |
| `list_contracts` | 合约列表 |
| `get_klines` | 历史 K 线 |
| `list_reports` / `get_report` | 分析报告 |
| `list_news` | 资讯 |
| `list_dimensions` / `list_dimension_facts` | 分析维度 |
| `list_followups` | Copilot 追问历史 |
| `get_settings` | 只读配置快照 |
| `market_subscribe` | 订阅轮询品种 |
| `trigger_analysis` | 同步触发分析 |
| `stream_analysis` | 异步流式分析（见事件） |
| `analysis_followup` | 异步追问（见事件） |

---

## 3. Tauri Events（推送）

| 事件 | 方向 | 说明 |
|---|---|---|
| `app-ready` | Rust → 前端 | 核心初始化完成 |
| `kline-update` | Rust → 前端 | K 线增量 `{ type, symbol, interval, data }` |
| `analysis-delta` | Rust → 前端 | 流式分析文本片段 |
| `analysis-done` | Rust → 前端 | 分析完成含 report_id |
| `analysis-error` | Rust → 前端 | 分析失败 |
| `followup-delta` / `followup-done` / `followup-error` | Rust → 前端 | 追问流式 |
| `notification` | Rust → 前端 | 系统通知（报告完成等） |

前端 `ws/socket.ts` 监听 `kline-update` 与 `notification`，兼容原 WS 消息形状。

---

## 4. 流式分析时序

```
前端 invoke("stream_analysis", { symbol, trigger })
  → Rust spawn run_analysis(stream=true)
  → emit analysis-delta (多次)
  → emit analysis-done
  → emit notification
```

前端用 `ReadableStream` 适配 SSE 消费逻辑（AiPanel）。

---

## 5. K 线实时更新

```
market_poll 定时拉取 AKShare
  → 合成/更新 K 线
  → emit("kline-update", WsMessage)
ChartPanel listen → candleRef.update / volumeRef.update
```

---

## 6. 错误处理

- Command 返回 `ApiResponse { code != 0, message }` 表示业务错误
- 流式任务失败通过 `analysis-error` / `followup-error` 传递
- 前端 `api/client.ts` 的 `unwrap()` 抛出 Error 供 react-query 处理
