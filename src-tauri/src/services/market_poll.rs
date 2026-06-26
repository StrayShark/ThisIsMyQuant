use std::sync::{Arc, Mutex as StdMutex};

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use tauri::{AppHandle, Emitter};

use crate::adapters::MarketFeed;
use crate::engine::KlineAggregator;
use crate::models::{Tick, TickUpdateEvent};
use crate::services::AnomalyWatcher;
use crate::state::AppState;

#[derive(Clone, Debug)]
pub struct PollStatusSnapshot {
    pub running: bool,
    pub interval: f64,
    pub symbols: Vec<String>,
    pub symbol_count: usize,
    pub feed_source: String,
}

pub struct MarketPollHandle {
    symbols: Arc<Mutex<Vec<String>>>,
    interval: f64,
    feed_source: String,
    task: Arc<StdMutex<Option<JoinHandle<()>>>>,
}

impl MarketPollHandle {
    pub fn abort(&self) {
        if let Ok(mut guard) = self.task.lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
                log::info!("MarketPoll aborted");
            }
        }
    }

    pub fn start(
        app: AppHandle,
        feed: Box<dyn MarketFeed>,
        state: Arc<AppState>,
        anomaly: Option<Arc<AnomalyWatcher>>,
        mut initial_symbols: Vec<String>,
        interval_secs: f64,
    ) -> Self {
        initial_symbols = initial_symbols
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        let feed_source = feed.source_name().to_string();
        let symbols = Arc::new(Mutex::new(initial_symbols.clone()));
        let symbols_task = symbols.clone();
        let interval = interval_secs.max(1.0);
        let task_slot = Arc::new(StdMutex::new(None));
        let task_slot_c = task_slot.clone();
        let handle = tokio::spawn(async move {
            let mut agg = KlineAggregator::new();
            let dur = tokio::time::Duration::from_secs_f64(interval);
            loop {
                let syms = symbols_task.lock().await.clone();
                if syms.is_empty() {
                    tokio::time::sleep(dur).await;
                    continue;
                }
                for sym in syms {
                    let ticks_enabled = state.config().ticks_enabled;
                    match feed.fetch_latest_tick(&sym).await {
                        Ok(Some(tick)) => {
                            emit_tick_update(&app, &tick);
                            emit_kline_updates(&app, &mut agg, &tick);
                            if ticks_enabled {
                                let _ = state.db.save_tick(&tick);
                            }
                            if let Some(w) = &anomaly {
                                w.on_tick(&state, &app, &tick);
                            }
                        }
                        Ok(None) => log::debug!("no tick for {sym}"),
                        Err(e) => log::debug!("tick fetch failed {sym}: {e}"),
                    }
                }
                tokio::time::sleep(dur).await;
            }
        });
        *task_slot_c.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);
        log::info!(
            "MarketPoll started ({feed_source}): {} symbols, interval={interval}s",
            initial_symbols.len()
        );
        Self {
            symbols,
            interval,
            feed_source,
            task: task_slot,
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

    pub async fn unsubscribe(&self, remove_symbols: Vec<String>) {
        let mut syms = self.symbols.lock().await;
        for s in remove_symbols {
            let lower = s.to_lowercase();
            syms.retain(|x| x != &lower);
        }
    }

    pub async fn status(&self) -> PollStatusSnapshot {
        let syms = self.symbols.lock().await.clone();
        PollStatusSnapshot {
            running: true,
            interval: self.interval,
            symbols: syms.clone(),
            symbol_count: syms.len(),
            feed_source: self.feed_source.clone(),
        }
    }
}

fn emit_tick_update(app: &AppHandle, tick: &Tick) {
    let _ = app.emit(
        "tick-update",
        TickUpdateEvent {
            symbol: tick.symbol.clone(),
            last_price: tick.last_price,
            volume: tick.volume,
            timestamp: tick.timestamp.clone(),
        },
    );
}

fn emit_kline_updates(app: &AppHandle, agg: &mut KlineAggregator, tick: &Tick) {
    for ev in agg.on_tick(tick) {
        let _ = app.emit("kline-update", &ev);
    }
}
