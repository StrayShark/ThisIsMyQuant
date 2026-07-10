use std::sync::Arc;

use tauri::Emitter;

use crate::db::Database;
use crate::engine::sectors::get_product_by_symbol;
use crate::engine::sim_account::{initial_account, recalc_account, risk_ratio};
use crate::engine::sim_contract::{
    default_rules, estimate_order, normalize_quantity, validate_rule,
};
use crate::engine::sim_matching::{
    cancel_oco_pair, mark_to_market, match_order, process_stop_and_condition_orders,
    quote_liquidity_for, update_position_after_trade,
};
use crate::engine::sim_risk::{
    check_account_risk, check_order_risk, has_force_liquidation, should_block_open,
};
use crate::error::{AppError, AppResult};
use crate::models::{
    PlaceSimOrderRequest, RealtimeQuote, SimAccount, SimAccountSnapshot, SimContractRule,
    SimEquitySnapshot, SimJournalEntry, SimOrder, SimOrderEstimate, SimPerformance, SimPosition,
    SimRiskEvent, SimRiskRule, SimTrade,
};
use crate::services::QuoteCache;

pub struct SimTradingService {
    db: Arc<Database>,
    quote_cache: std::sync::Arc<std::sync::RwLock<QuoteCache>>,
}

impl SimTradingService {
    pub fn db(&self) -> &Arc<Database> {
        &self.db
    }

    pub fn new(
        db: Arc<Database>,
        quote_cache: std::sync::Arc<std::sync::RwLock<QuoteCache>>,
    ) -> Self {
        Self { db, quote_cache }
    }

    pub fn init_defaults(&self) -> AppResult<()> {
        let rules = default_rules();
        for rule in rules {
            if self.db.get_sim_contract_rule(&rule.symbol)?.is_none() {
                self.db.save_sim_contract_rule(&rule)?;
            }
        }
        if self.db.list_sim_accounts()?.is_empty() {
            let account = initial_account("默认模拟账户", 1_000_000.0);
            self.db.save_sim_account(&account)?;
        }
        Ok(())
    }

