use crate::engine::sim_contract::{commission_for_trade, compute_margin, round_to_tick};
use crate::error::{AppError, AppResult};
use crate::models::{RealtimeQuote, SimContractRule, SimOrder, SimPosition, SimTrade};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub trade: SimTrade,
    pub realized_pnl: f64,
    pub position_delta: PositionDelta,
}

#[derive(Debug, Clone, Default)]
pub struct PositionDelta {
    pub symbol: String,
    pub name: String,
    pub position_side: String,
    pub qty: i64,
    pub avg_price: f64,
    pub margin: f64,
}

/// 判断订单是否可在当前行情下成交。
/// 市价单、已触发的止损/止盈/移动止损单（已被转为市价/限价）可直接成交。
/// 限价单：买限价在最新价 <= 限价或卖一 <= 限价时成交；卖限价反之。
pub fn order_executable(order: &SimOrder, quote: &RealtimeQuote) -> bool {
    if order.quantity <= order.filled_quantity {
        return false;
    }
    match order.order_type.as_str() {
        "market" => true,
        "limit" | "stop_limit" | "take_profit_limit" => {
            let limit = order.price.unwrap_or(quote.last_price);
            if order.side == "buy" {
                quote.last_price <= limit || quote.ask_price <= limit
            } else {
                quote.last_price >= limit || quote.bid_price >= limit
            }
        }
        // 原始止损/止盈/条件/移动止损单由 process_stop_and_condition_orders 触发后转类型再撮合。
        "stop" | "stop_market" | "take_profit" | "trailing_stop" | "condition" => false,
        _ => false,
    }
}

/// 从行情报价读取对手盘可用量。买入看卖一量，卖出看买一量。
pub fn quote_liquidity_for(order: &SimOrder, quote: &RealtimeQuote) -> i64 {
    if order.side == "buy" {
        quote.ask_volume
    } else {
        quote.bid_volume
    }
}

/// 计算成交价。
/// 市价/止损/止盈/移动止损按三价取中规则取对交易者最不利价；
/// 限价单按「max(限价, 卖一, 最新价) / min(限价, 买一, 最新价)」成交。
fn fill_price(order: &SimOrder, quote: &RealtimeQuote, slippage_ticks: f64, tick: f64) -> f64 {
    let base = match order.order_type.as_str() {
        "market" | "stop" | "stop_market" | "take_profit" | "trailing_stop" => {
            if order.side == "buy" {
                quote.ask_price.max(quote.last_price)
            } else {
                quote.bid_price.min(quote.last_price)
            }
        }
        "limit" | "stop_limit" | "take_profit_limit" => {
            let limit = order.price.unwrap_or(quote.last_price);
            if order.side == "buy" {
                limit.max(quote.ask_price).max(quote.last_price)
            } else {
                limit.min(quote.bid_price).min(quote.last_price)
            }
        }
        _ => order.price.unwrap_or(quote.last_price),
    };
    let slippage = slippage_ticks * tick;
    if order.side == "buy" {
        round_to_tick(base + slippage, tick)
    } else {
        round_to_tick(base - slippage, tick)
    }
}

