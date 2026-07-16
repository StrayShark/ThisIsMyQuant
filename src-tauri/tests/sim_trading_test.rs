//! 模拟交易核心流程集成测试。

use std::sync::{Arc, RwLock};

use app_lib::db::Database;
use app_lib::models::{PlaceSimOrderRequest, RealtimeQuote, SimRiskRule};
use app_lib::services::{QuoteCache, SimTradingService};

fn temp_db() -> Arc<Database> {
    let path = std::env::temp_dir().join(format!(
        "thisismyquant-sim-test-{}.db",
        uuid::Uuid::new_v4()
    ));
    let db = Arc::new(Database::open(&path).expect("open temp db"));
    db.init_schema().expect("schema");
    db
}

fn test_service(db: Arc<Database>) -> Arc<SimTradingService> {
    let quote_cache = Arc::new(RwLock::new(QuoteCache::new()));
    let service = Arc::new(SimTradingService::new(db, quote_cache));
    service.init_defaults().expect("init defaults");
    service
}

fn req(
    symbol: &str,
    side: &str,
    offset: &str,
    order_type: &str,
    price: Option<f64>,
    qty: i64,
) -> PlaceSimOrderRequest {
    PlaceSimOrderRequest {
        account_id: String::new(),
        symbol: symbol.into(),
        side: side.into(),
        offset: offset.into(),
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
        quantity: qty,
    }
}

fn quote(symbol: &str, last: f64, bid: f64, ask: f64, bid_vol: i64, ask_vol: i64) -> RealtimeQuote {
    RealtimeQuote {
        symbol: symbol.into(),
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

#[test]
fn sim_service_creates_default_account() {
    let db = temp_db();
    let service = test_service(db);
    let accounts = service.list_accounts().expect("list accounts");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name, "默认模拟账户");
    assert_eq!(accounts[0].initial_balance, 1_000_000.0);
}

#[test]
fn market_order_open_updates_position_and_account() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "market", None, 2);
    order_req.account_id = account.id.clone();

    let order = service.place_order(order_req).expect("place market order");
    assert_eq!(order.status, "filled");

    let snapshot = service.get_snapshot(Some(&account.id)).expect("snapshot");
    assert_eq!(snapshot.positions.len(), 1);
    assert_eq!(snapshot.positions[0].total_qty, 2);
    assert_eq!(snapshot.positions[0].position_side, "long");
    assert!(snapshot.account.margin_used > 0.0);
    assert!(snapshot.account.cash_balance < snapshot.account.initial_balance);
}

#[test]
fn close_order_realizes_pnl_and_removes_position() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut open_req = req("RB0", "buy", "open", "market", None, 2);
    open_req.account_id = account.id.clone();
    service.place_order(open_req).expect("open");

    service.seed_price("RB0", 3600.0);
    let mut close_req = req("RB0", "sell", "close", "market", None, 2);
    close_req.account_id = account.id.clone();
    let close = service.place_order(close_req).expect("close");
    assert_eq!(close.status, "filled");

    let snapshot = service.get_snapshot(Some(&account.id)).expect("snapshot");
    assert_eq!(snapshot.positions.len(), 0);
    assert!(snapshot.account.realized_pnl > 0.0);
}

#[test]
fn limit_order_fills_when_price_update_crosses_limit() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "limit", Some(3490.0), 1);
    order_req.account_id = account.id.clone();

    // At 3500, buy limit 3490 is not executable.
    let order = service.place_order(order_req).expect("place limit order");
    assert_eq!(order.status, "open");

    // Price drops to 3485 -> limit crosses and order should fill.
    let affected = service
        .on_price_update("RB0", 3485.0)
        .expect("price update");
    assert!(affected.contains(&account.id));

    let orders = service
        .list_orders(Some(&account.id), Some("filled"), 10)
        .expect("list filled orders");
    assert_eq!(orders.len(), 1);

    let snapshot = service.get_snapshot(Some(&account.id)).expect("snapshot");
    assert_eq!(snapshot.positions.len(), 1);
    assert_eq!(snapshot.positions[0].total_qty, 1);
}

