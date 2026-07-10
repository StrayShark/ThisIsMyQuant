use std::sync::Arc;

use chrono::{Datelike, Local, Timelike, Utc};
use tokio::sync::Mutex;

use crate::adapters::{AkshareStockProvider, StockBarsRequest, StockDataProvider};
use crate::db::Database;
use crate::error::AppResult;
use crate::models::dt_to_iso;
use crate::state::AppState;
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
pub struct StockDataSyncService {
    db: Arc<Database>,
    provider: AkshareStockProvider,
    running: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl StockDataSyncService {
    pub fn new(db: Arc<Database>, provider: AkshareStockProvider) -> Self {
        Self {
            db,
            provider,
            running: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn sync_symbols(&self, task_id: &str) -> AppResult<usize> {
        self.running
            .lock()
            .await
            .insert(task_id.to_string(), "sync_symbols".to_string());
        let symbols = self.provider.list_symbols().await?;
        let count = symbols.len();
        let result = self.db.save_stock_symbols(&symbols);
        self.running.lock().await.remove(task_id);
        result?;
        log::info!("stock sync symbols: {}", count);
        Ok(count)
    }

    pub async fn sync_indices(&self, task_id: &str) -> AppResult<usize> {
        self.running
            .lock()
            .await
            .insert(task_id.to_string(), "sync_indices".to_string());
        let index_codes = vec!["000001.SH", "399001.SZ", "399006.SZ", "000688.SH"];
        let mut total = 0usize;
        for code in index_codes {
            let req = StockBarsRequest {
                code: code.to_string(),
                adjustment: "none".to_string(),
                start_date: None,
                end_date: None,
                limit: 30,
            };
            let bars = self.provider.list_index_bars(req).await?;
            total += bars.len();
            self.db.save_stock_index_daily_bars(&bars)?;
        }
        self.running.lock().await.remove(task_id);
        log::info!("stock sync indices: {}", total);
        Ok(total)
    }

    pub async fn sync_daily_bars(
        &self,
        task_id: &str,
        symbols: Option<Vec<String>>,
    ) -> AppResult<usize> {
        self.running
            .lock()
            .await
            .insert(task_id.to_string(), "sync_daily_bars".to_string());
        let ts_codes = match symbols {
            Some(list) => list,
            None => self
                .db
                .list_stock_symbols(None, None, 1000)?
                .into_iter()
                .map(|s| s.ts_code)
                .collect(),
        };
        let mut total = 0usize;
        for ts_code in ts_codes {
            let req = StockBarsRequest {
                code: ts_code.clone(),
                adjustment: "none".to_string(),
                start_date: None,
                end_date: None,
                limit: 250,
            };
            match self.provider.list_stock_bars(req).await {
                Ok(bars) => {
                    total += bars.len();
                    if let Err(e) = self.db.save_stock_daily_bars(&bars) {
                        log::warn!("save daily bars for {ts_code}: {e}");
                    }
                }
                Err(e) => {
                    log::warn!("fetch daily bars for {ts_code}: {e}");
                }
            }
        }
        self.running.lock().await.remove(task_id);
        log::info!("stock sync daily bars: {}", total);
        Ok(total)
    }

    pub async fn sync_boards(&self, task_id: &str) -> AppResult<usize> {
        self.running
            .lock()
            .await
            .insert(task_id.to_string(), "sync_boards".to_string());
        let boards = self.provider.list_boards().await?;
        let board_count = boards.len();
        self.db.save_stock_boards(&boards)?;

        let mut member_count = 0usize;
        for board in boards.iter().take(100) {
            match self.provider.list_board_members(&board.board_code).await {
                Ok(members) => {
                    member_count += members.len();
                    if let Err(e) = self.db.save_stock_board_members(&members) {
                        log::warn!("save board members {}: {e}", board.board_code);
                    }
                }
                Err(e) => {
                    log::warn!("fetch board members {}: {e}", board.board_code);
                }
            }
        }
        self.running.lock().await.remove(task_id);
        log::info!(
            "stock sync boards: {} boards, {} members",
            board_count,
            member_count
        );
        Ok(board_count + member_count)
    }

    pub async fn sync_market_snapshot(&self, task_id: &str) -> AppResult<usize> {
        self.running
            .lock()
            .await
            .insert(task_id.to_string(), "sync_market_snapshot".to_string());
        // 复用 indices 同步作为市场快照的一部分
        let index_codes = vec!["000001.SH", "399001.SZ", "399006.SZ", "000688.SH"];
        let now = dt_to_iso(Utc::now());
        let mut total = 0usize;
        for code in index_codes {
            let req = StockBarsRequest {
                code: code.to_string(),
                adjustment: "none".to_string(),
                start_date: None,
                end_date: None,
                limit: 1,
            };
            let bars = self.provider.list_index_bars(req).await?;
            total += bars.len();
            self.db.save_stock_index_daily_bars(&bars)?;
        }
        // 同步板块快照：先取本地板块，再用 provider 获取最新行情（P1 实现）
        let boards = self.db.list_stock_boards(Some("industry"))?;
        let mut snapshots = Vec::new();
        for board in boards.iter().take(100) {
            snapshots.push(crate::models::StockBoardSnapshot {
                board_code: board.board_code.clone(),
                trade_date: today_trade_date(),
                pct_chg: None,
                amount: None,
                turnover_rate: None,
                net_flow: None,
                up_count: None,
                down_count: None,
                source: "placeholder".to_string(),
                updated_at: now.clone(),
            });
        }
        self.db.save_stock_board_snapshots(&snapshots)?;
        self.running.lock().await.remove(task_id);
        log::info!("stock sync market snapshot: {}", total);
        Ok(total + snapshots.len())
    }

    pub fn is_running(&self, task_id: &str) -> bool {
        // 同步检查：非阻塞
        if let Ok(map) = self.running.try_lock() {
            return map.contains_key(task_id);
        }
        false
    }
}

fn today_trade_date() -> String {
    Utc::now().format("%Y%m%d").to_string()
}

/// 每日收盘后自动同步 A 股数据（指数、市场快照、自选股日 K）。
pub fn spawn_stock_data_sync(state: Arc<AppState>, app: AppHandle) {
    tokio::spawn(async move {
        let mut last_run_ordinal: Option<u32> = None;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let now = Local::now();
            // A 股收盘后 15:35 执行，预留数据更新延迟
            if now.hour() != 15 || now.minute() < 35 {
                continue;
            }
            let ordinal = now.ordinal();
            if last_run_ordinal == Some(ordinal) {
                continue;
            }
            last_run_ordinal = Some(ordinal);

            let task_id = uuid::Uuid::new_v4().to_string();
            let _ = app.emit(
                "stock-sync-progress",
                serde_json::json!({
                    "task_id": &task_id,
                    "scope": "daily",
                    "status": "running",
                    "message": "A 股收盘同步启动",
                }),
            );

            // 收集自选股池中的唯一代码
            let watchlist_symbols: Vec<String> = match state.db.list_stock_watchlists() {
                Ok(lists) => {
                    let mut set = std::collections::HashSet::new();
                    for list in lists {
                        for s in list.symbols {
                            set.insert(s);
                        }
                    }
                    set.into_iter().collect()
                }
                Err(e) => {
                    log::warn!("daily stock sync: list watchlists failed: {e}");
                    vec![]
                }
            };

            let sync = state.stock_sync.clone();
            let app2 = app.clone();
            let symbols = if watchlist_symbols.is_empty() {
                None
            } else {
                Some(watchlist_symbols)
            };
            tokio::spawn(async move {
                let mut total = 0usize;
                if let Ok(n) = sync.sync_indices(&task_id).await {
                    total += n;
                }
                if let Ok(n) = sync.sync_market_snapshot(&task_id).await {
                    total += n;
                }
                if let Ok(n) = sync.sync_daily_bars(&task_id, symbols).await {
                    total += n;
                }
                let _ = app2.emit(
                    "stock-sync-done",
                    serde_json::json!({
                        "task_id": &task_id,
                        "scope": "daily",
                        "status": "done",
                        "total": total,
                    }),
                );
                log::info!("daily stock sync done: {total} records");
            });
        }
    });
}