/// 对单笔订单尝试撮合。available_liquidity 为本 tick 对手盘可用量；
/// 若盘口量不足则按部分成交返回，剩余委托保持 open/partially_filled。
pub fn match_order(
    order: &SimOrder,
    quote: &RealtimeQuote,
    rule: &SimContractRule,
    slippage_ticks: f64,
    existing_positions: &[SimPosition],
    available_liquidity: i64,
) -> AppResult<MatchResult> {
    if !order_executable(order, quote) {
        return Err(AppError::Msg("订单当前不可成交".into()));
    }
    if order.quantity <= order.filled_quantity {
        return Err(AppError::Msg("订单已全部成交".into()));
    }

    let remaining = order.quantity - order.filled_quantity;
    let max_by_liquidity = remaining.min(available_liquidity.max(0));
    let qty = if order.offset.starts_with("close") {
        let opposite = if order.side == "buy" { "short" } else { "long" };
        let available = existing_positions
            .iter()
            .find(|p| p.symbol == order.symbol && p.position_side == opposite)
            .map(|p| p.total_qty)
            .unwrap_or(0);
        max_by_liquidity.min(available)
    } else {
        max_by_liquidity
    };

    if qty <= 0 {
        return Err(AppError::Msg("无可成交量".into()));
    }

    let executed = fill_price(order, quote, slippage_ticks, rule.price_tick);
    let now = chrono::Utc::now().to_rfc3339();
    let commission = commission_for_trade(rule, executed, qty, &order.offset);

    let realized_pnl = if order.offset.starts_with("close") {
        calc_close_pnl(
            order,
            qty,
            executed,
            existing_positions,
            rule.contract_multiplier,
        )
    } else {
        0.0
    };

    let margin = if order.offset.starts_with("close") {
        0.0
    } else {
        compute_margin(rule, executed, qty, &order.side)
    };

    let base_slippage_price = fill_price(order, quote, 0.0, rule.price_tick);
    let trade = SimTrade {
        id: uuid::Uuid::new_v4().to_string(),
        order_id: order.id.clone(),
        account_id: order.account_id.clone(),
        symbol: order.symbol.clone(),
        name: order.name.clone(),
        side: order.side.clone(),
        offset: order.offset.clone(),
        price: executed,
        quantity: qty,
        commission,
        slippage: (executed - base_slippage_price).abs(),
        realized_pnl,
        traded_at: now.clone(),
    };

    let position_side = if order.side == "buy" { "long" } else { "short" };
    let delta = PositionDelta {
        symbol: order.symbol.clone(),
        name: order.name.clone(),
        position_side: position_side.into(),
        qty,
        avg_price: executed,
        margin,
    };

    Ok(MatchResult {
        trade,
        realized_pnl,
        position_delta: delta,
    })
}

fn calc_close_pnl(
    order: &SimOrder,
    qty: i64,
    price: f64,
    positions: &[SimPosition],
    contract_multiplier: f64,
) -> f64 {
    let opposite = if order.side == "buy" { "short" } else { "long" };
    let pos = positions
        .iter()
        .find(|p| p.symbol == order.symbol && p.position_side == opposite);
    if let Some(p) = pos {
        let close_qty = qty.min(p.total_qty);
        let pnl = if order.side == "buy" {
            (p.avg_price - price) * contract_multiplier * close_qty as f64
        } else {
            (price - p.avg_price) * contract_multiplier * close_qty as f64
        };
        return pnl;
    }
    0.0
}

pub fn update_position_after_trade(
    positions: &mut Vec<SimPosition>,
    order: &SimOrder,
    qty: i64,
    price: f64,
    rule: &SimContractRule,
) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let target_side = if order.side == "buy" { "long" } else { "short" };
    let margin_rate = if target_side == "long" {
        rule.margin_rate_long
    } else {
        rule.margin_rate_short
    };

    if order.offset.starts_with("close") {
        let opposite = if target_side == "long" {
            "short"
        } else {
            "long"
        };
        if let Some(idx) = positions
            .iter()
            .position(|p| p.symbol == order.symbol && p.position_side == opposite)
        {
            let pos = &mut positions[idx];
            let close_qty = qty.min(pos.total_qty);
            if order.offset == "close_today" {
                let from_today = close_qty.min(pos.today_qty);
                pos.today_qty -= from_today;
                pos.history_qty -= close_qty - from_today;
            } else {
                let from_history = close_qty.min(pos.history_qty);
                pos.history_qty -= from_history;
                pos.today_qty -= close_qty - from_history;
            }
            pos.total_qty -= close_qty;
            pos.margin =
                pos.total_qty as f64 * pos.avg_price * rule.contract_multiplier * margin_rate;
            pos.updated_at = now.clone();
            if pos.total_qty == 0 {
                positions.remove(idx);
            }
        }
    } else {
        if let Some(idx) = positions
            .iter()
            .position(|p| p.symbol == order.symbol && p.position_side == target_side)
        {
            let pos = &mut positions[idx];
            let total_value = pos.avg_price * pos.total_qty as f64 + price * qty as f64;
            pos.total_qty += qty;
            pos.today_qty += qty;
            pos.avg_price = total_value / pos.total_qty as f64;
            pos.margin =
                pos.total_qty as f64 * pos.avg_price * rule.contract_multiplier * margin_rate;
            pos.updated_at = now.clone();
        } else {
            positions.push(SimPosition {
                account_id: order.account_id.clone(),
                symbol: order.symbol.clone(),
                name: order.name.clone(),
                position_side: target_side.into(),
                today_qty: qty,
                history_qty: 0,
                total_qty: qty,
                avg_price: price,
                margin: qty as f64 * price * rule.contract_multiplier * margin_rate,
                unrealized_pnl: 0.0,
                updated_at: now,
            });
        }
    }
    Ok(())
}

