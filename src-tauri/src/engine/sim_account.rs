use crate::error::{AppError, AppResult};
use crate::models::{SimAccount, SimPosition};

pub fn initial_account(name: impl Into<String>, initial_balance: f64) -> SimAccount {
    let now = chrono::Utc::now().to_rfc3339();
    SimAccount {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.into(),
        currency: "CNY".into(),
        initial_balance,
        cash_balance: initial_balance,
        equity: initial_balance,
        margin_used: 0.0,
        realized_pnl: 0.0,
        unrealized_pnl: 0.0,
        status: "active".into(),
        created_at: now.clone(),
        updated_at: now,
    }
}

pub fn recalc_account(
    account: &mut SimAccount,
    positions: &[SimPosition],
    realized_delta: f64,
) -> AppResult<()> {
    account.margin_used = positions.iter().map(|p| p.margin).sum();
    account.unrealized_pnl = positions.iter().map(|p| p.unrealized_pnl).sum();
    account.realized_pnl += realized_delta;
    account.equity = account.cash_balance + account.margin_used + account.unrealized_pnl;
    if account.equity <= 0.0 {
        return Err(AppError::Msg("账户权益不足，已触发模拟强平".into()));
    }
    account.updated_at = chrono::Utc::now().to_rfc3339();
    Ok(())
}

pub fn risk_ratio(account: &SimAccount) -> f64 {
    if account.equity <= 0.0 {
        1.0
    } else {
        account.margin_used / account.equity
    }
}

pub fn today_pnl(account: &SimAccount, snapshot_at_start: Option<f64>) -> f64 {
    let start = snapshot_at_start.unwrap_or(account.initial_balance);
    account.equity - start
}
