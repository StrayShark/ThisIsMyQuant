use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::models::{
    dt_to_iso, StockBar, StockPaperAccount, StockPaperOrder, StockPaperPosition, StockPaperTrade,
};

/// A 股模拟组合撮合引擎
/// 规则：
/// - 买入/卖出单位为 100 股（1 手）
/// - T+1：当日买入的股票当日不可用，次日才可卖出
/// - 涨跌停限制：限价单价格不得超过昨日收盘价的 ±10%（ST 为 ±5%，这里简化为 ±10%）
/// - 费用：佣金（默认万 3，最低 5 元）、印花税（卖出千 1）、过户费（十万分之一，双向）

const LOT_SIZE: i64 = 100;
const PRICE_LIMIT_PCT: f64 = 0.10;
const COMMISSION_RATE: f64 = 0.0003;
const MIN_COMMISSION: f64 = 5.0;
const STAMP_TAX_RATE: f64 = 0.001;
const TRANSFER_FEE_RATE: f64 = 0.00001;

pub struct StockPaperEngine;

impl StockPaperEngine {
    pub fn validate_order(
        account: &StockPaperAccount,
        position: Option<&StockPaperPosition>,
        bar: &StockBar,
        order: &StockPaperOrder,
    ) -> AppResult<()> {
        if order.quantity <= 0 || order.quantity % LOT_SIZE != 0 {
            return Err(AppError::Msg(format!(
                "A 股委托数量必须为 {LOT_SIZE} 的整数倍"
            )));
        }

        let prev_close = bar.pre_close.unwrap_or(bar.close.unwrap_or(0.0));
        if prev_close > 0.0 {
            let price = order.price.unwrap_or(bar.close.unwrap_or(prev_close));
            let limit_up = prev_close * (1.0 + PRICE_LIMIT_PCT);
            let limit_down = prev_close * (1.0 - PRICE_LIMIT_PCT);
            if price > limit_up + 1e-6 {
                return Err(AppError::Msg(format!(
                    "买入价格不能超过涨停价 {:.2}",
                    limit_up
                )));
            }
            if price < limit_down - 1e-6 {
                return Err(AppError::Msg(format!(
                    "卖出价格不能低于跌停价 {:.2}",
                    limit_down
                )));
            }
        }

        match order.side.as_str() {
            "buy" => {
                let price = order.price.unwrap_or(bar.close.unwrap_or(0.0));
                let estimated = Self::estimate_buy_cost(price, order.quantity);
                if account.cash_balance < estimated.1 {
                    return Err(AppError::Msg(format!(
                        "可用资金不足，需要 {:.2}，当前 {:.2}",
                        estimated.1, account.cash_balance
                    )));
                }
            }
            "sell" => {
                let pos = position.ok_or_else(|| AppError::Msg("没有可卖出持仓".to_string()))?;
                if pos.available_quantity < order.quantity {
                    return Err(AppError::Msg(format!(
                        "可卖数量不足，当前可卖 {}，委托 {}",
                        pos.available_quantity, order.quantity
                    )));
                }
            }
            _ => return Err(AppError::Msg(format!("未知委托方向: {}", order.side))),
        }

        Ok(())
    }

    pub fn estimate_buy_cost(price: f64, quantity: i64) -> (f64, f64) {
        let amount = price * quantity as f64;
        let commission = (amount * COMMISSION_RATE).max(MIN_COMMISSION);
        let transfer_fee = amount * TRANSFER_FEE_RATE;
        let total = amount + commission + transfer_fee;
        (amount, total)
    }

    pub fn estimate_sell_proceeds(price: f64, quantity: i64) -> (f64, f64) {
        let amount = price * quantity as f64;
        let commission = (amount * COMMISSION_RATE).max(MIN_COMMISSION);
        let stamp_tax = amount * STAMP_TAX_RATE;
        let transfer_fee = amount * TRANSFER_FEE_RATE;
        let net = amount - commission - stamp_tax - transfer_fee;
        (amount, net)
    }

