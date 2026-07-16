use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::models::{
    ApiResponse, DatabaseSummary, PlaceSimOrderRequest, ReplayKlinePayload, ReplayState,
    SimAccount, SimAccountSnapshot, SimContractRule, SimEquitySnapshot, SimJournalEntry, SimOrder,
    SimOrderEstimate, SimPerformance, SimPosition, SimRiskRule, SimTrade,
};
use crate::state::AppState;

fn sim_service(state: &AppState) -> Arc<crate::services::SimTradingService> {
    state.sim_trading.clone()
}

#[tauri::command]
pub fn list_sim_accounts(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<SimAccount>>, String> {
    match sim_service(&state).list_accounts() {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn create_sim_account(
    state: State<'_, Arc<AppState>>,
    name: String,
    initial_balance: f64,
) -> Result<ApiResponse<SimAccount>, String> {
    match sim_service(&state).create_account(name, initial_balance) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn reset_sim_account(
    state: State<'_, Arc<AppState>>,
    account_id: String,
) -> Result<ApiResponse<SimAccount>, String> {
    match sim_service(&state).reset_account(&account_id) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn get_sim_account_snapshot(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
) -> Result<ApiResponse<SimAccountSnapshot>, String> {
    match sim_service(&state).get_snapshot(account_id.as_deref()) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_positions(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
) -> Result<ApiResponse<Vec<SimPosition>>, String> {
    match sim_service(&state).list_positions(account_id.as_deref()) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_orders(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<SimOrder>>, String> {
    match sim_service(&state).list_orders(
        account_id.as_deref(),
        status.as_deref(),
        limit.unwrap_or(100),
    ) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_trades(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
    symbol: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<SimTrade>>, String> {
    match sim_service(&state).list_trades(
        account_id.as_deref(),
        symbol.as_deref(),
        limit.unwrap_or(100),
    ) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn get_sim_performance(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
) -> Result<ApiResponse<SimPerformance>, String> {
    let account_id = match account_id {
        Some(id) => id,
        None => match sim_service(&state).default_account() {
            Ok(a) => a.id,
            Err(e) => return Ok(ApiResponse::err(e.to_string())),
        },
    };
    match sim_service(&state).get_performance(&account_id) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_equity_curve(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
    days: Option<i64>,
) -> Result<ApiResponse<Vec<SimEquitySnapshot>>, String> {
    let account_id = match account_id {
        Some(id) => id,
        None => match sim_service(&state).default_account() {
            Ok(a) => a.id,
            Err(e) => return Ok(ApiResponse::err(e.to_string())),
        },
    };
    match sim_service(&state).list_equity_curve(&account_id, days.unwrap_or(30)) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn place_sim_order(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    payload: PlaceSimOrderRequest,
) -> Result<ApiResponse<SimOrder>, String> {
    let service = sim_service(&state);
    match service.place_order(payload) {
        Ok(order) => {
            if let Ok(account) = service.default_account() {
                let _ = service.snapshot_equity(&account);
            }
            let _ = crate::services::emit_sim_update(&app, &order.account_id);
            Ok(ApiResponse::ok(order))
        }
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn cancel_sim_order(
    state: State<'_, Arc<AppState>>,
    order_id: String,
) -> Result<ApiResponse<SimOrder>, String> {
    match sim_service(&state).cancel_order(&order_id) {
        Ok(order) => Ok(ApiResponse::ok(order)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn estimate_sim_order(
    state: State<'_, Arc<AppState>>,
    payload: PlaceSimOrderRequest,
) -> Result<ApiResponse<SimOrderEstimate>, String> {
    match sim_service(&state).estimate_order(&payload) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_contract_rules(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<SimContractRule>>, String> {
    match sim_service(&state).list_contract_rules() {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn save_sim_contract_rule(
    state: State<'_, Arc<AppState>>,
    payload: SimContractRule,
) -> Result<ApiResponse<SimContractRule>, String> {
    match sim_service(&state).save_contract_rule(payload) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn delete_sim_contract_rule(
    state: State<'_, Arc<AppState>>,
    symbol: String,
) -> Result<ApiResponse<String>, String> {
    match sim_service(&state).delete_contract_rule(&symbol) {
        Ok(_) => Ok(ApiResponse::ok(symbol)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_risk_rules(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
) -> Result<ApiResponse<Vec<SimRiskRule>>, String> {
    match sim_service(&state).list_risk_rules(account_id.as_deref()) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn save_sim_risk_rule(
    state: State<'_, Arc<AppState>>,
    payload: SimRiskRule,
) -> Result<ApiResponse<SimRiskRule>, String> {
    match sim_service(&state).save_risk_rule(payload) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn delete_sim_risk_rule(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ApiResponse<String>, String> {
    match sim_service(&state).delete_risk_rule(&id) {
        Ok(_) => Ok(ApiResponse::ok(id)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn force_liquidate(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    account_id: String,
    symbol: Option<String>,
) -> Result<ApiResponse<Vec<SimOrder>>, String> {
    match sim_service(&state).force_liquidate(&account_id, symbol.as_deref()) {
        Ok(orders) => {
            let _ = crate::services::emit_sim_update(&app, &account_id);
            Ok(ApiResponse::ok(orders))
        }
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn save_sim_journal_entry(
    state: State<'_, Arc<AppState>>,
    payload: SimJournalEntry,
) -> Result<ApiResponse<SimJournalEntry>, String> {
    match sim_service(&state).save_journal(payload) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn list_sim_journal_entries(
    state: State<'_, Arc<AppState>>,
    account_id: Option<String>,
    symbol: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<SimJournalEntry>>, String> {
    match sim_service(&state).list_journals(
        account_id.as_deref(),
        symbol.as_deref(),
        limit.unwrap_or(100),
    ) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn start_market_replay(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    symbol: String,
    date: String,
    account_id: Option<String>,
    speed: Option<i32>,
) -> Result<ApiResponse<ReplayState>, String> {
    let symbol = symbol.to_uppercase();
    let account_id = match account_id {
        Some(id) => id,
        None => match sim_service(&state).default_account() {
            Ok(a) => a.id,
            Err(e) => return Ok(ApiResponse::err(e.to_string())),
        },
    };

    let runner = match crate::services::ReplayRunner::load(
        &state.db,
        &symbol,
        &date,
        Some("1m"),
        Some(&account_id),
        speed,
        Some(app.clone()),
    ) {
        Ok(r) => r,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    crate::services::spawn_replay_loop(app, state.inner().clone());

    let replay = runner.state();
    {
        let mut guard = state
            .replay_runner
            .write()
            .unwrap_or_else(|e| e.into_inner());
        *guard = Some(runner);
    }
    Ok(ApiResponse::ok(replay))
}

#[tauri::command]
pub fn stop_market_replay(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<ReplayState>, String> {
    let mut guard = state
        .replay_runner
        .write()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(runner) = guard.as_mut() {
        runner.set_running(false);
        let _ = runner.save_session(&state.db);
        return Ok(ApiResponse::ok(runner.state()));
    }
    Ok(ApiResponse::ok(empty_replay_state()))
}

#[tauri::command]
pub fn step_market_replay(
    state: State<'_, Arc<AppState>>,
    steps: Option<i32>,
) -> Result<ApiResponse<ReplayState>, String> {
    let steps = steps.unwrap_or(1).max(1) as usize;
    let mut guard = state
        .replay_runner
        .write()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(runner) = guard.as_mut() {
        match runner.step(steps, &state.sim_trading) {
            Ok(state) => return Ok(ApiResponse::ok(state)),
            Err(e) => return Ok(ApiResponse::err(e.to_string())),
        }
    }
    Ok(ApiResponse::ok(empty_replay_state()))
}

#[tauri::command]
pub fn get_replay_state(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<ReplayState>, String> {
    let guard = state
        .replay_runner
        .read()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(runner) = guard.as_ref() {
        return Ok(ApiResponse::ok(runner.state()));
    }
    // 若内存中无 runner，尝试从数据库恢复空壳状态用于 UI 展示
    if let Ok(Some(session)) = state.db.load_replay_session() {
        return Ok(ApiResponse::ok(ReplayState {
            running: false,
            symbol: session.symbol,
            date: session.replay_date,
            interval: session.interval,
            account_id: session.account_id,
            current_index: session.current_index,
            total_bars: 0,
            current_bar_time: None,
            current_price: 0.0,
            speed: session.speed,
            completed: false,
        }));
    }
    Ok(ApiResponse::ok(empty_replay_state()))
}

#[tauri::command]
pub fn get_replay_klines(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<ReplayKlinePayload>, String> {
    let guard = state
        .replay_runner
        .read()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(runner) = guard.as_ref() {
        let bars = runner.visible_bars();
        return Ok(ApiResponse::ok(ReplayKlinePayload {
            current_index: runner.current_index() as i32,
            total_bars: runner.bars().len() as i32,
            bars,
        }));
    }
    Ok(ApiResponse::ok(ReplayKlinePayload {
        current_index: 0,
        total_bars: 0,
        bars: vec![],
    }))
}

fn empty_replay_state() -> ReplayState {
    ReplayState {
        running: false,
        symbol: String::new(),
        date: String::new(),
        interval: String::new(),
        account_id: None,
        current_index: 0,
        total_bars: 0,
        current_bar_time: None,
        current_price: 0.0,
        speed: 1,
        completed: false,
    }
}

#[tauri::command]
pub fn get_database_summary(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<DatabaseSummary>, String> {
    let path = state.config().database_path.clone();
    match state.db.get_database_summary(&path.to_string_lossy()) {
        Ok(data) => Ok(ApiResponse::ok(data)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub fn backup_database(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<String>, String> {
    let src = state.config().database_path.clone();
    let dst = format!(
        "{}.bak.{}",
        src.display(),
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    match sqlite_vacuum_into(&src, Path::new(&dst)) {
        Ok(_) => Ok(ApiResponse::ok(dst)),
        Err(e) => Ok(ApiResponse::err(e)),
    }
}

#[tauri::command]
pub fn prepare_database_restore(
    state: State<'_, Arc<AppState>>,
    backup_path: String,
) -> Result<ApiResponse<String>, String> {
    let src = PathBuf::from(backup_path.trim());
    if !src.exists() {
        return Ok(ApiResponse::err("备份文件不存在"));
    }
    if let Err(e) = validate_sqlite_file(&src) {
        return Ok(ApiResponse::err(format!("备份文件校验失败：{e}")));
    }
    let db_path = state.config().database_path.clone();
    let restore_dir = db_path
        .parent()
        .map(|p| p.join("restore"))
        .unwrap_or_else(|| PathBuf::from("data/restore"));
    if let Err(e) = std::fs::create_dir_all(&restore_dir) {
        return Ok(ApiResponse::err(e.to_string()));
    }
    let pending = restore_dir.join("quant.restore.pending.db");
    if let Err(e) = std::fs::copy(&src, &pending) {
        return Ok(ApiResponse::err(e.to_string()));
    }
    Ok(ApiResponse::ok(format!(
        "已校验备份并复制到恢复候选：{}。为避免热覆盖 SQLite，请关闭应用后用该文件替换当前数据库。",
        pending.display()
    )))
}

fn sqlite_vacuum_into(src: &Path, dst: &Path) -> Result<(), String> {
    if dst.exists() {
        std::fs::remove_file(dst).map_err(|e| e.to_string())?;
    }
    let conn = rusqlite::Connection::open(src).map_err(|e| e.to_string())?;
    let quoted = dst.to_string_lossy().replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{}';", quoted))
        .map_err(|e| e.to_string())
}

fn validate_sqlite_file(path: &Path) -> Result<(), String> {
    let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sqlite_master", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if count <= 0 {
        return Err("未找到 SQLite schema".into());
    }
    Ok(())
}
