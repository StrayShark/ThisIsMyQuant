//! Debug 专用：127.0.0.1 E2E HTTP 桥，供 Playwright 在客户端运行时调用。

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::e2e_suite::run_client_e2e_suite;
use crate::state::AppState;

pub const E2E_HTTP_PORT: u16 = 17_845;

pub fn spawn_e2e_http_server(state: Arc<AppState>) {
    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{E2E_HTTP_PORT}");
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                log::warn!("e2e http bind {addr}: {e}");
                return;
            }
        };
        log::info!("e2e http listening on {addr}");
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                continue;
            };
            let st = state.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 16_384];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    return;
                }
                let req = String::from_utf8_lossy(&buf[..n]);
                let body = if req.contains("GET /health") {
                    r#"{"status":"ok"}"#.to_string()
                } else if req.contains("POST /e2e/run") {
                    let symbol = extract_symbol(&req).unwrap_or_else(|| "rb0".into());
                    let symbols = extract_symbols(&req);
                    let report = run_client_e2e_suite(&st, &symbol, &symbols).await;
                    serde_json::to_string(&report)
                        .unwrap_or_else(|e| format!(r#"{{"ok":false,"error":"{e}"}}"#))
                } else {
                    r#"{"error":"not found"}"#.to_string()
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    });
}

fn extract_symbol(req: &str) -> Option<String> {
    extract_json(req)?
        .get("symbol")?
        .as_str()
        .map(str::to_string)
}

fn extract_symbols(req: &str) -> Vec<String> {
    extract_json(req)
        .and_then(|v| {
            v.get("symbols")?.as_array().map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
        })
        .unwrap_or_default()
}

fn extract_json(req: &str) -> Option<serde_json::Value> {
    let body_start = req.find("\r\n\r\n")? + 4;
    let body = req.get(body_start..)?.trim();
    if body.is_empty() {
        return None;
    }
    serde_json::from_str(body).ok()
}