    /// 立即撮合（简化：按当前 bar 收盘价成交）
    pub fn fill_order(
        account: &mut StockPaperAccount,
        position: &mut Option<StockPaperPosition>,
        order: &mut StockPaperOrder,
        bar: &StockBar,
    ) -> AppResult<StockPaperTrade> {
        let fill_price = order.price.or(bar.close).unwrap_or(0.0);
        if fill_price <= 0.0 {
            return Err(AppError::Msg("无效成交价格".to_string()));
        }

        order.filled_quantity = order.quantity;
        order.status = "filled".to_string();
        order.updated_at = dt_to_iso(Utc::now());

        let trade_id = uuid::Uuid::new_v4().to_string();
        let now = dt_to_iso(Utc::now());

        match order.side.as_str() {
            "buy" => {
                let (amount, total_cost) = Self::estimate_buy_cost(fill_price, order.quantity);
                let commission = total_cost - amount;
                account.cash_balance -= total_cost;

                let pos = position.get_or_insert_with(|| StockPaperPosition {
                    account_id: account.id.clone(),
                    ts_code: order.ts_code.clone(),
                    name: order.name.clone(),
                    quantity: 0,
                    available_quantity: 0,
                    avg_cost: 0.0,
                    total_cost: 0.0,
                    market_value: 0.0,
                    unrealized_pnl: 0.0,
                    updated_at: now.clone(),
                });
                let new_total_cost = pos.total_cost + amount;
                pos.quantity += order.quantity;
                pos.total_cost = new_total_cost;
                pos.avg_cost = if pos.quantity > 0 {
                    new_total_cost / pos.quantity as f64
                } else {
                    0.0
                };
                pos.updated_at = now.clone();

                account.total_cost += amount;

                Ok(StockPaperTrade {
                    id: trade_id,
                    order_id: order.id.clone(),
                    account_id: account.id.clone(),
                    ts_code: order.ts_code.clone(),
                    name: order.name.clone(),
                    side: "buy".to_string(),
                    price: fill_price,
                    quantity: order.quantity,
                    commission,
                    traded_at: now,
                })
            }
            "sell" => {
                let pos = position
                    .as_mut()
                    .ok_or_else(|| AppError::Msg("没有持仓".to_string()))?;
                let (amount, net_proceeds) =
                    Self::estimate_sell_proceeds(fill_price, order.quantity);
                let commission = amount - net_proceeds;
                account.cash_balance += net_proceeds;

                let realized = (fill_price - pos.avg_cost) * order.quantity as f64 - commission;
                account.realized_pnl += realized;

                pos.quantity -= order.quantity;
                pos.available_quantity -= order.quantity;
                if pos.quantity > 0 {
                    pos.total_cost = pos.avg_cost * pos.quantity as f64;
                } else {
                    pos.total_cost = 0.0;
                    pos.avg_cost = 0.0;
                }
                pos.updated_at = now.clone();

                account.total_cost -= amount;

                Ok(StockPaperTrade {
                    id: trade_id,
                    order_id: order.id.clone(),
                    account_id: account.id.clone(),
                    ts_code: order.ts_code.clone(),
                    name: order.name.clone(),
                    side: "sell".to_string(),
                    price: fill_price,
                    quantity: order.quantity,
                    commission,
                    traded_at: now,
                })
            }
            _ => Err(AppError::Msg(format!("未知委托方向: {}", order.side))),
        }
    }

    pub fn mark_end_of_day(positions: &mut [StockPaperPosition]) {
        let today = Utc::now().date_naive().to_string();
        for pos in positions.iter_mut() {
            let pos_date = &pos.updated_at[..10.min(pos.updated_at.len())];
            if pos_date < today.as_str() {
                pos.available_quantity = pos.quantity;
                pos.updated_at = dt_to_iso(Utc::now());
            }
        }
    }

