use crate::error::{AppError, AppResult};
use crate::models::{Contract, SimContractRule};

pub fn default_rules() -> Vec<SimContractRule> {
    vec![
        rule(
            "RB0",
            "螺纹钢",
            "SHFE",
            10.0,
            1.0,
            0.1,
            0.1,
            3.0,
            3.0,
            0.0,
            1.0,
        ),
        rule(
            "AU0", "黄金", "SHFE", 1000.0, 0.02, 0.1, 0.1, 10.0, 10.0, 0.0, 1.0,
        ),
        rule(
            "SC0", "原油", "INE", 1000.0, 0.1, 0.12, 0.12, 20.0, 20.0, 0.0, 2.0,
        ),
        rule(
            "EC0",
            "集运欧线",
            "INE",
            50.0,
            0.1,
            0.2,
            0.2,
            6.0,
            6.0,
            0.0,
            2.0,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn rule(
    symbol: &str,
    name: &str,
    exchange: &str,
    multiplier: f64,
    tick: f64,
    margin_long: f64,
    margin_short: f64,
    open_fee: f64,
    close_fee: f64,
    close_today_fee: f64,
    slippage_ticks: f64,
) -> SimContractRule {
    SimContractRule {
        symbol: symbol.into(),
        name: name.into(),
        exchange: exchange.into(),
        contract_multiplier: multiplier,
        price_tick: tick,
        margin_rate_long: margin_long,
        margin_rate_short: margin_short,
        commission_mode: "per_hand".into(),
        commission_open: open_fee,
        commission_close: close_fee,
        commission_close_today: close_today_fee,
        min_order_qty: 1,
        lot_size: 1,
        max_order_qty: 0,
        daily_price_limit_up: 0.0,
        daily_price_limit_down: 0.0,
        default_slippage_ticks: slippage_ticks,
        is_custom: false,
        updated_at: "2024-01-01T00:00:00Z".into(),
    }
}

pub fn rule_from_contract(contract: &Contract) -> Option<SimContractRule> {
    if contract.multiplier <= 0.0 || contract.margin_ratio <= 0.0 {
        return None;
    }
    Some(SimContractRule {
        symbol: contract.symbol.to_uppercase(),
        name: contract.name.clone(),
        exchange: contract.exchange.clone(),
        contract_multiplier: contract.multiplier,
        price_tick: 1.0,
        margin_rate_long: contract.margin_ratio,
        margin_rate_short: contract.margin_ratio,
        commission_mode: "per_hand".into(),
        commission_open: 0.0,
        commission_close: 0.0,
        commission_close_today: 0.0,
        min_order_qty: 1,
        lot_size: 1,
        max_order_qty: 0,
        daily_price_limit_up: 0.0,
        daily_price_limit_down: 0.0,
        default_slippage_ticks: 0.0,
        is_custom: false,
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn validate_rule(rule: &SimContractRule) -> AppResult<()> {
    if rule.symbol.trim().is_empty() {
        return Err(AppError::Msg("合约代码不能为空".into()));
    }
    if rule.contract_multiplier <= 0.0 {
        return Err(AppError::Msg("合约乘数必须大于 0".into()));
    }
    if rule.price_tick <= 0.0 {
        return Err(AppError::Msg("最小变动价位必须大于 0".into()));
    }
    if rule.margin_rate_long < 0.0 || rule.margin_rate_short < 0.0 {
        return Err(AppError::Msg("保证金率不能为负".into()));
    }
    if rule.lot_size <= 0 {
        return Err(AppError::Msg("每手数量必须大于 0".into()));
    }
    if rule.max_order_qty > 0 && rule.max_order_qty < rule.min_order_qty {
        return Err(AppError::Msg("最大下单量不能小于最小下单量".into()));
    }
    if rule.commission_mode != "per_hand" && rule.commission_mode != "per_amount" {
        return Err(AppError::Msg(
            "手续费模式必须是 per_hand 或 per_amount".into(),
        ));
    }
    Ok(())
}

pub fn compute_margin(rule: &SimContractRule, price: f64, qty: i64, side: &str) -> f64 {
    let rate = if side == "short" {
        rule.margin_rate_short
    } else {
        rule.margin_rate_long
    };
    price * rule.contract_multiplier * qty as f64 * rate
}

pub fn round_to_tick(price: f64, tick: f64) -> f64 {
    if tick <= 0.0 {
        return price;
    }
    (price / tick).round() * tick
}

pub fn commission_for_trade(rule: &SimContractRule, price: f64, qty: i64, offset: &str) -> f64 {
    let fee = match offset {
        "close_today" => rule.commission_close_today,
        "close" | "close_yesterday" => rule.commission_close,
        _ => rule.commission_open,
    };
    if rule.commission_mode == "per_amount" {
        price * rule.contract_multiplier * qty as f64 * fee
    } else {
        fee * qty as f64
    }
}

pub fn estimate_order(
    rule: &SimContractRule,
    price: f64,
    qty: i64,
    side: &str,
    offset: &str,
    slippage_ticks: f64,
) -> AppResult<(f64, f64, f64)> {
    if qty <= 0 {
        return Err(AppError::Msg("手数必须大于 0".into()));
    }
    let fill_price = round_to_tick(price + slippage_ticks * rule.price_tick, rule.price_tick);
    let margin = compute_margin(rule, fill_price, qty, side);
    let commission = commission_for_trade(rule, fill_price, qty, offset);
    let slippage_cost = (fill_price - price).abs() * rule.contract_multiplier * qty as f64;
    Ok((margin, commission, slippage_cost))
}

/// 按合约规则规范化手数：必须是 lot_size 的整数倍，且落在 [min, max] 区间。
pub fn normalize_quantity(qty: i64, rule: &SimContractRule) -> AppResult<i64> {
    if qty <= 0 {
        return Err(AppError::Msg("手数必须大于 0".into()));
    }
    if rule.min_order_qty > 0 && qty < rule.min_order_qty {
        return Err(AppError::Msg(format!(
            "最小下单量为 {} 手",
            rule.min_order_qty
        )));
    }
    if rule.max_order_qty > 0 && qty > rule.max_order_qty {
        return Err(AppError::Msg(format!(
            "最大下单量为 {} 手",
            rule.max_order_qty
        )));
    }
    if rule.lot_size > 1 && qty % rule.lot_size != 0 {
        return Err(AppError::Msg(format!(
            "手数必须是 {} 的整数倍",
            rule.lot_size
        )));
    }
    Ok(qty)
}