#[test]
fn limit_order_can_be_cancelled_before_fill() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "limit", Some(1000.0), 1);
    order_req.account_id = account.id.clone();

    let order = service.place_order(order_req).expect("place limit order");
    assert_eq!(order.status, "open");

    let cancelled = service.cancel_order(&order.id).expect("cancel order");
    assert_eq!(cancelled.status, "cancelled");

    let open_orders = service
        .list_orders(Some(&account.id), Some("open"), 10)
        .expect("list open orders");
    assert!(open_orders.is_empty());
}

#[test]
fn price_update_revalues_open_position() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut open_req = req("RB0", "buy", "open", "market", None, 1);
    open_req.account_id = account.id.clone();
    service.place_order(open_req).expect("open");

    let before = service.get_snapshot(Some(&account.id)).expect("before");
    let before_unrealized = before.positions[0].unrealized_pnl;
    assert!(before_unrealized.abs() < f64::EPSILON);

    service.on_price_update("RB0", 3600.0).expect("price up");
    let after = service.get_snapshot(Some(&account.id)).expect("after");
    let after_unrealized = after.positions[0].unrealized_pnl;
    assert!(after_unrealized > 0.0);
    assert!(after.account.equity > before.account.equity);
}

#[test]
fn close_limit_order_fills_on_price_update() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    // Seed a price, place a market open for 1 lot, then a limit sell close for 1 lot
    // at a price above market so it stays open. When price rises, it fills.
    service.seed_price("RB0", 3500.0);
    let mut open_req = req("RB0", "buy", "open", "market", None, 1);
    open_req.account_id = account.id.clone();
    service.place_order(open_req).expect("open");

    let mut close_req = req("RB0", "sell", "close", "limit", Some(3550.0), 1);
    close_req.account_id = account.id.clone();
    let close = service.place_order(close_req).expect("place close limit");
    assert_eq!(close.status, "open");
    assert_eq!(close.filled_quantity, 0);

    service
        .on_price_update("RB0", 3560.0)
        .expect("price crosses limit");
    let filled = service
        .list_orders(Some(&account.id), Some("filled"), 10)
        .expect("list filled");
    assert_eq!(filled.len(), 2);
    let close_filled = filled
        .iter()
        .find(|o| o.offset == "close")
        .expect("close order filled");
    assert_eq!(close_filled.filled_quantity, 1);
}

#[test]
fn partial_fill_respects_ask_volume() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    // 盘口仅 2 手，下单 5 手，应部分成交 2 手。
    service.seed_quote("RB0", quote("RB0", 3490.0, 3489.0, 3490.0, 5, 2));
    let mut order_req = req("RB0", "buy", "open", "limit", Some(3500.0), 5);
    order_req.account_id = account.id.clone();

    let order = service.place_order(order_req).expect("place order");
    assert_eq!(order.status, "partially_filled");
    assert_eq!(order.filled_quantity, 2);

    // 后续盘口补足，剩余 3 手成交。
    service.seed_quote("RB0", quote("RB0", 3490.0, 3489.0, 3490.0, 5, 10));
    service
        .on_price_update("RB0", 3490.0)
        .expect("continue fill");

    let filled = service
        .list_orders(Some(&account.id), Some("filled"), 10)
        .expect("list filled");
    assert_eq!(filled.len(), 1);
    assert_eq!(filled[0].filled_quantity, 5);
}

#[test]
fn oco_stop_loss_and_take_profit_cancel_each_other() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "market", None, 1);
    order_req.account_id = account.id.clone();
    order_req.stop_loss_price = Some(3490.0);
    order_req.take_profit_price = Some(3600.0);

    let parent = service.place_order(order_req).expect("place order");
    assert_eq!(parent.status, "filled");

    let children = service
        .list_orders(Some(&account.id), Some("open"), 100)
        .expect("list children");
    assert_eq!(children.len(), 2);
    let sl = children
        .iter()
        .find(|o| o.reason == Some("止损".into()))
        .unwrap();
    let tp = children
        .iter()
        .find(|o| o.reason == Some("止盈".into()))
        .unwrap();
    assert_eq!(sl.oco_group_id, tp.oco_group_id);
    assert!(!sl.oco_group_id.as_ref().unwrap().is_empty());

    // 价格下跌触发止损，止盈子单应被撤销。
    service
        .on_price_update("RB0", 3490.0)
        .expect("trigger stop");
    let all = service
        .list_orders(Some(&account.id), None, 100)
        .expect("list all");
    let sl_filled = all.iter().find(|o| o.id == sl.id).unwrap();
    let tp_cancelled = all.iter().find(|o| o.id == tp.id).unwrap();
    assert_eq!(sl_filled.status, "filled");
    assert_eq!(tp_cancelled.status, "cancelled");

    let snapshot = service.get_snapshot(Some(&account.id)).expect("snapshot");
    assert_eq!(snapshot.positions.len(), 0);
}

