use std::sync::Arc;

use chrono::Utc;

use crate::db::Database;
use crate::engine::stock_paper::StockPaperEngine;
use crate::error::{AppError, AppResult};
use crate::models::{
    dt_to_iso, CancelStockPaperOrderRequest, CreateStockPaperAccountRequest,
    PlaceStockPaperOrderRequest, StockPaperAccount, StockPaperOrder, StockPaperPortfolioView,
};

pub struct StockPaperTradingService {
    db: Arc<Database>,
}

impl StockPaperTradingService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn create_account(
        &self,
        req: &CreateStockPaperAccountRequest,
    ) -> AppResult<StockPaperAccount> {
        let now = dt_to_iso(Utc::now());
        let account = StockPaperAccount {
            id: uuid::Uuid::new_v4().to_string(),
            name: req.name.clone(),
            initial_balance: req.initial_balance,
            cash_balance: req.initial_balance,
            market_value: 0.0,
            total_equity: req.initial_balance,
            total_cost: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };
        self.db.save_stock_paper_account(&account)?;
        Ok(account)
    }

    pub fn list_accounts(&self) -> AppResult<Vec<StockPaperAccount>> {
        self.db.list_stock_paper_accounts()
    }

    pub fn get_account(&self, id: &str) -> AppResult<Option<StockPaperAccount>> {
        self.db.get_stock_paper_account(id)
    }

    pub fn get_portfolio(&self, account_id: &str) -> AppResult<StockPaperPortfolioView> {
        let account = self
            .db
            .get_stock_paper_account(account_id)?
            .ok_or_else(|| AppError::Msg("account not found".to_string()))?;
        let positions = self.db.list_stock_paper_positions(account_id)?;
        let orders = self.db.list_stock_paper_orders(account_id)?;
        let trades = self.db.list_stock_paper_trades(account_id)?;
        Ok(StockPaperPortfolioView {
            account,
            positions,
            orders,
            trades,
        })
    }

    pub fn place_order(&self, req: &PlaceStockPaperOrderRequest) -> AppResult<StockPaperOrder> {
        let symbol = self
            .db
            .get_stock_symbol(&req.ts_code)?
            .ok_or_else(|| AppError::Msg("symbol not found".to_string()))?;
        let account = self
            .db
            .get_stock_paper_account(&req.account_id)?
            .ok_or_else(|| AppError::Msg("account not found".to_string()))?;
        let position = self
            .db
            .get_stock_paper_position(&req.account_id, &req.ts_code)?;
        let bars = self.db.get_stock_daily_bars(&req.ts_code, "none", 1)?;
        let bar = bars
            .last()
            .ok_or_else(|| AppError::Msg("no price data".to_string()))?;

        let now = dt_to_iso(Utc::now());
        let mut order = StockPaperOrder {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: req.account_id.clone(),
            ts_code: req.ts_code.clone(),
            name: symbol.name.clone(),
            side: req.side.clone(),
            order_type: req.order_type.clone(),
            price: req.price,
            quantity: req.quantity,
            filled_quantity: 0,
            status: "open".to_string(),
            reason: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        StockPaperEngine::validate_order(&account, position.as_ref(), bar, &order)?;

        let mut account = account;
        let mut position = position;
        let trade = StockPaperEngine::fill_order(&mut account, &mut position, &mut order, bar)?;

        // 更新账户市值与权益
        self.recalc_account(&mut account)?;

        self.db.save_stock_paper_account(&account)?;
        self.db.save_stock_paper_order(&order)?;
        self.db.save_stock_paper_trade(&trade)?;
        if let Some(pos) = position.as_ref() {
            self.db.save_stock_paper_position(pos)?;
        }

        Ok(order)
    }

    pub fn cancel_order(&self, req: &CancelStockPaperOrderRequest) -> AppResult<StockPaperOrder> {
        let orders = self.db.list_stock_paper_orders(&req.account_id)?;
        let mut order = orders
            .into_iter()
            .find(|o| o.id == req.order_id)
            .ok_or_else(|| AppError::Msg("order not found".to_string()))?;
        if order.status != "open" {
            return Err(AppError::Msg(
                "only open orders can be cancelled".to_string(),
            ));
        }
        order.status = "cancelled".to_string();
        order.updated_at = dt_to_iso(Utc::now());
        self.db.save_stock_paper_order(&order)?;
        Ok(order)
    }

    pub fn mark_end_of_day(&self, account_id: &str) -> AppResult<()> {
        let mut positions = self.db.list_stock_paper_positions(account_id)?;
        StockPaperEngine::mark_end_of_day(&mut positions);
        for pos in positions {
            self.db.save_stock_paper_position(&pos)?;
        }
        Ok(())
    }

    fn recalc_account(&self, account: &mut StockPaperAccount) -> AppResult<()> {
        let positions = self.db.list_stock_paper_positions(&account.id)?;
        account.market_value = positions.iter().map(|p| p.market_value).sum();
        account.unrealized_pnl = positions.iter().map(|p| p.unrealized_pnl).sum();
        account.total_equity = account.cash_balance + account.market_value;
        account.updated_at = dt_to_iso(Utc::now());
        Ok(())
    }
}

/// 每个交易日收盘后自动将 A 股模拟组合的可用持仓置为与持仓相等，实现 T+1。
pub fn spawn_stock_paper_eod(state: std::sync::Arc<crate::state::AppState>) {
    use chrono::{Datelike, Local, Timelike};
    use tokio::time::{sleep, Duration};

    tokio::spawn(async move {
        let mut last_run_ordinal: Option<u32> = None;
        loop {
            sleep(Duration::from_secs(60)).await;
            let now = Local::now();
            // A 股收盘时间 15:05（预留同步延迟）
            if now.hour() != 15 || now.minute() < 5 {
                continue;
            }
            let ordinal = now.ordinal();
            if last_run_ordinal == Some(ordinal) {
                continue;
            }
            last_run_ordinal = Some(ordinal);
            let accounts = match state.db.list_stock_paper_accounts() {
                Ok(a) => a,
                Err(e) => {
                    log::warn!("stock paper eod: list accounts failed: {e}");
                    continue;
                }
            };
            for acc in &accounts {
                if acc.status != "active" {
                    continue;
                }
                if let Err(e) = state.stock_paper.mark_end_of_day(&acc.id) {
                    log::warn!("stock paper eod {}: {e}", acc.id);
                }
            }
            log::info!(
                "stock paper eod: marked available positions for {} accounts",
                accounts.len()
            );
        }
    });
}
