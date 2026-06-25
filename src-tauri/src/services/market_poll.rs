use std::sync::Arc;

use tokio::sync::{Mutex};
use tokio::task::JoinHandle;

use crate::adapters::AkshareClient;
use crate::engine::KlineAggregator;
use crate::models::Tick;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Debug)]
pub struct PollStatusSnapshot {
    pub running: bool,
    pub interval: f64,
    pub symbols: Vec<String>,
    pub symbol_count: usize,
}

pub struct MarketPollHandle {
    symbols: Arc<Mutex<Vec<String>>>,
    interval: f64,
    _task: JoinHandle<()>,
}

impl MarketPollHandle {
    pub fn start(
        app: AppHandle,
        akshare: AkshareClient,
        mut initial_symbols: Vec<String>,
        interval_secs: f64,
    ) -> Self {
        initial_symbols = initial_symbols
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        let symbols = Arc::new(Mutex::new(initial_symbols.clone()));
        let symbols_task = symbols.clone();
        let interval = interval_secs.max(1.0);
        let task = tokio::spawn(async move {
            let mut agg = KlineAggregator::new();
            let dur = tokio::time::Duration::from_secs_f64(interval);
            loop {
                let syms = symbols_task.lock().await.clone();
                if syms.is_empty() {
                    tokio::time::sleep(dur).await;
                    continue;
                }
                for sym in syms {
                    match akshare.fetch_latest_tick(&sym).await {
                        Ok(Some(tick)) => emit_ticks(&app, &mut agg, tick),
                        Ok(None) => log::debug!("no tick for {sym}"),
                        Err(e) => log::debug!("tick fetch failed {sym}: {e}"),
                    }
                }
                tokio::time::sleep(dur).await;
            }
        });
        log::info!(
            "MarketPoll started: {} symbols, interval={interval}s",
            initial_symbols.len()
        );
        Self {
            symbols,
            interval,
            _task: task,
        }
    }

    pub async fn subscribe(&self, new_symbols: Vec<String>) {
        let mut syms = self.symbols.lock().await;
        for s in new_symbols {
            let lower = s.to_lowercase();
            if !syms.contains(&lower) {
                syms.push(lower);
            }
        }
    }

    pub async fn status(&self) -> PollStatusSnapshot {
        let syms = self.symbols.lock().await.clone();
        PollStatusSnapshot {
            running: true,
            interval: self.interval,
            symbols: syms.clone(),
            symbol_count: syms.len(),
        }
    }
}

fn emit_ticks(app: &AppHandle, agg: &mut KlineAggregator, tick: Tick) {
    for ev in agg.on_tick(&tick) {
        let _ = app.emit("kline-update", &ev);
    }
}
