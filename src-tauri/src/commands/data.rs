use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::State;

use crate::engine::sectors;
use crate::models::{
    AlertSignalView, ApiResponse, DecisionFlowItem, FactorSignal, FactorSnapshot, OverseasLinkView,
    ProfessionalDashboardView, ReportWorkflowItem,
};
use crate::state::AppState;

#[tauri::command]
pub async fn export_klines_csv(
    state: State<'_, Arc<AppState>>,
    symbol: String,
    interval: String,
    limit: Option<i64>,
) -> Result<ApiResponse<String>, String> {
    let end = Utc::now();
    let start = end - Duration::days(365);
    let klines = state
        .db
        .get_klines(
            &symbol.to_lowercase(),
            &interval,
            start,
            end,
            limit.unwrap_or(5000).min(10000),
        )
        .map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(crate::services::klines_to_csv(&klines)))
}

#[tauri::command]
pub async fn import_klines_csv(
    state: State<'_, Arc<AppState>>,
    csv: String,
    symbol: String,
    interval: String,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let klines = crate::services::parse_klines_csv(&csv, &symbol, &interval)?;
    let n = state.db.save_klines(&klines).map_err(|e| e.to_string())?;
    Ok(ApiResponse::ok(serde_json::json!({ "imported": n })))
}

#[tauri::command]
pub async fn get_professional_dashboard(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<ProfessionalDashboardView>, String> {
    Ok(ApiResponse::ok(
        professional_dashboard_view(state.inner()).await,
    ))
}

pub(crate) async fn professional_dashboard_view(
    state: &Arc<AppState>,
) -> ProfessionalDashboardView {
    let representatives = ["RB0", "AU0", "M0", "SC0", "EC0"];
    let latest_news = state.db.get_latest_news(24).unwrap_or_default();
    let reports = state.db.get_reports(None, None, 30).unwrap_or_default();
    let quotes = {
        let symbols: Vec<String> = representatives.iter().map(|s| (*s).to_string()).collect();
        state
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .snapshot(Some(&symbols))
    };
    let overseas = crate::adapters::list_overseas_symbols();
    let schedule = state.schedule_status.lock().await.clone();

    ProfessionalDashboardView {
        decision_flow: build_decision_flow(&latest_news),
        factors: build_factor_snapshots(state, &representatives, &quotes),
        alerts: build_alerts(&representatives, &quotes),
        report_workflow: build_report_workflow(&reports, &schedule),
        overseas_links: build_overseas_links(&overseas),
    }
}

fn build_decision_flow(news: &[crate::models::NewsItemView]) -> Vec<DecisionFlowItem> {
    news.iter()
        .take(12)
        .map(|item| {
            let best = item.classifications.iter().max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let symbol = best.map(|c| c.symbol.clone());
            let product = symbol.as_deref().and_then(sectors::get_product_by_symbol);
            let sector = symbol.as_deref().and_then(sectors::get_sector_by_symbol);
            let text = format!("{} {}", item.title, item.summary);
            DecisionFlowItem {
                id: item.id.clone(),
                title: item.title.clone(),
                summary: item.summary.clone(),
                source: item.source.clone(),
                display_time: item.display_time.clone(),
                symbol,
                product_name: product.map(|p| p.name),
                sector: sector.map(|s| s.name),
                dimension_code: best.map(|c| c.dimension_code.clone()),
                dimension_label: best.map(|c| c.dimension_label.clone()),
                impact: infer_news_impact(&text),
                confidence: best.map(|c| c.confidence).unwrap_or(0.35),
            }
        })
        .collect()
}

fn build_factor_snapshots(
    state: &Arc<AppState>,
    symbols: &[&str],
    quotes: &[crate::models::RealtimeQuote],
) -> Vec<FactorSnapshot> {
    let quote_by_symbol: std::collections::HashMap<String, crate::models::RealtimeQuote> = quotes
        .iter()
        .map(|q| (q.symbol.to_uppercase(), q.clone()))
        .collect();
    let end = Utc::now();
    let start = end - Duration::days(45);

    symbols
        .iter()
        .filter_map(|symbol| {
            let product = sectors::get_product_by_symbol(symbol)?;
            let sector = sectors::get_sector_by_symbol(symbol)?;
            let quote = quote_by_symbol.get(&symbol.to_uppercase());
            let klines = state
                .db
                .get_klines(&symbol.to_lowercase(), "1d", start, end, 40)
                .unwrap_or_default();
            let last = klines.last();
            let prev = if klines.len() >= 2 {
                klines.get(klines.len() - 2)
            } else {
                None
            };
            let change_pct = quote.map(|q| q.change_pct).or_else(|| match (last, prev) {
                (Some(l), Some(p)) if p.close > 0.0 => Some((l.close - p.close) / p.close * 100.0),
                _ => None,
            });
            let volume_value = last.map(|k| k.volume).unwrap_or_default();
            let driver = sector
                .drivers
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" / ");
            let quality = match (quote.is_some(), last.is_some()) {
                (true, true) => "live+history",
                (true, false) => "live",
                (false, true) => "history",
                (false, false) => "pending",
            }
            .to_string();

            Some(FactorSnapshot {
                sector: sector.name,
                symbol: (*symbol).to_string(),
                product_name: product.name,
                updated_at: quote
                    .map(|q| q.timestamp.clone())
                    .or_else(|| last.map(|k| k.start_time.clone()))
                    .unwrap_or_else(|| end.to_rfc3339()),
                quality,
                signals: vec![
                    FactorSignal {
                        label: "价格动量".into(),
                        value: change_pct
                            .map(|v| format!("{v:+.2}%"))
                            .unwrap_or_else(|| "待回填".into()),
                        signal: match change_pct {
                            Some(v) if v >= 1.0 => "bullish".into(),
                            Some(v) if v <= -1.0 => "bearish".into(),
                            Some(_) => "neutral".into(),
                            None => "pending".into(),
                        },
                        detail: "来自实时主力报价或最近日 K 收盘变化".into(),
                    },
                    FactorSignal {
                        label: "成交活跃度".into(),
                        value: if volume_value > 0 {
                            format!("{volume_value}")
                        } else {
                            "待回填".into()
                        },
                        signal: if volume_value > 0 {
                            "tracked"
                        } else {
                            "pending"
                        }
                        .into(),
                        detail: "使用主力连续日线成交量；后续可升级为交易所持仓排名".into(),
                    },
                    FactorSignal {
                        label: "核心驱动".into(),
                        value: driver,
                        signal: "watch".into(),
                        detail: "按板块固定因子组织，承接后续库存、利润、到港、运价等专源".into(),
                    },
                ],
            })
        })
        .collect()
}

