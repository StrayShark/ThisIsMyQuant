//! 结构化日志辅助：贯通 ApiResponse.trace_id。

pub fn log_trace(trace_id: &str, level: &str, msg: &str) {
    match level {
        "error" => log::error!("[trace={trace_id}] {msg}"),
        "warn" => log::warn!("[trace={trace_id}] {msg}"),
        "debug" => log::debug!("[trace={trace_id}] {msg}"),
        _ => log::info!("[trace={trace_id}] {msg}"),
    }
}