#[test]
fn condition_order_below_triggers_and_fills() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "condition", None, 1);
    order_req.account_id = account.id.clone();
    order_req.trigger_price = Some(3490.0);
    order_req.condition_operator = Some("<=".into());

    let order = service.place_order(order_req).expect("place condition");
    assert_eq!(order.status, "open");

    service.on_price_update("RB0", 3485.0).expect("trigger");
    let filled = service
        .list_orders(Some(&account.id), Some("filled"), 10)
        .expect("list filled");
    assert_eq!(filled.len(), 1);
}

#[test]
fn trailing_stop_follows_price_higher_and_triggers_on_pullback() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "market", None, 1);
    order_req.account_id = account.id.clone();
    order_req.stop_loss_price = None;
    order_req.take_profit_price = None;
    let parent = service.place_order(order_req).expect("open");
    assert_eq!(parent.status, "filled");

    // 独立挂一张移动止损子单（模拟在成交后追加）。
    let mut trailing_req = req("RB0", "sell", "close", "trailing_stop", None, 1);
    trailing_req.account_id = account.id.clone();
    trailing_req.trigger_price = Some(3495.0);
    trailing_req.trailing_distance_ticks = Some(5.0);
    service.place_order(trailing_req).expect("place trailing");

    // 价格上涨，触发价跟随上移。
    service.on_price_update("RB0", 3510.0).expect("price up");
    let open_orders = service
        .list_orders(Some(&account.id), Some("open"), 100)
        .expect("list open");
    let trailing = open_orders
        .iter()
        .find(|o| o.order_type == "trailing_stop")
        .unwrap();
    assert_eq!(trailing.trigger_price, Some(3505.0));
    let trailing_id = trailing.id.clone();

    // 价格回落到新的触发价。
    service
        .on_price_update("RB0", 3505.0)
        .expect("trigger trailing");
    let all = service
        .list_orders(Some(&account.id), None, 100)
        .expect("list all");
    let trailing_filled = all.iter().find(|o| o.id == trailing_id).unwrap();
    assert_eq!(trailing_filled.status, "filled");

    let snapshot = service.get_snapshot(Some(&account.id)).expect("snapshot");
    assert_eq!(snapshot.positions.len(), 0);
}

#[test]
fn risk_block_open_rejects_new_position() {
    let db = temp_db();
    let service = test_service(db);
    let account = service.default_account().expect("default account");

    let rule = SimRiskRule {
        id: "risk-1".into(),
        account_id: account.id.clone(),
        scope: "account".into(),
        symbol: None,
        rule_type: "risk_ratio".into(),
        threshold: 0.001,
        action: "block_open".into(),
        enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    service.save_risk_rule(rule).expect("save risk rule");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "market", None, 1);
    order_req.account_id = account.id.clone();
    let err = service.place_order(order_req).unwrap_err();
    assert!(err.to_string().contains("风控禁止开仓"));
}

#[test]
fn force_liquidation_records_risk_event() {
    let db = temp_db();
    let service = test_service(Arc::clone(&db));
    let account = service.default_account().expect("default account");

    let rule = SimRiskRule {
        id: "risk-2".into(),
        account_id: account.id.clone(),
        scope: "account".into(),
        symbol: None,
        rule_type: "loss_limit".into(),
        threshold: 1.0,
        action: "force_liquidate".into(),
        enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    service.save_risk_rule(rule).expect("save risk rule");

    service.seed_price("RB0", 3500.0);
    let mut order_req = req("RB0", "buy", "open", "market", None, 10);
    order_req.account_id = account.id.clone();
    service.place_order(order_req).expect("open");

    // 价格大幅下跌触发亏损限额强平。
    service
        .on_price_update("RB0", 2000.0)
        .expect("price crash triggers liquidation");

    let events = db
        .list_sim_risk_events(&account.id, 10)
        .expect("list risk events");
    assert!(!events.is_empty());
    assert_eq!(events[0].action_taken, "force_liquidate");
}