pub fn mark_to_market(
    positions: &mut [SimPosition],
    symbol: &str,
    price: f64,
    rule: &SimContractRule,
) {
    for pos in positions.iter_mut() {
        if pos.symbol == symbol {
            pos.unrealized_pnl = if pos.position_side == "long" {
                (price - pos.avg_price) * rule.contract_multiplier * pos.total_qty as f64
            } else {
                (pos.avg_price - price) * rule.contract_multiplier * pos.total_qty as f64
            };
        }
    }
}

/// 处理 STOP / TAKE_PROFIT / TRAILING_STOP / CONDITION 单触发。
/// 返回被激活后应重新撮合的订单列表（已转为 market 或 limit）。
pub fn process_stop_and_condition_orders(
    orders: &mut [SimOrder],
    quote: &RealtimeQuote,
    tick: f64,
) -> Vec<SimOrder> {
    let mut activated = Vec::new();
    for order in orders.iter_mut() {
        if order.status != "open" || order.filled_quantity >= order.quantity {
            continue;
        }
        let trigger = match order.trigger_price {
            Some(p) => p,
            None => continue,
        };

        let triggered = match order.order_type.as_str() {
            "stop" | "stop_market" => {
                // 买入止损：价格上穿触发价；卖出止损：价格下穿触发价。
                if order.side == "buy" {
                    quote.last_price >= trigger
                } else {
                    quote.last_price <= trigger
                }
            }
            "stop_limit" => {
                // 触发条件与 stop 相同，触发后转为限价单。
                if order.side == "buy" {
                    quote.last_price >= trigger
                } else {
                    quote.last_price <= trigger
                }
            }
            "take_profit" | "take_profit_limit" => {
                // 止盈方向与止损相反：多头止盈（卖）在价格 >= 触发价时触发；空头止盈（买）反之。
                if order.side == "buy" {
                    quote.last_price <= trigger
                } else {
                    quote.last_price >= trigger
                }
            }
            "trailing_stop" => {
                // 初始化或更新参考价。
                let distance = order.trailing_distance_ticks.unwrap_or(0.0) * tick;
                if distance <= 0.0 {
                    false
                } else {
                    let mut reference = order.trailing_reference_price.unwrap_or(quote.last_price);
                    if order.side == "buy" {
                        // 空头移动止损（买入平仓）：参考价为最低价，触发价 = 参考价 + distance
                        if quote.last_price < reference {
                            reference = quote.last_price;
                            order.trailing_reference_price = Some(reference);
                            order.trigger_price = Some(round_to_tick(reference + distance, tick));
                        }
                        quote.last_price >= trigger
                    } else {
                        // 多头移动止损（卖出平仓）：参考价为最高价，触发价 = 参考价 - distance
                        if quote.last_price > reference {
                            reference = quote.last_price;
                            order.trailing_reference_price = Some(reference);
                            order.trigger_price = Some(round_to_tick(reference - distance, tick));
                        }
                        quote.last_price <= trigger
                    }
                }
            }
            "condition" => {
                let op = order.condition_operator.as_deref().unwrap_or(">=");
                if op == "<=" {
                    quote.last_price <= trigger
                } else {
                    quote.last_price >= trigger
                }
            }
            _ => false,
        };

        if triggered {
            let now = chrono::Utc::now().to_rfc3339();
            order.updated_at = now;
            let activated_order = match order.order_type.as_str() {
                "stop_limit" | "take_profit_limit" => {
                    order.order_type = "limit".into();
                    if order.price.is_none() {
                        order.price = order.trigger_price;
                    }
                    order.clone()
                }
                _ => {
                    order.order_type = "market".into();
                    order.clone()
                }
            };
            activated.push(activated_order);
        }
    }
    activated
}

