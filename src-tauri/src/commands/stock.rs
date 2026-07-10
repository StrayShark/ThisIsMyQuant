use std::sync::Arc;

use chrono::Utc;
use tauri::State;

use crate::engine::stock_factors::{
    compute_factors, matches_criteria, StockFactorInputs, StockScreenerCriteria,
};
use crate::models::{
    dt_to_iso, AStockDashboardView, ApiResponse, CancelStockPaperOrderRequest,
    CreateStockPaperAccountRequest, PlaceStockPaperOrderRequest, SaveStockWatchlistRequest,
    StockBoardView, StockDataQuality, StockDataSyncRequest, StockDataSyncStatus, StockDetailQuery,
    StockDetailView, StockFactorSnapshot, StockFinancialMetric, StockIndexQuote,
    StockIndustriesQuery, StockIndustryDetailQuery, StockMarketBreadth, StockScreenTemplate,
    StockScreenerRequest, StockScreenerResultView, StockSymbol, StockSymbolSnapshot,
    StockSymbolsQuery, StockWatchlist,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_stock_symbols(
    state: State<'_, Arc<AppState>>,
    query: StockSymbolsQuery,
) -> Result<ApiResponse<Vec<StockSymbol>>, String> {
    let db = &state.db;
    match db.list_stock_symbols(
        query.query.as_deref(),
        query.industry.as_deref(),
        query.limit.unwrap_or(500),
    ) {
        Ok(symbols) => Ok(ApiResponse::ok(symbols)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_a_stock_dashboard(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<AStockDashboardView>, String> {
    let db = &state.db;
    let now = crate::models::dt_to_iso(Utc::now());

    // 指数：优先从本地 index daily bars 取最新日
    let index_codes = vec!["000001.SH", "399001.SZ", "399006.SZ", "000688.SH"];
    let mut indices = Vec::new();
    for code in index_codes {
        let name = match code {
            "000001.SH" => "上证指数",
            "399001.SZ" => "深证成指",
            "399006.SZ" => "创业板指",
            "000688.SH" => "科创50",
            _ => code,
        };
        let bars = db.get_stock_index_daily_bars(code, 1).unwrap_or_default();
        let (close, pct_chg, amount, trade_date) = bars
            .last()
            .map(|b| (b.close, b.pct_chg, b.amount, Some(b.trade_date.clone())))
            .unwrap_or((None, None, None, None));
        indices.push(StockIndexQuote {
            index_code: code.to_string(),
            name: name.to_string(),
            close,
            pct_chg,
            amount,
            trade_date,
            source: "local".to_string(),
            updated_at: now.clone(),
        });
    }

    let breadth = db.get_stock_market_breadth().unwrap_or(StockMarketBreadth {
        trade_date: None,
        up_count: 0,
        down_count: 0,
        flat_count: 0,
        limit_up_count: 0,
        limit_down_count: 0,
        total_amount: None,
        prev_amount: None,
        amount_change_pct: None,
        source: "local".to_string(),
        updated_at: now.clone(),
    });

    let boards = match db.list_stock_boards(Some("industry")) {
        Ok(list) => {
            let mut views = Vec::new();
            for board in list.iter().take(80) {
                if let Ok(Some(snap)) = db.get_stock_board_snapshot(&board.board_code, None) {
                    views.push(StockBoardView {
                        board_code: board.board_code.clone(),
                        board_name: board.board_name.clone(),
                        board_type: board.board_type.clone(),
                        pct_chg: snap.pct_chg,
                        amount: snap.amount,
                        net_flow: snap.net_flow,
                        up_count: snap.up_count,
                        down_count: snap.down_count,
                        trade_date: Some(snap.trade_date),
                    });
                }
            }
            views.sort_by(|a, b| {
                b.pct_chg
                    .partial_cmp(&a.pct_chg)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            views
        }
        Err(_) => vec![],
    };

    let quality = StockDataQuality {
        status: if db.count_stock_symbols().unwrap_or(0) > 0 {
            "available".to_string()
        } else {
            "pending".to_string()
        },
        message: None,
        last_success_at: db.get_latest_stock_daily_bar_date().unwrap_or(None),
    };

    Ok(ApiResponse::ok(AStockDashboardView {
        indices,
        breadth,
        boards,
        trade_date: quality.last_success_at.clone(),
        source: "local".to_string(),
        updated_at: now,
        quality,
    }))
}

#[tauri::command]
pub async fn get_stock_klines(
    state: State<'_, Arc<AppState>>,
    ts_code: String,
    adjustment: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<crate::models::StockBar>>, String> {
    let adj = adjustment.unwrap_or_else(|| "none".to_string());
    match state
        .db
        .get_stock_daily_bars(&ts_code, &adj, limit.unwrap_or(250))
    {
        Ok(bars) => Ok(ApiResponse::ok(bars)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_stock_detail(
    state: State<'_, Arc<AppState>>,
    query: StockDetailQuery,
) -> Result<ApiResponse<StockDetailView>, String> {
    let db = &state.db;
    let symbol = match db.get_stock_symbol(&query.ts_code) {
        Ok(Some(s)) => s,
        Ok(None) => return Ok(ApiResponse::err("symbol not found")),
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };
    let bars = db
        .get_stock_daily_bars(&query.ts_code, "none", 250)
        .unwrap_or_default();
    let latest_bar = bars.last().cloned();
    let latest_valuation = db.get_latest_stock_valuation(&query.ts_code).ok().flatten();
    let latest_financial = db
        .get_stock_financial_metrics(&query.ts_code, 1)
        .ok()
        .and_then(|v| v.into_iter().next());
    let factor_scores = match compute_factors(&StockFactorInputs {
        bars: bars.clone(),
        financial: latest_financial.clone(),
        valuation: latest_valuation.clone(),
    }) {
        Ok(scores) => Some(StockFactorSnapshot {
            ts_code: query.ts_code.clone(),
            factor_date: latest_bar
                .as_ref()
                .map(|b| b.trade_date.clone())
                .unwrap_or_else(|| dt_to_iso(Utc::now())),
            momentum: scores.momentum,
            quality: scores.quality,
            valuation: scores.valuation,
            growth: scores.growth,
            volatility: scores.volatility,
            liquidity: scores.liquidity,
            capital_flow: scores.capital_flow,
            score: scores.score,
            factor_version: "v1".to_string(),
            source: "local".to_string(),
            updated_at: dt_to_iso(Utc::now()),
        }),
        Err(_) => None,
    };
    let related_boards = vec![]; // P1：从 stock_board_members 反查
    let quality = StockDataQuality {
        status: "available".to_string(),
        message: None,
        last_success_at: latest_bar.as_ref().map(|b| b.trade_date.clone()),
    };
    Ok(ApiResponse::ok(StockDetailView {
        symbol,
        latest_bar,
        latest_valuation,
        latest_financial,
        factor_scores,
        related_boards,
        quality,
    }))
}

#[tauri::command]
pub async fn list_stock_industries(
    state: State<'_, Arc<AppState>>,
    query: StockIndustriesQuery,
) -> Result<ApiResponse<Vec<StockBoardView>>, String> {
    let db = &state.db;
    let board_type = query.board_type.as_deref().unwrap_or("industry");
    match db.list_stock_boards(Some(board_type)) {
        Ok(boards) => {
            let mut views = Vec::new();
            for board in boards {
                if let Ok(Some(snap)) = db.get_stock_board_snapshot(&board.board_code, None) {
                    views.push(StockBoardView {
                        board_code: board.board_code,
                        board_name: board.board_name,
                        board_type: board.board_type,
                        pct_chg: snap.pct_chg,
                        amount: snap.amount,
                        net_flow: snap.net_flow,
                        up_count: snap.up_count,
                        down_count: snap.down_count,
                        trade_date: Some(snap.trade_date),
                    });
                }
            }
            views.sort_by(|a, b| {
                b.pct_chg
                    .partial_cmp(&a.pct_chg)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            Ok(ApiResponse::ok(views))
        }
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_stock_industry_detail(
    state: State<'_, Arc<AppState>>,
    query: StockIndustryDetailQuery,
) -> Result<ApiResponse<crate::models::StockBoardDetailView>, String> {
    let db = &state.db;
    let board = match db.list_stock_boards(None) {
        Ok(boards) => boards
            .into_iter()
            .find(|b| b.board_code == query.board_code),
        Err(_) => None,
    };
    let board = match board {
        Some(b) => b,
        None => return Ok(ApiResponse::err("board not found")),
    };
    let snapshot = db
        .get_stock_board_snapshot(&query.board_code, query.trade_date.as_deref())
        .ok()
        .flatten();
    let members = db
        .list_stock_board_members(&query.board_code)
        .unwrap_or_default();
    let mut snapshots: Vec<StockSymbolSnapshot> = members
        .iter()
        .filter_map(|m| {
            let sym = db.get_stock_symbol(&m.ts_code).ok()?;
            let bar = db
                .get_stock_daily_bars(&m.ts_code, "none", 1)
                .ok()?
                .into_iter()
                .next();
            let val = db.get_latest_stock_valuation(&m.ts_code).ok()?;
            Some(to_symbol_snapshot(
                sym.as_ref()?,
                bar.as_ref(),
                val.as_ref(),
            ))
        })
        .collect();
    snapshots.sort_by(|a, b| {
        b.pct_chg
            .partial_cmp(&a.pct_chg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_stocks = snapshots.iter().take(10).cloned().collect();
    let bottom_stocks = snapshots.iter().rev().take(10).cloned().collect();
    Ok(ApiResponse::ok(crate::models::StockBoardDetailView {
        board,
        snapshot,
        top_stocks,
        bottom_stocks,
        members: snapshots,
    }))
}

#[tauri::command]
pub async fn run_stock_screener(
    state: State<'_, Arc<AppState>>,
    req: StockScreenerRequest,
) -> Result<ApiResponse<StockScreenerResultView>, String> {
    let criteria: StockScreenerCriteria = match serde_json::from_str(&req.criteria_json) {
        Ok(c) => c,
        Err(e) => return Ok(ApiResponse::err(format!("invalid criteria: {e}"))),
    };
    let db = &state.db;
    let symbols = match db.list_stock_symbols(None, None, 2000) {
        Ok(s) => s,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let mut rows = Vec::new();
    let mut latest_trade_date: Option<String> = None;
    let mut latest_report_period: Option<String> = None;

    for symbol in symbols {
        let bars = db
            .get_stock_daily_bars(&symbol.ts_code, "none", 250)
            .unwrap_or_default();
        let bar = bars.last();
        let fin = db
            .get_stock_financial_metrics(&symbol.ts_code, 1)
            .ok()
            .and_then(|v| v.into_iter().next());
        let val = db
            .get_latest_stock_valuation(&symbol.ts_code)
            .ok()
            .flatten();

        if !matches_criteria(&criteria, bar, fin.as_ref(), val.as_ref()) {
            continue;
        }

        if let Some(b) = bar {
            if latest_trade_date
                .as_ref()
                .map_or(true, |d| b.trade_date > *d)
            {
                latest_trade_date = Some(b.trade_date.clone());
            }
        }
        if let Some(f) = fin.as_ref() {
            if latest_report_period
                .as_ref()
                .map_or(true, |p| f.report_period > *p)
            {
                latest_report_period = Some(f.report_period.clone());
            }
        }

        rows.push(to_symbol_snapshot(&symbol, bar, val.as_ref()));
    }

    // 按市值降序
    rows.sort_by(|a, b| {
        b.market_cap
            .partial_cmp(&a.market_cap)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let count = rows.len();

    let result = StockScreenerResultView {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.unwrap_or_else(|| "筛选结果".to_string()),
        criteria_json: req.criteria_json,
        trade_date: latest_trade_date,
        report_period: latest_report_period,
        rows,
        count,
    };

    // 可选保存结果快照
    if let Err(e) = db.save_stock_screen_result(&crate::models::StockScreenResult {
        id: result.id.clone(),
        template_id: None,
        name: result.name.clone(),
        criteria_json: result.criteria_json.clone(),
        result_json: serde_json::to_string(&result.rows).unwrap_or_default(),
        trade_date: result.trade_date.clone(),
        report_period: result.report_period.clone(),
        source_summary: Some(format!("count={}", result.count)),
        created_at: dt_to_iso(Utc::now()),
    }) {
        log::warn!("save screen result: {e}");
    }

    Ok(ApiResponse::ok(result))
}

#[tauri::command]
pub async fn save_stock_screen(
    state: State<'_, Arc<AppState>>,
    req: StockScreenerRequest,
) -> Result<ApiResponse<StockScreenTemplate>, String> {
    let template = StockScreenTemplate {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.unwrap_or_else(|| "未命名模板".to_string()),
        criteria_json: req.criteria_json,
        created_at: crate::models::dt_to_iso(Utc::now()),
        updated_at: crate::models::dt_to_iso(Utc::now()),
    };
    match state.db.save_stock_screen_template(&template) {
        Ok(_) => Ok(ApiResponse::ok(template)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_stock_screen_templates(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<StockScreenTemplate>>, String> {
    match state.db.list_stock_screen_templates() {
        Ok(templates) => Ok(ApiResponse::ok(templates)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn delete_stock_screen_template(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ApiResponse<()>, String> {
    match state.db.delete_stock_screen_template(&id) {
        Ok(_) => Ok(ApiResponse::ok(())),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_stock_financials(
    state: State<'_, Arc<AppState>>,
    ts_code: String,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<StockFinancialMetric>>, String> {
    match state
        .db
        .get_stock_financial_metrics(&ts_code, limit.unwrap_or(20))
    {
        Ok(metrics) => Ok(ApiResponse::ok(metrics)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn list_stock_watchlists(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<StockWatchlist>>, String> {
    match state.db.list_stock_watchlists() {
        Ok(watchlists) => Ok(ApiResponse::ok(watchlists)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn save_stock_watchlist(
    state: State<'_, Arc<AppState>>,
    req: SaveStockWatchlistRequest,
) -> Result<ApiResponse<StockWatchlist>, String> {
    let now = dt_to_iso(Utc::now());
    let watchlist = StockWatchlist {
        id: req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        name: req.name,
        symbols: req.symbols,
        created_at: now.clone(),
        updated_at: now,
    };
    match state.db.save_stock_watchlist(&watchlist) {
        Ok(_) => Ok(ApiResponse::ok(watchlist)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn delete_stock_watchlist(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ApiResponse<()>, String> {
    match state.db.delete_stock_watchlist(&id) {
        Ok(_) => Ok(ApiResponse::ok(())),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn summarize_stock_screen(
    state: State<'_, Arc<AppState>>,
    criteria_json: String,
    result_summary: String,
) -> Result<ApiResponse<crate::models::AnalysisReport>, String> {
    let prompt = format!(
        "你是一位 A 股研究助手。请基于以下筛选条件和结果生成一段简短总结，包含：\n\
        1. 筛选意图（在寻找什么类型的股票）；\n\
        2. 结果共同特征（行业集中度、估值/财务区间等）；\n\
        3. 主要风险或需要二次排除的点；\n\
        4. 可纳入模拟组合观察的简短理由。\n\n\
        筛选条件 JSON：{}\n\n\
        结果摘要：{}\n\n\
        必须注明数据来源和报告期，并附加免责声明（本内容仅供研究参考，不构成投资建议）。",
        criteria_json, result_summary
    );

    let llm = state.llm_snapshot();
    let provider = state.config().default_llm_provider.clone();
    match llm.complete(&prompt, "", Some(&provider)).await {
        Ok(content) => Ok(ApiResponse::ok(crate::models::AnalysisReport {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: "screen".to_string(),
            trigger: "manual".to_string(),
            provider: provider.clone(),
            prompt_version: "stock-screen-summary-v1".to_string(),
            context_summary: prompt.chars().take(200).collect(),
            content,
            created_at: dt_to_iso(Utc::now()),
            tags: vec!["a-stock".to_string(), "screen-summary".to_string()],
            dimension_summary: None,
            news_ids: vec![],
            anomaly_reason: None,
        })),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn trigger_stock_data_sync(
    state: State<'_, Arc<AppState>>,
    req: StockDataSyncRequest,
) -> Result<ApiResponse<StockDataSyncStatus>, String> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let scope = req.scope.clone();
    let sync = state.stock_sync.clone();
    let task_id2 = task_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = match scope.as_str() {
            "symbols" => sync.sync_symbols(&task_id2).await,
            "indices" => sync.sync_indices(&task_id2).await,
            "daily_bars" => sync.sync_daily_bars(&task_id2, req.symbols).await,
            "boards" => sync.sync_boards(&task_id2).await,
            "market_snapshot" | "all" => sync.sync_market_snapshot(&task_id2).await,
            _ => Ok(0),
        };
        if let Err(e) = result {
            log::warn!("stock data sync {task_id2}: {e}");
        }
    });
    let status = StockDataSyncStatus {
        task_id: task_id.clone(),
        scope: req.scope,
        status: "running".to_string(),
        message: "A 股同步任务已在后台启动".to_string(),
    };
    Ok(ApiResponse::ok(status))
}

fn to_symbol_snapshot(
    symbol: &StockSymbol,
    bar: Option<&crate::models::StockBar>,
    val: Option<&crate::models::StockValuationSnapshot>,
) -> StockSymbolSnapshot {
    StockSymbolSnapshot {
        ts_code: symbol.ts_code.clone(),
        symbol: symbol.symbol.clone(),
        name: symbol.name.clone(),
        exchange: symbol.exchange.clone(),
        industry: symbol.industry.clone(),
        close: bar.map(|b| b.close).unwrap_or(None),
        pct_chg: bar.map(|b| b.pct_chg).unwrap_or(None),
        amount: bar.map(|b| b.amount).unwrap_or(None),
        market_cap: val.and_then(|v| v.market_cap),
        pe_ttm: val.and_then(|v| v.pe_ttm),
        pb: val.and_then(|v| v.pb),
        trade_date: bar.map(|b| b.trade_date.clone()),
    }
}

#[tauri::command]
pub async fn list_stock_paper_accounts(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<Vec<crate::models::StockPaperAccount>>, String> {
    match state.db.list_stock_paper_accounts() {
        Ok(accounts) => Ok(ApiResponse::ok(accounts)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn create_stock_paper_account(
    state: State<'_, Arc<AppState>>,
    req: CreateStockPaperAccountRequest,
) -> Result<ApiResponse<crate::models::StockPaperAccount>, String> {
    match state.stock_paper.create_account(&req) {
        Ok(account) => Ok(ApiResponse::ok(account)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_stock_paper_portfolio(
    state: State<'_, Arc<AppState>>,
    account_id: String,
) -> Result<ApiResponse<crate::models::StockPaperPortfolioView>, String> {
    match state.stock_paper.get_portfolio(&account_id) {
        Ok(portfolio) => Ok(ApiResponse::ok(portfolio)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn place_stock_paper_order(
    state: State<'_, Arc<AppState>>,
    req: PlaceStockPaperOrderRequest,
) -> Result<ApiResponse<crate::models::StockPaperOrder>, String> {
    match state.stock_paper.place_order(&req) {
        Ok(order) => Ok(ApiResponse::ok(order)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn cancel_stock_paper_order(
    state: State<'_, Arc<AppState>>,
    req: CancelStockPaperOrderRequest,
) -> Result<ApiResponse<crate::models::StockPaperOrder>, String> {
    match state.stock_paper.cancel_order(&req) {
        Ok(order) => Ok(ApiResponse::ok(order)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn estimate_stock_paper_order(
    _state: State<'_, Arc<AppState>>,
    price: f64,
    quantity: i64,
    side: String,
) -> Result<ApiResponse<crate::models::StockPaperOrderEstimate>, String> {
    use crate::engine::stock_paper::StockPaperEngine;
    let (amount, total) = if side == "buy" {
        StockPaperEngine::estimate_buy_cost(price, quantity)
    } else {
        let (_, net) = StockPaperEngine::estimate_sell_proceeds(price, quantity);
        (price * quantity as f64, net)
    };
    let commission = (amount * 0.0003).max(5.0);
    let stamp_tax = if side == "sell" { amount * 0.001 } else { 0.0 };
    let transfer_fee = amount * 0.00001;
    Ok(ApiResponse::ok(crate::models::StockPaperOrderEstimate {
        estimated_amount: amount,
        commission,
        stamp_tax,
        transfer_fee,
        total_cost: total,
    }))
}

#[tauri::command]
pub async fn generate_stock_summary(
    state: State<'_, Arc<AppState>>,
    ts_code: String,
) -> Result<ApiResponse<crate::models::AnalysisReport>, String> {
    let detail = match get_stock_detail(
        state.clone(),
        StockDetailQuery {
            ts_code: ts_code.clone(),
        },
    )
    .await
    {
        Ok(resp) => resp
            .data
            .ok_or_else(|| "detail response empty".to_string())?,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let prompt = format!(
        "请基于以下 A 股信息生成一份简短的研究速览，包含业务概览、近期价格行为、财务质量、估值位置、主要风险和观察点。\
        必须注明数据来源和报告期，并附加免责声明（本内容仅供研究参考，不构成投资建议）。\n\n\
        股票：{} ({})\n\
        最新价：{}\n\
        涨跌幅：{}%\n\
        报告期：{}\n\
        营收：{}\n\
        净利润同比：{}%\n\
        ROE：{}%\n\
        PE(TTM)：{}\n\
        PB：{}\n",
        detail.symbol.name,
        detail.symbol.ts_code,
        detail.latest_bar.as_ref().and_then(|b| b.close).map(|v| v.to_string()).unwrap_or_else(|| "--".to_string()),
        detail.latest_bar.as_ref().and_then(|b| b.pct_chg).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "--".to_string()),
        detail.latest_financial.as_ref().map(|f| f.report_period.clone()).unwrap_or_else(|| "--".to_string()),
        detail.latest_financial.as_ref().and_then(|f| f.revenue).map(|v| format!("{:.0}", v)).unwrap_or_else(|| "--".to_string()),
        detail.latest_financial.as_ref().and_then(|f| f.net_profit_yoy).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "--".to_string()),
        detail.latest_financial.as_ref().and_then(|f| f.roe).map(|v| format!("{:.2}", v)).unwrap_or_else(|| "--".to_string()),
        detail.latest_valuation.as_ref().and_then(|v| v.pe_ttm).map(|v| v.to_string()).unwrap_or_else(|| "--".to_string()),
        detail.latest_valuation.as_ref().and_then(|v| v.pb).map(|v| v.to_string()).unwrap_or_else(|| "--".to_string()),
    );

    let llm = state.llm_snapshot();
    let provider = state.config().default_llm_provider.clone();
    match llm.complete(&prompt, "", Some(&provider)).await {
        Ok(content) => Ok(ApiResponse::ok(crate::models::AnalysisReport {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: ts_code,
            trigger: "manual".to_string(),
            provider: provider.clone(),
            prompt_version: "stock-summary-v1".to_string(),
            context_summary: prompt.chars().take(200).collect(),
            content,
            created_at: dt_to_iso(Utc::now()),
            tags: vec!["a-stock".to_string(), "summary".to_string()],
            dimension_summary: None,
            news_ids: vec![],
            anomaly_reason: None,
        })),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn generate_stock_portfolio_review(
    state: State<'_, Arc<AppState>>,
    account_id: String,
) -> Result<ApiResponse<crate::models::AnalysisReport>, String> {
    let portfolio = match state.stock_paper.get_portfolio(&account_id) {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::err(e.to_string())),
    };

    let positions_summary = portfolio
        .positions
        .iter()
        .map(|p| {
            format!(
                "{} {} 持仓{} 成本{} 市值{} 浮动盈亏{}",
                p.ts_code, p.name, p.quantity, p.avg_cost, p.market_value, p.unrealized_pnl
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "请基于以下 A 股模拟组合持仓生成一份复盘总结，包含组合表现、行业暴露、风险点和下一步观察建议。\
        必须附加免责声明（本内容仅供研究参考，不构成投资建议）。\n\n\
        账户：{}\n\
        初始资金：{}\n\
        总资产：{}\n\
        累计已实现盈亏：{}\n\
        浮动盈亏：{}\n\n\
        持仓：\n{}\n",
        portfolio.account.name,
        portfolio.account.initial_balance,
        portfolio.account.total_equity,
        portfolio.account.realized_pnl,
        portfolio.account.unrealized_pnl,
        positions_summary
    );

    let llm = state.llm_snapshot();
    let provider = state.config().default_llm_provider.clone();
    match llm.complete(&prompt, "", Some(&provider)).await {
        Ok(content) => Ok(ApiResponse::ok(crate::models::AnalysisReport {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: account_id.clone(),
            trigger: "manual".to_string(),
            provider: provider.clone(),
            prompt_version: "stock-portfolio-review-v1".to_string(),
            context_summary: prompt.chars().take(200).collect(),
            content,
            created_at: dt_to_iso(Utc::now()),
            tags: vec!["a-stock".to_string(), "portfolio-review".to_string()],
            dimension_summary: None,
            news_ids: vec![],
            anomaly_reason: None,
        })),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}
