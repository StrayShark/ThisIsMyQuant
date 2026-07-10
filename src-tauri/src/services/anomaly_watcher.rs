//! 异动触发：检测后自动发起 anomaly 分析、持仓风险联动并推送通知。

use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::engine::anomaly::AnomalyDetector;
use crate::models::NotificationEvent;
use crate::services::run_analysis;
use crate::state::AppState;

pub struct AnomalyWatcher {
    detector: Arc<AnomalyDetector>,
}

impl AnomalyWatcher {
    pub fn new(detector: Arc<AnomalyDetector>) -> Self {
        Self { detector }
    }

    pub fn detector(&self) -> &Arc<AnomalyDetector> {
        &self.detector
    }

    pub fn update_config(&self, cfg: crate::engine::anomaly::AnomalyConfig) {
        self.detector.update_config(cfg);
    }

    pub fn on_tick(&self, state: &Arc<AppState>, app: &AppHandle, tick: &crate::models::Tick) {
        let Some(reason) = self.detector.on_tick(tick) else {
            return;
        };

        let sym = tick.symbol.clone();
        let reason_owned = reason.clone();
        let impact = evaluate_position_risk(state, &sym, tick.last_price);

        log::info!("anomaly trigger {sym}: {reason_owned}");

        // 持仓风险联动通知
        if let Some(impact) = impact {
            let body = format!("{}\n\n持仓风险联动：{}", reason_owned, impact.description);
            let _ = app.emit(
                "notification",
                NotificationEvent {
                    msg_type: "notification".into(),
                    level: "warning".into(),
                    title: format!("{} 异动触发", sym),
                    body,
                    link: Some("/simulation".into()),
                },
            );
            let _ = app.emit("anomaly-position-risk", impact);
        } else {
            let _ = app.emit(
                "notification",
                NotificationEvent {
                    msg_type: "notification".into(),
                    level: "warning".into(),
                    title: format!("{} 异动触发", sym),
                    body: reason_owned.clone(),
                    link: Some("/reports".into()),
                },
            );
        }

        if state.llm_snapshot().available_providers().is_empty() {
            log::warn!("anomaly detected for {} but no LLM configured", tick.symbol);
            return;
        }

        let st = Arc::clone(state);
        let app = app.clone();

        tauri::async_runtime::spawn(async move {
            match run_analysis(
                &st,
                Some(&app),
                &sym,
                "anomaly",
                None,
                false,
                Some(&reason_owned),
            )
            .await
            {
                Ok(report) => {
                    let _ = app.emit(
                        "notification",
                        NotificationEvent {
                            msg_type: "notification".into(),
                            level: "info".into(),
                            title: format!("{} 异动分析报告", report.symbol),
                            body: report.context_summary.clone(),
                            link: Some(format!("/reports/{}", report.id)),
                        },
                    );
                }
                Err(e) => log::warn!("anomaly analysis failed for {sym}: {e}"),
            }
        });
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PositionRiskImpact {
    pub symbol: String,
    pub account_id: String,
    pub account_name: String,
    pub position_side: String,
    pub position_qty: i64,
    pub avg_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub pnl_change_if_hit: f64,
    pub risk_ratio: f64,
    pub description: String,
}

fn evaluate_position_risk(
    state: &Arc<AppState>,
    symbol: &str,
    current_price: f64,
) -> Option<PositionRiskImpact> {
    let positions = state.sim_trading.list_positions_by_symbol(symbol).ok()?;
    if positions.is_empty() {
        return None;
    }

    let accounts = state.sim_trading.list_accounts().ok()?;
    let account_map: std::collections::HashMap<String, crate::models::SimAccount> =
        accounts.into_iter().map(|a| (a.id.clone(), a)).collect();

    // 取风险暴露最大的持仓
    let mut worst: Option<PositionRiskImpact> = None;
    for pos in positions {
        let account = match account_map.get(&pos.account_id) {
            Some(a) => a,
            None => continue,
        };
        let pnl_change = if pos.position_side == "long" {
            (current_price - pos.avg_price) * pos.total_qty as f64
        } else {
            (pos.avg_price - current_price) * pos.total_qty as f64
        };
        let risk_ratio = crate::engine::sim_account::risk_ratio(account);
        let description = format!(
            "账户 {} 持有 {}{} {} 手，均价 {:.2}，当前价 {:.2}，浮盈 {:.2}，风险度 {:.1}%",
            account.name,
            pos.symbol,
            if pos.position_side == "long" {
                "多"
            } else {
                "空"
            },
            pos.total_qty,
            pos.avg_price,
            current_price,
            pos.unrealized_pnl,
            risk_ratio * 100.0
        );
        let impact = PositionRiskImpact {
            symbol: pos.symbol.clone(),
            account_id: pos.account_id.clone(),
            account_name: account.name.clone(),
            position_side: pos.position_side.clone(),
            position_qty: pos.total_qty,
            avg_price: pos.avg_price,
            current_price,
            unrealized_pnl: pos.unrealized_pnl,
            pnl_change_if_hit: pnl_change,
            risk_ratio,
            description,
        };
        match &worst {
            Some(w) if w.unrealized_pnl.abs() >= impact.unrealized_pnl.abs() => {}
            _ => worst = Some(impact),
        }
    }
    worst
}