/// 找出与给定订单同 OCO 组的另一张挂单并将其取消。
pub fn cancel_oco_pair(orders: &mut [SimOrder], filled_order: &SimOrder) {
    let group_id = match &filled_order.oco_group_id {
        Some(id) => id,
        None => return,
    };
    for order in orders.iter_mut() {
        if order.id == filled_order.id {
            continue;
        }
        let same_group = order
            .oco_group_id
            .as_ref()
            .map(|g| g == group_id)
            .unwrap_or(false);
        if (order.status == "open" || order.status == "partially_filled") && same_group {
            order.status = "cancelled".into();
            order.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> SimContractRule {
        SimContractRule {
            symbol: "RB0".into(),
            name: "螺纹钢".into(),
            exchange: "SHFE".into(),
            contract_multiplier: 10.0,
            price_tick: 1.0,
            margin_rate_long: 0.1,
            margin_rate_short: 0.1,
            commission_mode: "per_hand".into(),
            commission_open: 3.0,
            commission_close: 3.0,
            commission_close_today: 0.0,
            min_order_qty: 1,
            lot_size: 1,
            max_order_qty: 0,
            daily_price_limit_up: 0.0,
            daily_price_limit_down: 0.0,
            default_slippage_ticks: 0.0,
            is_custom: false,
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    fn quote(last: f64, bid: f64, ask: f64, bid_vol: i64, ask_vol: i64) -> RealtimeQuote {
        RealtimeQuote {
            symbol: "RB0".into(),
            last_price: last,
            bid_price: bid,
            ask_price: ask,
            bid_volume: bid_vol,
            ask_volume: ask_vol,
            prev_close: last,
            change_pct: 0.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            forming_daily: None,
        }
    }

    fn order(order_type: &str, side: &str, qty: i64, price: Option<f64>) -> SimOrder {
        SimOrder {
            id: "o1".into(),
            account_id: "a1".into(),
            symbol: "RB0".into(),
            name: "螺纹钢".into(),
            side: side.into(),
            offset: "open".into(),
            order_type: order_type.into(),
            price,
            trigger_price: None,
            stop_loss_price: None,
            take_profit_price: None,
            oco_group_id: None,
            parent_order_id: None,
            tif: None,
            condition_operator: None,
            trailing_distance_ticks: None,
            trailing_reference_price: None,
            quantity: qty,
            filled_quantity: 0,
            status: "open".into(),
            reason: None,
            source: "manual".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn limit_buy_fill_price_uses_worst_of_three_prices() {
        let o = order("limit", "buy", 1, Some(3500.0));
        let q = quote(3490.0, 3480.0, 3485.0, 10, 10);
        let r = match_order(&o, &q, &rule(), 0.0, &[], 100).unwrap();
        // max(limit=3500, ask=3485, last=3490) = 3500
        assert_eq!(r.trade.price, 3500.0);
    }

    #[test]
    fn limit_sell_fill_price_uses_worst_of_three_prices() {
        let o = order("limit", "sell", 1, Some(3510.0));
        let q = quote(3520.0, 3515.0, 3525.0, 10, 10);
        let r = match_order(&o, &q, &rule(), 0.0, &[], 100).unwrap();
        // min(limit=3510, bid=3515, last=3520) = 3510
        assert_eq!(r.trade.price, 3510.0);
    }

    #[test]
    fn partial_fill_when_liquidity_is_low() {
        let o = order("limit", "buy", 10, Some(3500.0));
        let q = quote(3490.0, 3480.0, 3485.0, 10, 3);
        let r = match_order(&o, &q, &rule(), 0.0, &[], 3).unwrap();
        assert_eq!(r.trade.quantity, 3);
    }

    #[test]
    fn stop_loss_sell_triggers_when_price_drops() {
        let mut o = order("stop", "sell", 1, None);
        o.trigger_price = Some(3500.0);
        let mut orders = vec![o];
        let q = quote(3495.0, 3494.0, 3496.0, 10, 10);
        let activated = process_stop_and_condition_orders(&mut orders, &q, 1.0);
        assert_eq!(activated.len(), 1);
        assert_eq!(orders[0].order_type, "market");
    }

    #[test]
    fn take_profit_sell_triggers_when_price_rises() {
        let mut o = order("take_profit", "sell", 1, None);
        o.trigger_price = Some(3550.0);
        let mut orders = vec![o];
        let q = quote(3550.0, 3549.0, 3551.0, 10, 10);
        let activated = process_stop_and_condition_orders(&mut orders, &q, 1.0);
        assert_eq!(activated.len(), 1);
        assert_eq!(orders[0].order_type, "market");
    }

    #[test]
    fn condition_below_triggers_when_price_drops() {
        let mut o = order("condition", "buy", 1, None);
        o.trigger_price = Some(3500.0);
        o.condition_operator = Some("<=".into());
        let mut orders = vec![o];
        let q = quote(3495.0, 3494.0, 3496.0, 10, 10);
        let activated = process_stop_and_condition_orders(&mut orders, &q, 1.0);
        assert_eq!(activated.len(), 1);
        assert_eq!(orders[0].order_type, "market");
    }

    #[test]
    fn trailing_stop_moves_reference_higher() {
        let mut o = order("trailing_stop", "sell", 1, None);
        o.trigger_price = Some(3495.0);
        o.trailing_distance_ticks = Some(5.0);
        o.trailing_reference_price = Some(3500.0);
        let mut orders = vec![o];

        // 价格创新高，参考价上移，触发价跟随上移。
        let q1 = quote(3505.0, 3504.0, 3506.0, 10, 10);
        process_stop_and_condition_orders(&mut orders, &q1, 1.0);
        assert_eq!(orders[0].trailing_reference_price, Some(3505.0));
        assert_eq!(orders[0].trigger_price, Some(3500.0));

        // 价格从高点回落到新的触发价，应触发。
        let q2 = quote(3500.0, 3499.0, 3501.0, 10, 10);
        let activated = process_stop_and_condition_orders(&mut orders, &q2, 1.0);
        assert_eq!(activated.len(), 1);
        assert_eq!(orders[0].order_type, "market");
    }

    #[test]
    fn oco_pair_cancelled_when_one_fills() {
        let mut a = order("stop", "sell", 1, None);
        a.id = "a1".into();
        a.oco_group_id = Some("g1".into());
        a.trigger_price = Some(3500.0);
        let mut b = order("take_profit", "sell", 1, None);
        b.id = "b1".into();
        b.oco_group_id = Some("g1".into());
        b.trigger_price = Some(3600.0);
        let mut orders = vec![a.clone(), b.clone()];

        // 模拟 a 已成交。
        a.status = "filled".into();
        cancel_oco_pair(&mut orders, &a);
        assert_eq!(orders[1].status, "cancelled");
    }
}