fn build_alerts(symbols: &[&str], quotes: &[crate::models::RealtimeQuote]) -> Vec<AlertSignalView> {
    let mut alerts: Vec<_> = quotes
        .iter()
        .filter(|q| q.change_pct.abs() >= 0.8)
        .filter_map(|q| {
            let product = sectors::get_product_by_symbol(&q.symbol)?;
            let sector = sectors::get_sector_by_symbol(&q.symbol)?;
            let direction = if q.change_pct >= 0.0 {
                "上涨"
            } else {
                "下跌"
            };
            Some(AlertSignalView {
                symbol: q.symbol.to_uppercase(),
                product_name: product.name,
                sector: sector.name,
                severity: if q.change_pct.abs() >= 1.5 {
                    "high".into()
                } else {
                    "medium".into()
                },
                reason: format!("主力报价较昨收{direction}{:.2}%", q.change_pct.abs()),
                change_pct: q.change_pct,
                timestamp: q.timestamp.clone(),
            })
        })
        .collect();

    if alerts.is_empty() {
        alerts = symbols
            .iter()
            .filter_map(|symbol| {
                let product = sectors::get_product_by_symbol(symbol)?;
                let sector = sectors::get_sector_by_symbol(symbol)?;
                Some(AlertSignalView {
                    symbol: (*symbol).to_string(),
                    product_name: product.name,
                    sector: sector.name,
                    severity: "watch".into(),
                    reason: "暂无实时阈值异动，继续监控价格/成交/新闻密度".into(),
                    change_pct: 0.0,
                    timestamp: Utc::now().to_rfc3339(),
                })
            })
            .collect();
    }
    alerts.truncate(8);
    alerts
}

