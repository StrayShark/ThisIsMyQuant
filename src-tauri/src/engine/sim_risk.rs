use crate::models::{SimAccount, SimContractRule, SimOrder, SimPosition, SimRiskRule};

#[derive(Debug, Clone)]
pub struct RiskViolation {
    pub rule_id: String,
    pub rule_type: String,
    pub message: String,
    pub action: String,
}

/// 对单笔下单进行风控检查。estimated_price 用于估算保证金，市价单应传入最新价。
pub fn check_order_risk(
    order: &SimOrder,
    account: &SimAccount,
    rule: &SimRiskRule,
    positions: &[SimPosition],
    contract_rule: &SimContractRule,
    estimated_price: f64,
) -> Option<RiskViolation> {
    if !rule.enabled {
        return None;
    }
    if rule.scope == "symbol" {
        if let Some(sym) = &rule.symbol {
            if sym.to_uppercase() != order.symbol.to_uppercase() {
                return None;
            }
        }
    }

    match rule.rule_type.as_str() {
        "max_lots" => check_max_lots(order, rule),
        "symbol_margin_ratio" => check_symbol_margin_ratio(
            order,
            account,
            rule,
            positions,
            contract_rule,
            estimated_price,
        ),
        "risk_ratio" => check_risk_ratio(order, account, rule, contract_rule, estimated_price),
        "loss_limit" => check_loss_limit(account, rule),
        _ => None,
    }
}

fn check_max_lots(order: &SimOrder, rule: &SimRiskRule) -> Option<RiskViolation> {
    if order.quantity as f64 > rule.threshold {
        Some(violation(
            rule,
            format!(
                "下单量 {} 手超过最大限制 {} 手",
                order.quantity, rule.threshold
            ),
        ))
    } else {
        None
    }
}

fn check_symbol_margin_ratio(
    order: &SimOrder,
    account: &SimAccount,
    rule: &SimRiskRule,
    positions: &[SimPosition],
    contract_rule: &SimContractRule,
    estimated_price: f64,
) -> Option<RiskViolation> {
    if order.offset.starts_with("close") {
        return None;
    }
    let current_margin: f64 = positions
        .iter()
        .filter(|p| p.symbol == order.symbol)
        .map(|p| p.margin)
        .sum();
    let estimated_price = estimated_price.max(1.0);
    let rate = margin_rate_for(order, contract_rule);
    let new_margin = current_margin
        + estimated_price * contract_rule.contract_multiplier * order.quantity as f64 * rate;
    let ratio = if account.equity > 0.0 {
        new_margin / account.equity
    } else {
        1.0
    };
    if ratio > rule.threshold {
        Some(violation(
            rule,
            format!(
                "品种 {} 保证金占比 {:.2}% 超过限制 {:.2}%",
                order.symbol,
                ratio * 100.0,
                rule.threshold * 100.0
            ),
        ))
    } else {
        None
    }
}

fn check_risk_ratio(
    order: &SimOrder,
    account: &SimAccount,
    rule: &SimRiskRule,
    contract_rule: &SimContractRule,
    estimated_price: f64,
) -> Option<RiskViolation> {
    if order.offset.starts_with("close") {
        return None;
    }
    let estimated_price = estimated_price.max(1.0);
    let rate = margin_rate_for(order, contract_rule);
    let additional_margin =
        estimated_price * contract_rule.contract_multiplier * order.quantity as f64 * rate;
    let new_margin_used = account.margin_used + additional_margin;
    let new_risk = if account.equity > 0.0 {
        new_margin_used / account.equity
    } else {
        1.0
    };
    if new_risk > rule.threshold {
        Some(violation(
            rule,
            format!(
                "开仓后风险度 {:.2}% 超过限制 {:.2}%",
                new_risk * 100.0,
                rule.threshold * 100.0
            ),
        ))
    } else {
        None
    }
}

fn check_loss_limit(account: &SimAccount, rule: &SimRiskRule) -> Option<RiskViolation> {
    let loss = account.initial_balance - account.equity;
    if loss > rule.threshold {
        Some(violation(
            rule,
            format!("账户亏损 {:.2} 超过止损线 {:.2}", loss, rule.threshold),
        ))
    } else {
        None
    }
}

fn margin_rate_for(order: &SimOrder, rule: &SimContractRule) -> f64 {
    if order.side == "short" || order.side == "sell" {
        rule.margin_rate_short
    } else {
        rule.margin_rate_long
    }
}

fn violation(rule: &SimRiskRule, message: String) -> RiskViolation {
    RiskViolation {
        rule_id: rule.id.clone(),
        rule_type: rule.rule_type.clone(),
        message,
        action: rule.action.clone(),
    }
}

/// 检查账户级风险，用于触发强平等动作。
pub fn check_account_risk(account: &SimAccount, rules: &[SimRiskRule]) -> Vec<RiskViolation> {
    let mut out = Vec::new();
    for rule in rules {
        if !rule.enabled || rule.scope != "account" {
            continue;
        }
        if let Some(v) = check_loss_limit(account, rule) {
            out.push(v);
        }
        if rule.rule_type == "risk_ratio" {
            let risk = if account.equity > 0.0 {
                account.margin_used / account.equity
            } else {
                1.0
            };
            if risk > rule.threshold {
                out.push(violation(
                    rule,
                    format!(
                        "账户风险度 {:.2}% 超过强平线 {:.2}%",
                        risk * 100.0,
                        rule.threshold * 100.0
                    ),
                ));
            }
        }
    }
    out
}

/// 判断一组风控违规中是否包含需要强制平仓的动作。
pub fn has_force_liquidation(violations: &[RiskViolation]) -> bool {
    violations.iter().any(|v| v.action == "force_liquidate")
}

/// 判断一组风控违规中是否包含禁止开仓的动作。
pub fn should_block_open(violations: &[RiskViolation]) -> bool {
    violations
        .iter()
        .any(|v| v.action == "block_open" || v.action == "force_liquidate")
}