    pub fn default_account(&self) -> AppResult<SimAccount> {
        let accounts = self.db.list_sim_accounts()?;
        accounts
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Msg("没有可用的模拟账户".into()))
    }

    pub fn get_rule(&self, symbol: &str) -> AppResult<SimContractRule> {
        self.db
            .get_sim_contract_rule(symbol)?
            .or_else(|| {
                self.db
                    .get_sim_contract_rule(&symbol.to_uppercase())
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| AppError::Msg(format!("未找到合约规则: {symbol}")))
    }

    pub fn list_accounts(&self) -> AppResult<Vec<SimAccount>> {
        self.db.list_sim_accounts()
    }

    pub fn create_account(&self, name: String, initial_balance: f64) -> AppResult<SimAccount> {
        if initial_balance <= 0.0 {
            return Err(AppError::Msg("初始资金必须大于 0".into()));
        }
        let account = initial_account(name, initial_balance);
        self.db.save_sim_account(&account)?;
        Ok(account)
    }

    pub fn reset_account(&self, account_id: &str) -> AppResult<SimAccount> {
        let mut account = self
            .db
            .get_sim_account(account_id)?
            .ok_or_else(|| AppError::Msg("账户不存在".into()))?;
        account.cash_balance = account.initial_balance;
        account.equity = account.initial_balance;
        account.margin_used = 0.0;
        account.realized_pnl = 0.0;
        account.unrealized_pnl = 0.0;
        account.status = "active".into();
        account.updated_at = chrono::Utc::now().to_rfc3339();
        self.db.save_sim_account(&account)?;
        Ok(account)
    }

    pub fn get_snapshot(&self, account_id: Option<&str>) -> AppResult<SimAccountSnapshot> {
        let account = match account_id {
            Some(id) => self
                .db
                .get_sim_account(id)?
                .ok_or_else(|| AppError::Msg("账户不存在".into()))?,
            None => self.default_account()?,
        };
        let positions = self.db.list_sim_positions(Some(&account.id))?;
        let pending = self
            .db
            .list_sim_orders(Some(&account.id), Some("open"), 1000)?
            .len();
        let today_pnl = self.calc_today_pnl(&account)?;
        let risk = risk_ratio(&account);
        Ok(SimAccountSnapshot {
            account,
            positions,
            risk_ratio: risk,
            today_pnl,
            pending_orders: pending,
        })
    }

    fn calc_today_pnl(&self, account: &SimAccount) -> AppResult<f64> {
        let start_of_day = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|n| n.and_utc().to_rfc3339())
            .unwrap_or_default();
        let snapshots = self
            .db
            .list_sim_equity_snapshots(&account.id, 1000)?
            .into_iter()
            .filter(|s| s.snapshot_at >= start_of_day)
            .collect::<Vec<_>>();
        if let Some(first) = snapshots.last() {
            Ok(account.equity - first.equity)
        } else {
            Ok(account.equity - account.initial_balance)
        }
    }

    pub fn estimate_order(&self, req: &PlaceSimOrderRequest) -> AppResult<SimOrderEstimate> {
        let rule = self.get_rule(&req.symbol)?;
        let price = req.price.unwrap_or_else(|| self.last_price(&req.symbol));
        let qty = normalize_quantity(req.quantity, &rule)?;
        let (margin, commission, slippage_cost) = estimate_order(
            &rule,
            price,
            qty,
            &req.side,
            &req.offset,
            rule.default_slippage_ticks,
        )?;
        Ok(SimOrderEstimate {
            margin_required: margin,
            commission_estimate: commission,
            slippage_estimate: slippage_cost,
            total_cost: margin + commission + slippage_cost,
        })
    }

    pub fn place_order(&self, req: PlaceSimOrderRequest) -> AppResult<SimOrder> {
        let account = match self.db.get_sim_account(&req.account_id)? {
            Some(a) => a,
            None => return Err(AppError::Msg("账户不存在".into())),
        };
        if account.status != "active" {
            return Err(AppError::Msg("账户非活跃状态".into()));
        }
        let rule = self.get_rule(&req.symbol)?;
        let qty = normalize_quantity(req.quantity, &rule)?;
        let product =
            get_product_by_symbol(&req.symbol).ok_or_else(|| AppError::Msg("未知品种".into()))?;

        // 风控检查
        let risk_rules = self.db.list_sim_risk_rules(Some(&account.id))?;
        let positions = self.db.list_sim_positions(Some(&account.id))?;
        let probe_order = SimOrder {
            id: String::new(),
            account_id: account.id.clone(),
            symbol: req.symbol.to_uppercase(),
            name: product.name.clone(),
            side: req.side.clone(),
            offset: req.offset.clone(),
            order_type: req.order_type.clone(),
            price: req.price,
            trigger_price: req.trigger_price,
            stop_loss_price: req.stop_loss_price,
            take_profit_price: req.take_profit_price,
            oco_group_id: None,
            parent_order_id: None,
            tif: req.tif.clone(),
            condition_operator: req.condition_operator.clone(),
            trailing_distance_ticks: req.trailing_distance_ticks,
            trailing_reference_price: None,
            quantity: qty,
            filled_quantity: 0,
            status: "open".into(),
            reason: None,
            source: "manual".into(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        let estimated_price = match req.order_type.as_str() {
            "market" | "stop" | "stop_market" | "take_profit" | "take_profit_limit"
            | "trailing_stop" | "condition" => self.last_price(&req.symbol),
            _ => req.price.unwrap_or_else(|| self.last_price(&req.symbol)),
        };
        let violations: Vec<_> = risk_rules
            .iter()
            .filter_map(|r| {
                check_order_risk(
                    &probe_order,
                    &account,
                    r,
                    &positions,
                    &rule,
                    estimated_price,
                )
            })
            .collect();
        if should_block_open(&violations) {
            let msg = violations
                .first()
                .map(|v| format!("风控禁止开仓: {}", v.message))
                .unwrap_or_else(|| "风控禁止开仓".into());
            return Err(AppError::Msg(msg));
        }
        for violation in &violations {
            if violation.action == "reject" {
                return Err(AppError::Msg(format!("风控拦截: {}", violation.message)));
            }
        }

        let price = match req.order_type.as_str() {
            "market" | "stop" | "stop_market" | "take_profit" | "trailing_stop" | "condition" => {
                None
            }
            _ => Some(crate::engine::sim_contract::round_to_tick(
                req.price.unwrap_or_else(|| self.last_price(&req.symbol)),
                rule.price_tick,
            )),
        };

        let estimate = self.estimate_order(&req)?;
        if req.offset == "open" && account.cash_balance < estimate.total_cost {
            return Err(AppError::Msg("可用资金不足".into()));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let trailing_reference = if req.order_type == "trailing_stop" {
            Some(self.last_price(&req.symbol))
        } else {
            None
        };
        let order = SimOrder {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account.id.clone(),
            symbol: req.symbol.to_uppercase(),
            name: product.name.clone(),
            side: req.side.clone(),
            offset: req.offset.clone(),
            order_type: req.order_type.clone(),
            price,
            trigger_price: req.trigger_price,
            stop_loss_price: req.stop_loss_price,
            take_profit_price: req.take_profit_price,
            oco_group_id: req.oco_group_id.clone(),
            parent_order_id: req.parent_order_id.clone(),
            tif: req.tif.clone(),
            condition_operator: req.condition_operator.clone(),
            trailing_distance_ticks: req.trailing_distance_ticks,
            trailing_reference_price: trailing_reference,
            quantity: qty,
            filled_quantity: 0,
            status: "open".into(),
            reason: None,
            source: "manual".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        self.db.save_sim_order(&order)?;

        // 对 STOP / 止盈 / 移动止损 / 条件单不立即撮合，只保存挂单。
        let is_algo = matches!(
            order.order_type.as_str(),
            "stop"
                | "stop_market"
                | "stop_limit"
                | "take_profit"
                | "take_profit_limit"
                | "trailing_stop"
                | "condition"
        );

        if !is_algo {
            if let Ok(filled) = self.try_fill(&order, &rule) {
                return Ok(filled);
            }
        }

        Ok(order)
    }

    /// 为父单已成交的部分生成或追加止损/止盈子单。
    fn ensure_child_orders_for_fill(
        &self,
        parent: &SimOrder,
        delta_qty: i64,
        rule: &SimContractRule,
    ) -> AppResult<()> {
        if parent.offset != "open" || delta_qty <= 0 {
            return Ok(());
        }
        let has_sl = parent.stop_loss_price.is_some();
        let has_tp = parent.take_profit_price.is_some();
        if !has_sl && !has_tp {
            return Ok(());
        }

        let children: Vec<SimOrder> = self
            .db
            .list_sim_orders(Some(&parent.account_id), None, 1000)?
            .into_iter()
            .filter(|o| {
                o.parent_order_id.as_deref() == Some(&parent.id)
                    && (o.status == "open" || o.status == "partially_filled")
            })
            .collect();
        let sl_child = children
            .iter()
            .find(|o| o.reason.as_deref() == Some("止损"))
            .cloned();
        let tp_child = children
            .iter()
            .find(|o| o.reason.as_deref() == Some("止盈"))
            .cloned();

        let oco_group: Option<String> = if has_sl && has_tp {
            sl_child
                .as_ref()
                .and_then(|o| o.oco_group_id.clone())
                .or_else(|| tp_child.as_ref().and_then(|o| o.oco_group_id.clone()))
                .or_else(|| Some(uuid::Uuid::new_v4().to_string()))
        } else {
            None
        };

        let now = chrono::Utc::now().to_rfc3339();
        if let Some(sl) = parent.stop_loss_price {
            if let Some(mut child) = sl_child {
                child.quantity += delta_qty;
                child.updated_at = now.clone();
                self.db.save_sim_order(&child)?;
            } else {
                let child = SimOrder {
                    id: uuid::Uuid::new_v4().to_string(),
                    account_id: parent.account_id.clone(),
                    symbol: parent.symbol.clone(),
                    name: parent.name.clone(),
                    side: if parent.side == "buy" {
                        "sell".into()
                    } else {
                        "buy".into()
                    },
                    offset: "close".into(),
                    order_type: "stop".into(),
                    price: None,
                    trigger_price: Some(crate::engine::sim_contract::round_to_tick(
                        sl,
                        rule.price_tick,
                    )),
                    stop_loss_price: None,
                    take_profit_price: None,
                    oco_group_id: oco_group.clone(),
                    parent_order_id: Some(parent.id.clone()),
                    tif: Some("GTC".into()),
                    condition_operator: None,
                    trailing_distance_ticks: None,
                    trailing_reference_price: None,
                    quantity: delta_qty,
                    filled_quantity: 0,
                    status: "open".into(),
                    reason: Some("止损".into()),
                    source: "auto".into(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                self.db.save_sim_order(&child)?;
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        if let Some(tp) = parent.take_profit_price {
            if let Some(mut child) = tp_child {
                child.quantity += delta_qty;
                child.updated_at = now.clone();
                self.db.save_sim_order(&child)?;
            } else {
                let child = SimOrder {
                    id: uuid::Uuid::new_v4().to_string(),
                    account_id: parent.account_id.clone(),
                    symbol: parent.symbol.clone(),
                    name: parent.name.clone(),
                    side: if parent.side == "buy" {
                        "sell".into()
                    } else {
                        "buy".into()
                    },
                    offset: "close".into(),
                    order_type: "take_profit".into(),
                    price: None,
                    trigger_price: Some(crate::engine::sim_contract::round_to_tick(
                        tp,
                        rule.price_tick,
                    )),
                    stop_loss_price: None,
                    take_profit_price: None,
                    oco_group_id: oco_group.clone(),
                    parent_order_id: Some(parent.id.clone()),
                    tif: Some("GTC".into()),
                    condition_operator: None,
                    trailing_distance_ticks: None,
                    trailing_reference_price: None,
                    quantity: delta_qty,
                    filled_quantity: 0,
                    status: "open".into(),
                    reason: Some("止盈".into()),
                    source: "auto".into(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                self.db.save_sim_order(&child)?;
            }
        }
        Ok(())
    }

    fn try_fill(&self, order: &SimOrder, rule: &SimContractRule) -> AppResult<SimOrder> {
        let quote = self
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&order.symbol)
            .cloned()
            .unwrap_or_else(|| crate::models::RealtimeQuote {
                symbol: order.symbol.clone(),
                last_price: self.last_price(&order.symbol),
                bid_price: self.last_price(&order.symbol),
                ask_price: self.last_price(&order.symbol),
                bid_volume: 0,
                ask_volume: 0,
                prev_close: 0.0,
                change_pct: 0.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                forming_daily: None,
            });
        let positions = self.db.list_sim_positions(Some(&order.account_id))?;
        let liquidity = quote_liquidity_for(order, &quote);
        let remaining = order.quantity - order.filled_quantity;
        let available_liquidity = if liquidity > 0 { liquidity } else { remaining };
        let result = match_order(
            order,
            &quote,
            rule,
            rule.default_slippage_ticks,
            &positions,
            available_liquidity,
        )?;

        let mut account = self
            .db
            .get_sim_account(&order.account_id)?
            .ok_or_else(|| AppError::Msg("账户不存在".into()))?;

        // Update cash: open reduces cash by margin+commission; close adds realized pnl - commission
        if order.offset.starts_with("close") {
            account.cash_balance += result.realized_pnl - result.trade.commission;
        } else {
            account.cash_balance -= result.position_delta.margin + result.trade.commission;
        }

        self.db.save_sim_trade(&result.trade)?;

        let mut positions = positions;
        let position_keys_before: std::collections::HashSet<(String, String)> = positions
            .iter()
            .map(|p| (p.symbol.clone(), p.position_side.clone()))
            .collect();
        update_position_after_trade(
            &mut positions,
            order,
            result.trade.quantity,
            result.trade.price,
            rule,
        )?;
        let position_keys_after: std::collections::HashSet<(String, String)> = positions
            .iter()
            .map(|p| (p.symbol.clone(), p.position_side.clone()))
            .collect();
        for key in position_keys_before.difference(&position_keys_after) {
            self.db
                .delete_sim_position(&order.account_id, &key.0, &key.1)?;
        }
        for p in &positions {
            self.db.save_sim_position(p)?;
        }

        let now = chrono::Utc::now().to_rfc3339();
        let mut filled_order = order.clone();
        filled_order.filled_quantity = order.filled_quantity + result.trade.quantity;
        filled_order.status = if filled_order.filled_quantity >= order.quantity {
            "filled".into()
        } else {
            "partially_filled".into()
        };
        filled_order.updated_at = now.clone();
        self.db.save_sim_order(&filled_order)?;

        if order.offset == "open"
            && result.trade.quantity > 0
            && (order.stop_loss_price.is_some() || order.take_profit_price.is_some())
        {
            self.ensure_child_orders_for_fill(&filled_order, result.trade.quantity, rule)?;
        }

        recalc_account(&mut account, &positions, result.realized_pnl)?;
        self.db.save_sim_account(&account)?;
        self.snapshot_equity(&account)?;

        Ok(filled_order)
    }

    pub fn cancel_order(&self, order_id: &str) -> AppResult<SimOrder> {
        let mut order = self
            .db
            .get_sim_order(order_id)?
            .ok_or_else(|| AppError::Msg("委托不存在".into()))?;
        if order.status != "open" {
            return Err(AppError::Msg("只有挂单中的委托可以撤销".into()));
        }
        order.status = "cancelled".into();
        order.updated_at = chrono::Utc::now().to_rfc3339();
        self.db.save_sim_order(&order)?;
        Ok(order)
    }

    pub fn list_orders(
        &self,
        account_id: Option<&str>,
        status: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<SimOrder>> {
        self.db.list_sim_orders(account_id, status, limit)
    }

    pub fn list_trades(
        &self,
        account_id: Option<&str>,
        symbol: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<SimTrade>> {
        self.db.list_sim_trades(account_id, symbol, limit)
    }

    pub fn list_positions(&self, account_id: Option<&str>) -> AppResult<Vec<SimPosition>> {
        self.db.list_sim_positions(account_id)
    }

    pub fn list_equity_curve(
        &self,
        account_id: &str,
        days: i64,
    ) -> AppResult<Vec<SimEquitySnapshot>> {
        let mut snaps = self.db.list_sim_equity_snapshots(account_id, days * 10)?;
        snaps.reverse();
        Ok(snaps)
    }

    pub fn get_performance(&self, account_id: &str) -> AppResult<SimPerformance> {
        let account = self
            .db
            .get_sim_account(account_id)?
            .ok_or_else(|| AppError::Msg("账户不存在".into()))?;
        let trades = self.db.list_sim_trades(Some(account_id), None, 10000)?;
        let curve = self.list_equity_curve(account_id, 3650)?;

        let total_pnl = account.equity - account.initial_balance;
        let total_return_pct = if account.initial_balance > 0.0 {
            total_pnl / account.initial_balance
        } else {
            0.0
        };

        let mut max_equity = account.initial_balance;
        let mut max_drawdown = 0.0;
        let mut max_drawdown_pct = 0.0;
        for snap in &curve {
            if snap.equity > max_equity {
                max_equity = snap.equity;
            }
            let dd = max_equity - snap.equity;
            if dd > max_drawdown {
                max_drawdown = dd;
                max_drawdown_pct = if max_equity > 0.0 {
                    dd / max_equity
                } else {
                    0.0
                };
            }
        }

        let realized_pnls: Vec<f64> = trades.iter().map(|t| t.realized_pnl).collect();
        let winning: Vec<&f64> = realized_pnls.iter().filter(|p| **p > 0.0).collect();
        let losing: Vec<&f64> = realized_pnls.iter().filter(|p| **p < 0.0).collect();
        let total_trades = trades.len();
        let winning_trades = winning.len();
        let losing_trades = losing.len();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_win = if !winning.is_empty() {
            winning.iter().map(|p| **p).sum::<f64>() / winning.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losing.is_empty() {
            losing.iter().map(|p| **p).sum::<f64>().abs() / losing.len() as f64
        } else {
            0.0
        };
        let profit_loss_ratio = if avg_loss > 0.0 {
            avg_win / avg_loss
        } else {
            0.0
        };

        let mut symbol_contribution: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for t in &trades {
            *symbol_contribution.entry(t.symbol.clone()).or_insert(0.0) += t.realized_pnl;
        }

        let mut hourly_contribution: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for t in &trades {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&t.traded_at) {
                let hour = dt.format("%H:00").to_string();
                *hourly_contribution.entry(hour).or_insert(0.0) += t.realized_pnl;
            }
        }

        // 风险回报比：使用止损/止盈子单触发价估算（仅当同时存在时计算）。
        let mut risk_return_ratio = 0.0;
        if let Ok(orders) = self.db.list_sim_orders(Some(account_id), None, 10000) {
            let mut parent_to_sl: std::collections::HashMap<String, f64> =
                std::collections::HashMap::new();
            let mut parent_to_tp: std::collections::HashMap<String, f64> =
                std::collections::HashMap::new();
            for o in &orders {
                if let Some(parent) = &o.parent_order_id {
                    if let Some(trigger) = o.trigger_price {
                        if o.reason.as_deref() == Some("止损") {
                            parent_to_sl.insert(parent.clone(), trigger);
                        } else if o.reason.as_deref() == Some("止盈") {
                            parent_to_tp.insert(parent.clone(), trigger);
                        }
                    }
                }
            }
            let mut ratios = Vec::new();
            for (parent, sl) in &parent_to_sl {
                if let Some(tp) = parent_to_tp.get(parent) {
                    if *sl > 0.0 {
                        ratios.push((tp - sl).abs() / *sl);
                    }
                }
            }
            if !ratios.is_empty() {
                risk_return_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
            }
        }

        // 平均持仓时长：通过匹配同品种开平仓成交估算。
        let mut holding_hours = Vec::new();
        let mut overnight_count = 0usize;
        let mut open_events: Vec<(String, chrono::DateTime<chrono::Utc>)> = Vec::new();
        for t in trades.iter().filter(|t| t.offset == "open") {
            if let Some(dt) = chrono::DateTime::parse_from_rfc3339(&t.traded_at)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
            {
                open_events.push((t.symbol.clone(), dt));
            }
        }
        for t in trades.iter().filter(|t| t.offset.starts_with("close")) {
            if let Some(dt) = chrono::DateTime::parse_from_rfc3339(&t.traded_at)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
            {
                if let Some(idx) = open_events.iter().position(|(s, _)| s == &t.symbol) {
                    let (_, open_dt) = open_events.remove(idx);
                    let hours = (dt - open_dt).num_seconds() as f64 / 3600.0;
                    holding_hours.push(hours.max(0.0));
                    if dt.date_naive() != open_dt.date_naive() {
                        overnight_count += 1;
                    }
                }
            }
        }
        let avg_holding_hours = if !holding_hours.is_empty() {
            holding_hours.iter().sum::<f64>() / holding_hours.len() as f64
        } else {
            0.0
        };

        Ok(SimPerformance {
            account_id: account_id.into(),
            total_return: total_pnl,
            total_return_pct,
            total_pnl,
            max_drawdown,
            max_drawdown_pct,
            win_rate,
            profit_loss_ratio,
            avg_win,
            avg_loss,
            total_trades,
            winning_trades,
            losing_trades,
            risk_return_ratio,
            symbol_contribution,
            hourly_contribution,
            avg_holding_hours,
            overnight_count,
        })
    }

    pub fn save_journal(&self, mut entry: SimJournalEntry) -> AppResult<SimJournalEntry> {
        let now = chrono::Utc::now().to_rfc3339();
        if entry.id.is_empty() {
            entry.id = uuid::Uuid::new_v4().to_string();
            entry.created_at = now.clone();
        }
        entry.updated_at = now;
        if entry.account_id.is_empty() {
            entry.account_id = self.default_account()?.id;
        }
        self.db.save_sim_journal_entry(&entry)?;
        Ok(entry)
    }

    pub fn list_journals(
        &self,
        account_id: Option<&str>,
        symbol: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<SimJournalEntry>> {
        self.db.list_sim_journal_entries(account_id, symbol, limit)
    }

    pub fn list_contract_rules(&self) -> AppResult<Vec<SimContractRule>> {
        self.db.list_sim_contract_rules()
    }

    pub fn snapshot_equity(&self, account: &SimAccount) -> AppResult<()> {
        let snapshot = SimEquitySnapshot {
            account_id: account.id.clone(),
            snapshot_at: chrono::Utc::now().to_rfc3339(),
            equity: account.equity,
            cash_balance: account.cash_balance,
            margin_used: account.margin_used,
            realized_pnl: account.realized_pnl,
            unrealized_pnl: account.unrealized_pnl,
            risk_ratio: risk_ratio(account),
        };
        self.db.save_sim_equity_snapshot(&snapshot)?;
        Ok(())
    }

    pub fn seed_price(&self, symbol: &str, price: f64) {
        let ts = chrono::Utc::now().to_rfc3339();
        self.quote_cache
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .update_price(symbol, price, &ts);
    }

    #[allow(dead_code)]
    pub fn seed_quote(&self, symbol: &str, quote: RealtimeQuote) {
        let mut cache = self.quote_cache.write().unwrap_or_else(|e| e.into_inner());
        cache.insert_quote(quote);
        let _ = symbol;
    }

    pub fn on_price_update(&self, symbol: &str, price: f64) -> AppResult<Vec<String>> {
        self.seed_price(symbol, price);
        let rule = match self.get_rule(symbol) {
            Ok(r) => r,
            Err(_) => return Ok(vec![]),
        };
        let quote = self
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| crate::models::RealtimeQuote {
                symbol: symbol.into(),
                last_price: price,
                bid_price: price,
                ask_price: price,
                bid_volume: 0,
                ask_volume: 0,
                prev_close: 0.0,
                change_pct: 0.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
                forming_daily: None,
            });
        let mut affected = Vec::new();
        let accounts = self.db.list_sim_accounts()?;
        for account in accounts {
            let mut positions = self.db.list_sim_positions(Some(&account.id))?;
            mark_to_market(&mut positions, symbol, price, &rule);

            // 1. 处理 STOP / 止盈 / 移动止损 / 条件单：触发后转市价/限价并保存。
            //    同时保存移动止损的参考价/触发价更新。
            let mut orders = self.db.list_sim_orders(Some(&account.id), None, 1000)?;
            let activated = process_stop_and_condition_orders(&mut orders, &quote, rule.price_tick);
            for order in &orders {
                self.db.save_sim_order(order)?;
            }

            // 2. 撮合 open / partially_filled 订单。
            let mut account_changed = false;
            for order in activated.iter().chain(
                self.db
                    .list_sim_orders(Some(&account.id), None, 1000)?
                    .iter()
                    .filter(|o| {
                        o.symbol == symbol && (o.status == "open" || o.status == "partially_filled")
                    }),
            ) {
                if let Ok(filled) = self.try_fill(order, &rule) {
                    account_changed = true;
                    if filled.status == "filled" || filled.status == "partially_filled" {
                        // OCO：取消同组另一单
                        let mut all_orders =
                            self.db.list_sim_orders(Some(&account.id), None, 1000)?;
                        cancel_oco_pair(&mut all_orders, &filled);
                        for o in all_orders {
                            if o.status == "cancelled" {
                                self.db.save_sim_order(&o)?;
                            }
                        }
                    }
                    positions = self.db.list_sim_positions(Some(&account.id))?;
                    mark_to_market(&mut positions, symbol, price, &rule);
                }
            }

            // 3. 检查账户风控并执行强平。
            let risk_rules = self.db.list_sim_risk_rules(Some(&account.id))?;
            let mut account = self
                .db
                .get_sim_account(&account.id)?
                .ok_or_else(|| AppError::Msg("账户不存在".into()))?;
            recalc_account(&mut account, &positions, 0.0)?;
            let violations = check_account_risk(&account, &risk_rules);
            if has_force_liquidation(&violations) {
                let now = chrono::Utc::now().to_rfc3339();
                for v in violations.iter().filter(|v| v.action == "force_liquidate") {
                    let event = SimRiskEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        account_id: account.id.clone(),
                        rule_id: v.rule_id.clone(),
                        triggered_at: now.clone(),
                        description: v.message.clone(),
                        action_taken: "force_liquidate".into(),
                    };
                    if let Err(e) = self.db.save_sim_risk_event(&event) {
                        log::warn!("save risk event {}: {}", account.id, e);
                    }
                }
                if let Err(e) = self.force_liquidate(&account.id, Some(symbol)) {
                    log::warn!("force liquidate {}: {}", account.id, e);
                } else {
                    account_changed = true;
                    positions = self.db.list_sim_positions(Some(&account.id))?;
                }
            }

            recalc_account(&mut account, &positions, 0.0)?;
            self.db.save_sim_account(&account)?;
            for p in &positions {
                self.db.save_sim_position(p)?;
            }
            if account_changed {
                self.snapshot_equity(&account)?;
            }
            affected.push(account.id.clone());
        }
        Ok(affected)
    }

    pub fn force_liquidate(
        &self,
        account_id: &str,
        symbol: Option<&str>,
    ) -> AppResult<Vec<SimOrder>> {
        let positions = match symbol {
            Some(s) => self
                .db
                .list_sim_positions(Some(account_id))?
                .into_iter()
                .filter(|p| p.symbol == s)
                .collect(),
            None => self.db.list_sim_positions(Some(account_id))?,
        };
        let mut filled_orders = Vec::new();
        for pos in positions {
            if pos.total_qty <= 0 {
                continue;
            }
            let rule = self.get_rule(&pos.symbol)?;
            let order = SimOrder {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: pos.account_id.clone(),
                symbol: pos.symbol.clone(),
                name: pos.name.clone(),
                side: if pos.position_side == "long" {
                    "sell".into()
                } else {
                    "buy".into()
                },
                offset: "close".into(),
                order_type: "market".into(),
                price: None,
                trigger_price: None,
                stop_loss_price: None,
                take_profit_price: None,
                oco_group_id: None,
                parent_order_id: None,
                tif: None,
                condition_operator: None,
                trailing_distance_ticks: None,
                trailing_reference_price: None,
                quantity: pos.total_qty,
                filled_quantity: 0,
                status: "open".into(),
                reason: Some("风控强平".into()),
                source: "risk".into(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            self.db.save_sim_order(&order)?;
            if let Ok(filled) = self.try_fill(&order, &rule) {
                filled_orders.push(filled);
            }
        }
        Ok(filled_orders)
    }

    pub fn save_contract_rule(&self, mut rule: SimContractRule) -> AppResult<SimContractRule> {
        rule.symbol = rule.symbol.to_uppercase();
        validate_rule(&rule)?;
        rule.updated_at = chrono::Utc::now().to_rfc3339();
        self.db.save_sim_contract_rule(&rule)?;
        Ok(rule)
    }

    pub fn delete_contract_rule(&self, symbol: &str) -> AppResult<()> {
        self.db.delete_sim_contract_rule(&symbol.to_uppercase())
    }

    pub fn list_risk_rules(&self, account_id: Option<&str>) -> AppResult<Vec<SimRiskRule>> {
        self.db.list_sim_risk_rules(account_id)
    }

    pub fn save_risk_rule(&self, mut rule: SimRiskRule) -> AppResult<SimRiskRule> {
        if rule.id.is_empty() {
            rule.id = uuid::Uuid::new_v4().to_string();
            rule.created_at = chrono::Utc::now().to_rfc3339();
        }
        rule.updated_at = chrono::Utc::now().to_rfc3339();
        self.db.save_sim_risk_rule(&rule)?;
        Ok(rule)
    }

    pub fn delete_risk_rule(&self, id: &str) -> AppResult<()> {
        self.db.delete_sim_risk_rule(id)
    }

    pub fn list_positions_by_symbol(&self, symbol: &str) -> AppResult<Vec<SimPosition>> {
        let all = self.db.list_sim_positions(None)?;
        Ok(all.into_iter().filter(|p| p.symbol == symbol).collect())
    }

    fn last_price(&self, symbol: &str) -> f64 {
        self.quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .last_price(symbol)
            .unwrap_or(0.0)
    }
}

pub fn emit_sim_update<R: tauri::Runtime, E: Emitter<R>>(
    emitter: &E,
    account_id: &str,
) -> AppResult<()> {
    let _ = emitter.emit(
        "sim-order-update",
        serde_json::json!({ "account_id": account_id }),
    );
    let _ = emitter.emit(
        "sim-account-update",
        serde_json::json!({ "account_id": account_id }),
    );
    Ok(())
}
