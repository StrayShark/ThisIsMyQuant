//! 批量分析任务（多品种顺序触发）。

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::models::BatchJobStatus;
use crate::services::run_analysis;
use crate::state::AppState;

pub struct BatchAnalysisHandle {
    status: Arc<Mutex<BatchJobStatus>>,
}

impl BatchAnalysisHandle {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(BatchJobStatus::default())),
        }
    }

    pub async fn get_status(&self) -> BatchJobStatus {
        self.status.lock().await.clone()
    }

    pub fn spawn(
        &self,
        state: Arc<AppState>,
        symbols: Vec<String>,
        trigger: String,
        provider: Option<String>,
    ) -> Result<(), String> {
        let mut st = self.status.try_lock().map_err(|_| "batch busy")?;
        if st.running {
            return Err("batch job already running".into());
        }
        st.running = true;
        st.total = symbols.len();
        st.completed = 0;
        st.current_symbol = None;
        st.errors.clear();
        drop(st);

        let status = self.status.clone();
        tauri::async_runtime::spawn(async move {
            for sym in symbols {
                {
                    let mut s = status.lock().await;
                    s.current_symbol = Some(sym.clone());
                }
                match run_analysis(
                    &state,
                    None,
                    &sym,
                    &trigger,
                    provider.as_deref(),
                    false,
                    None,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        let mut s = status.lock().await;
                        s.errors.push(format!("{sym}: {e}"));
                    }
                }
                let mut s = status.lock().await;
                s.completed += 1;
            }
            let mut s = status.lock().await;
            s.running = false;
            s.current_symbol = None;
        });
        Ok(())
    }
}