fn build_report_workflow(
    reports: &[crate::models::AnalysisReport],
    schedule: &crate::models::ScheduleStatus,
) -> Vec<ReportWorkflowItem> {
    let steps = [
        ("tomorrow", "盘前计划"),
        ("anomaly", "盘中异动"),
        ("short_term", "短线跟踪"),
        ("scheduled", "收盘/定时复盘"),
    ];
    steps
        .iter()
        .map(|(trigger, label)| {
            let found = reports.iter().find(|r| r.trigger == *trigger);
            ReportWorkflowItem {
                trigger: (*trigger).into(),
                label: (*label).into(),
                status: if found.is_some() {
                    "ready".into()
                } else if schedule.cycle_in_progress {
                    "running".into()
                } else {
                    "pending".into()
                },
                report_id: found.map(|r| r.id.clone()),
                symbol: found.map(|r| r.symbol.clone()),
                created_at: found.map(|r| r.created_at.clone()),
                summary: found
                    .map(|r| r.context_summary.clone())
                    .unwrap_or_else(|| "等待定时任务或手动触发生成".into()),
            }
        })
        .collect()
}

fn build_overseas_links(overseas: &serde_json::Value) -> Vec<OverseasLinkView> {
    let overseas_names = overseas
        .get("symbols")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let symbol = item.get("symbol")?.as_str()?;
                    let name = item.get("name")?.as_str()?;
                    Some((symbol.to_string(), name.to_string()))
                })
                .collect::<std::collections::HashMap<_, _>>()
        })
        .unwrap_or_default();

    let pairs = [
        ("SC0", "CL=F", "外盘原油影响内盘原油、燃油、沥青和聚酯链"),
        ("AU0", "GC=F", "COMEX 黄金、美元与美债实际利率传导沪金沪银"),
        ("CU0", "HG=F", "COMEX 铜/LME 方向影响沪铜风险偏好"),
        ("M0", "ZS=F", "CBOT 大豆影响国内豆粕、豆油和菜粕链"),
        ("EC0", "BZ=F", "能源成本与地缘事件共同影响航运风险溢价"),
    ];

    pairs
        .iter()
        .filter_map(|(local, overseas_symbol, note)| {
            let product = sectors::get_product_by_symbol(local)?;
            Some(OverseasLinkView {
                local_symbol: (*local).into(),
                local_name: product.name,
                overseas_symbol: (*overseas_symbol).into(),
                overseas_name: overseas_names
                    .get(*overseas_symbol)
                    .cloned()
                    .unwrap_or_else(|| (*overseas_symbol).into()),
                driver: infer_overseas_driver(overseas_symbol).into(),
                transmission: (*note).into(),
                status: if overseas_names.contains_key(*overseas_symbol) {
                    "tracked".into()
                } else {
                    "pending".into()
                },
            })
        })
        .collect()
}

fn infer_news_impact(text: &str) -> String {
    let bearish = [
        "下滑", "下降", "走弱", "偏弱", "承压", "累库", "增产", "宽松",
    ];
    let bullish = [
        "上涨", "上调", "偏强", "去化", "减产", "扰动", "短缺", "收紧",
    ];
    if bearish.iter().any(|kw| text.contains(kw)) {
        "bearish".into()
    } else if bullish.iter().any(|kw| text.contains(kw)) {
        "bullish".into()
    } else {
        "neutral".into()
    }
}

fn infer_overseas_driver(symbol: &str) -> &'static str {
    match symbol {
        "CL=F" | "BZ=F" | "NG=F" => "能源",
        "GC=F" | "SI=F" => "贵金属",
        "HG=F" => "有色金属",
        "ZC=F" | "ZS=F" | "ZM=F" => "农产品",
        _ => "海外参考",
    }
}