    pub fn update_position_market_value(
        position: &mut StockPaperPosition,
        price: f64,
    ) -> AppResult<()> {
        position.market_value = price * position.quantity as f64;
        position.unrealized_pnl = (price - position.avg_cost) * position.quantity as f64;
        position.updated_at = dt_to_iso(Utc::now());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn sample_account() -> StockPaperAccount {
        StockPaperAccount {
            id: "acc-1".to_string(),
            name: "测试账户".to_string(),
            initial_balance: 100000.0,
            cash_balance: 100000.0,
            market_value: 0.0,
            total_equity: 100000.0,
            total_cost: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            status: "active".to_string(),
            created_at: dt_to_iso(Utc::now()),
            updated_at: dt_to_iso(Utc::now()),
        }
    }

    fn sample_bar() -> StockBar {
        StockBar {
            ts_code: "600000.SH".to_string(),
            trade_date: "20260101".to_string(),
            open: Some(10.0),
            high: Some(10.5),
            low: Some(9.8),
            close: Some(10.2),
            pre_close: Some(10.0),
            pct_chg: Some(2.0),
            volume: Some(1e6),
            amount: Some(1e8),
            turnover_rate: Some(0.01),
            adj_factor: None,
            adjustment: "none".to_string(),
            source: "test".to_string(),
            updated_at: dt_to_iso(Utc::now()),
        }
    }

    #[test]
    fn buy_order_lot_size_validation() {
        let account = sample_account();
        let bar = sample_bar();
        let order = StockPaperOrder {
            id: "o1".to_string(),
            account_id: "acc-1".to_string(),
            ts_code: "600000.SH".to_string(),
            name: "浦发银行".to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            price: Some(10.0),
            quantity: 150,
            filled_quantity: 0,
            status: "open".to_string(),
            reason: None,
            created_at: dt_to_iso(Utc::now()),
            updated_at: dt_to_iso(Utc::now()),
        };
        assert!(StockPaperEngine::validate_order(&account, None, &bar, &order).is_err());
    }

    #[test]
    fn buy_and_sell_t1() {
        let mut account = sample_account();
        let bar = sample_bar();
        let mut order = StockPaperOrder {
            id: "o1".to_string(),
            account_id: "acc-1".to_string(),
            ts_code: "600000.SH".to_string(),
            name: "浦发银行".to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            price: Some(10.0),
            quantity: 100,
            filled_quantity: 0,
            status: "open".to_string(),
            reason: None,
            created_at: dt_to_iso(Utc::now()),
            updated_at: dt_to_iso(Utc::now()),
        };
        let mut position: Option<StockPaperPosition> = None;
        let trade =
            StockPaperEngine::fill_order(&mut account, &mut position, &mut order, &bar).unwrap();
        assert_eq!(trade.quantity, 100);
        assert_eq!(order.status, "filled");
        assert!(position.as_ref().unwrap().available_quantity == 0); // T+1

        // 当日不可卖出
        let sell_order = StockPaperOrder {
            id: "o2".to_string(),
            account_id: "acc-1".to_string(),
            ts_code: "600000.SH".to_string(),
            name: "浦发银行".to_string(),
            side: "sell".to_string(),
            order_type: "limit".to_string(),
            price: Some(11.0),
            quantity: 100,
            filled_quantity: 0,
            status: "open".to_string(),
            reason: None,
            created_at: dt_to_iso(Utc::now()),
            updated_at: dt_to_iso(Utc::now()),
        };
        assert!(
            StockPaperEngine::validate_order(&account, position.as_ref(), &bar, &sell_order)
                .is_err()
        );

        // 次日可卖出
        position.as_mut().unwrap().updated_at = dt_to_iso(Utc::now() - Duration::days(1));
        StockPaperEngine::mark_end_of_day(std::slice::from_mut(position.as_mut().unwrap()));
        assert!(
            StockPaperEngine::validate_order(&account, position.as_ref(), &bar, &sell_order)
                .is_ok()
        );
    }
}
