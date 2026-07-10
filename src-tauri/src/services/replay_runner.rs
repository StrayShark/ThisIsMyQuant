use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use tauri::{AppHandle, Emitter};

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::models::{KLine, ReplaySession, ReplayState};
use crate::services::SimTradingService;

const DEFAULT_INTERVAL: &str = "1m";

/// 历史行情回放运行器。
/// 加载某品种某日的 1 分钟 K 线，按 bar 推进，并复用 SimTradingService 的
/// `on_price_update` 触发限价/止损/条件单撮合、持仓盯市与风控强平。
pub struct ReplayRunner {
    bars: Vec<KLine>,
    current_index: usize,
    running: bool,
    speed: u32,
    symbol: String,
    date: String,
    interval: String,
    account_id: String,
    app_handle: Option<AppHandle>,
}

impl ReplayRunner {
    pub fn load(
        db: &Database,
        symbol: &str,
        date: &str,
        interval: Option<&str>,
        account_id: Option<&str>,
        speed: Option<i32>,
        app_handle: Option<AppHandle>,
    ) -> AppResult<Self> {
        let interval = interval.unwrap_or(DEFAULT_INTERVAL);
        let naive_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| AppError::Msg(format!("日期格式错误: {e}")))?;
        let start = naive_date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::Msg("构建起始时间失败".into()))?
            .and_utc();
        let end = naive_date
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| AppError::Msg("构建结束时间失败".into()))?
            .and_utc();

        let mut bars = db.get_klines(symbol, interval, start, end, 5000)?;
        if bars.is_empty() {
            return Err(AppError::Msg(format!(
                "未找到 {symbol} 在 {date} 的 {interval} K 线数据"
            )));
        }
        bars.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        // 尝试恢复同品种同日会话的进度
        let mut current_index = 0usize;
        if let Ok(Some(session)) = db.load_replay_session() {
            if session.symbol.eq_ignore_ascii_case(symbol)
                && session.replay_date == date
                && session.interval == interval
            {
                current_index = session.current_index.max(0) as usize;
                if current_index >= bars.len() {
                    current_index = bars.len().saturating_sub(1);
                }
            }
        }

        let speed = speed.unwrap_or(1).clamp(1, 20) as u32;
        let account_id = account_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default".into());

        Ok(Self {
            bars,
            current_index,
            running: false,
            speed,
            symbol: symbol.to_uppercase(),
            date: date.into(),
            interval: interval.into(),
            account_id,
            app_handle,
        })
    }

    /// 推进若干根 bar，返回最新状态。
    pub fn step(
        &mut self,
        steps: usize,
        sim_service: &SimTradingService,
    ) -> AppResult<ReplayState> {
        let steps = steps.max(1);
        let max_index = self.bars.len().saturating_sub(1);
        let target = (self.current_index + steps).min(max_index);

        for idx in self.current_index..=target {
            let bar = &self.bars[idx];
            sim_service.seed_price(&self.symbol, bar.close);
            let _ = sim_service.on_price_update(&self.symbol, bar.close);
        }

        self.current_index = target;
        if self.current_index >= max_index {
            self.running = false;
        }

        self.emit_update(sim_service.db())?;
        Ok(self.state())
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed.clamp(1, 20);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn state(&self) -> ReplayState {
        let bar = self.bars.get(self.current_index);
        let completed = self.current_index >= self.bars.len().saturating_sub(1);
        ReplayState {
            running: self.running && !completed,
            symbol: self.symbol.clone(),
            date: self.date.clone(),
            interval: self.interval.clone(),
            account_id: Some(self.account_id.clone()),
            current_index: self.current_index as i32,
            total_bars: self.bars.len() as i32,
            current_bar_time: bar.map(|b| b.start_time.clone()),
            current_price: bar.map(|b| b.close).unwrap_or(0.0),
            speed: self.speed as i32,
            completed,
        }
    }

    pub fn visible_bars(&self) -> Vec<KLine> {
        self.bars
            .iter()
            .take(self.current_index + 1)
            .cloned()
            .collect()
    }

    pub fn save_session(&self, db: &Database) -> AppResult<()> {
        let session = ReplaySession {
            id: "default".into(),
            symbol: self.symbol.clone(),
            interval: self.interval.clone(),
            replay_date: self.date.clone(),
            current_index: self.current_index as i32,
            speed: self.speed as i32,
            running: self.running,
            account_id: Some(self.account_id.clone()),
            updated_at: Utc::now().to_rfc3339(),
        };
        db.save_replay_session(&session)
    }

    fn emit_update(&self, db: &Database) -> AppResult<()> {
        let _ = self.save_session(db);
        if let Some(handle) = &self.app_handle {
            let _ = crate::services::emit_sim_update(handle, &self.account_id);
        }
        Ok(())
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn interval(&self) -> &str {
        &self.interval
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn bars(&self) -> &[KLine] {
        &self.bars
    }
}

/// 启动后台回放循环。调用方需持有 AppState 的 Arc。
pub fn spawn_replay_loop(
    app: AppHandle,
    state: Arc<crate::state::AppState>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let should_step = {
                let guard = state
                    .replay_runner
                    .read()
                    .unwrap_or_else(|e| e.into_inner());
                guard.as_ref().map(|r| r.is_running()).unwrap_or(false)
            };

            if !should_step {
                continue;
            }

            let speed = {
                let guard = state
                    .replay_runner
                    .read()
                    .unwrap_or_else(|e| e.into_inner());
                guard.as_ref().map(|r| r.speed).unwrap_or(1)
            };

            // 倍速越大，单次睡眠越短；speed=1 时约 1 秒一根 bar
            let sleep_ms = (1000u64 / speed as u64).max(50);
            tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;

            let completed = {
                let mut guard = state
                    .replay_runner
                    .write()
                    .unwrap_or_else(|e| e.into_inner());
                if let Some(runner) = guard.as_mut() {
                    if runner.is_running() {
                        let _ = runner.step(1, &state.sim_trading);
                        runner.state().completed
                    } else {
                        true
                    }
                } else {
                    true
                }
            };

            if completed {
                break;
            }
        }
        let _ = app.emit("replay-completed", ());
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::services::QuoteCache;
    use std::sync::{Arc, RwLock};
    use tempfile::TempDir;

    fn test_db() -> (Database, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = Database::open(&dir.path().join("test.db")).unwrap();
        db.init_schema().unwrap();
        (db, dir)
    }

    fn seed_bars(db: &Database, symbol: &str, date: &str) {
        let base = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let mut klines = Vec::new();
        for i in 0..10 {
            let t = base.and_hms_opt(9, i as u32, 0).unwrap().and_utc();
            let price = 3000.0 + i as f64 * 10.0;
            klines.push(KLine {
                symbol: symbol.into(),
                interval: "1m".into(),
                open: price,
                high: price + 5.0,
                low: price - 5.0,
                close: price,
                volume: 100,
                turnover: 0.0,
                start_time: crate::models::dt_to_iso(t),
            });
        }
        db.save_klines(&klines).unwrap();
    }

    #[test]
    fn test_load_and_step() {
        let (db, _dir) = test_db();
        seed_bars(&db, "RB0", "2024-01-15");
        let quote_cache = Arc::new(RwLock::new(QuoteCache::new()));
        let sim = Arc::new(SimTradingService::new(Arc::new(db), quote_cache));
        let db2 = sim.db();
        let mut runner =
            ReplayRunner::load(db2, "RB0", "2024-01-15", None, None, None, None).unwrap();
        assert_eq!(runner.bars.len(), 10);
        assert_eq!(runner.current_index(), 0);

        let state = runner.step(1, &sim).unwrap();
        assert_eq!(state.current_index, 1);
        assert_eq!(runner.visible_bars().len(), 2);
    }

    #[test]
    fn test_limit_order_fills_on_step() {
        let (db, _dir) = test_db();
        seed_bars(&db, "RB0", "2024-01-15");
        let quote_cache = Arc::new(RwLock::new(QuoteCache::new()));
        let sim = Arc::new(SimTradingService::new(Arc::new(db), quote_cache));
        let _ = sim.init_defaults();

        // 默认账户在 init_defaults 中创建
        let account = sim.default_account().unwrap();

        // 先设置一个高于限价的价格，让限价单挂起
        sim.seed_price("RB0", 3050.0);
        let req = crate::models::PlaceSimOrderRequest {
            account_id: account.id.clone(),
            symbol: "RB0".into(),
            side: "buy".into(),
            offset: "open".into(),
            order_type: "limit".into(),
            price: Some(3010.0),
            trigger_price: None,
            stop_loss_price: None,
            take_profit_price: None,
            oco_group_id: None,
            parent_order_id: None,
            tif: Some("GTC".into()),
            condition_operator: None,
            trailing_distance_ticks: None,
            quantity: 1,
        };
        let order = sim.place_order(req).unwrap();
        assert_eq!(order.status, "open");

        let db2 = sim.db();
        let mut runner =
            ReplayRunner::load(db2, "RB0", "2024-01-15", None, None, None, None).unwrap();
        // 第一根 bar close=3000 <= 3010，应成交
        let _ = runner.step(1, &sim);

        let orders = sim.list_orders(Some(&account.id), None, 100).unwrap();
        assert!(orders
            .iter()
            .any(|o| o.id == order.id && o.status == "filled"));
    }
}
