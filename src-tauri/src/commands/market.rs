use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use tauri::State;

use crate::adapters::{StockBarsRequest, StockDataProvider};
use crate::engine::sectors::{self, SectorView};
use crate::models::{
    ApiResponse, AssetSparklineQuery, Contract, KLine, MarketAsset, MarketAssetQuery,
    MarketAssetSearchResult, MarketBreadthBrief, MarketIndexBrief, MarketLeaderboard,
    MarketLeaderboardQuery, MarketOverview, MarketSectorBrief, RealtimeQuote, StockSymbol,
};
use crate::state::AppState;

const LIVE_STALE_AFTER_SECS: i64 = 300;

#[tauri::command]
pub async fn list_products(
    state: State<'_, Arc<AppState>>,
    tier: Option<String>,
) -> Result<ApiResponse<Vec<SectorView>>, String> {
    let filter = tier.unwrap_or_else(|| "core".into());
    let liquidity = state.db.get_latest_liquidity_map().unwrap_or_default();
    Ok(ApiResponse::ok(sectors::build_catalog(&filter, &liquidity)))
}

#[tauri::command]
pub async fn list_contracts(
    state: State<'_, Arc<AppState>>,
    exchange: Option<String>,
) -> Result<ApiResponse<Vec<Contract>>, String> {
    let result = match state.db.get_contracts(exchange.as_deref()) {
        Ok(list) if !list.is_empty() => Ok(list),
        _ => state.akshare.get_contracts().await.inspect(|contracts| {
            let _ = state.db.save_contracts(contracts);
        }),
    };
    match result {
        Ok(contracts) => Ok(ApiResponse::ok(contracts)),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_klines(
    state: State<'_, Arc<AppState>>,
    symbol: String,
    interval: String,
    start: Option<String>,
    end: Option<String>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<KLine>>, String> {
    let end_dt = end
        .and_then(|s| crate::models::parse_dt(&s))
        .unwrap_or_else(Utc::now);
    let start_dt = start
        .and_then(|s| crate::models::parse_dt(&s))
        .unwrap_or_else(|| end_dt - Duration::days(120));
    let limit = limit.unwrap_or(1000).min(10000);
    let sym = symbol.to_lowercase();

    let mut klines = state
        .db
        .get_klines(&sym, &interval, start_dt, end_dt, limit)
        .unwrap_or_default();

    let needs_fetch =
        klines.is_empty() || (interval == "1d" && crate::services::is_daily_klines_stale(&klines));

    if needs_fetch {
        match state
            .akshare
            .get_history(&sym, &interval, start_dt, end_dt)
            .await
        {
            Ok(mut fetched) if !fetched.is_empty() => {
                if fetched.len() as i64 > limit {
                    fetched = fetched.split_off(fetched.len() - limit as usize);
                }
                let _ = state.db.save_klines(&fetched);
                klines = fetched;
            }
            Ok(_) if klines.is_empty() => {
                return Ok(ApiResponse::err(format!("no kline data for {sym}")));
            }
            Err(e) if klines.is_empty() => return Ok(ApiResponse::err(e.to_string())),
            Err(e) => log::debug!("kline refresh {sym}: {e}"),
            _ => {}
        }
    }

    if interval == "1d" {
        if let Some(forming) = state
            .quote_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .forming_daily(&sym)
        {
            crate::services::merge_forming_daily(&mut klines, &forming);
        }
    }

    Ok(ApiResponse::ok(klines))
}

#[tauri::command]
pub async fn market_subscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = state.poll_handle().await {
        poll.subscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(
            serde_json::json!({ "subscribed": symbols }),
        ))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}

#[tauri::command]
pub async fn market_unsubscribe(
    state: State<'_, Arc<AppState>>,
    symbols: Vec<String>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    if let Some(poll) = state.poll_handle().await {
        poll.unsubscribe(symbols.clone()).await;
        Ok(ApiResponse::ok(
            serde_json::json!({ "unsubscribed": symbols }),
        ))
    } else {
        Ok(ApiResponse::err("market poll not running"))
    }
}

#[tauri::command]
pub async fn get_realtime_quotes(
    state: State<'_, Arc<AppState>>,
    symbols: Option<Vec<String>>,
) -> Result<ApiResponse<Vec<RealtimeQuote>>, String> {
    let list = state
        .quote_cache
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot(symbols.as_deref());
    Ok(ApiResponse::ok(list))
}

#[tauri::command]
pub async fn get_symbol_context(symbol: String) -> Result<ApiResponse<serde_json::Value>, String> {
    Ok(ApiResponse::ok(sectors::sector_context(&symbol)))
}

// ============================================================================
// CMC 重构：统一市场 API
// ============================================================================

#[tauri::command]
pub async fn get_market_overview(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<MarketOverview>, String> {
    let now = Utc::now();

    let futures_sectors = {
        let cache = state.quote_cache.read().unwrap_or_else(|e| e.into_inner());
        build_futures_sector_briefs(&cache)
    };

    let mut a_stock_indices = Vec::new();
    for (code, name) in [
        ("000001.SH", "上证指数"),
        ("399001.SZ", "深证成指"),
        ("399006.SZ", "创业板指"),
        ("000688.SH", "科创50"),
    ] {
        if state
            .db
            .get_stock_index_daily_bars(code, 1)
            .unwrap_or_default()
            .is_empty()
        {
            let req = StockBarsRequest {
                code: code.to_string(),
                adjustment: "none".to_string(),
                start_date: None,
                end_date: None,
                limit: 1,
            };
            if let Ok(bars) = state.stock_provider.list_index_bars(req).await {
                let _ = state.db.save_stock_index_daily_bars(&bars);
            }
        }
        a_stock_indices.push(index_brief_from_db(&state.db, code, name));
    }

    let market_breadth = state
        .db
        .get_stock_market_breadth()
        .ok()
        .map(|b| MarketBreadthBrief {
            up_count: b.up_count,
            down_count: b.down_count,
            total_amount: b.total_amount,
        });

    let mut data_source_health = HashMap::new();
    data_source_health.insert(
        "akshare".to_string(),
        if state.akshare_ready {
            "ready".to_string()
        } else {
            "unavailable".to_string()
        },
    );
    let jinshi_status = if state.config().jinshi_enabled {
        if state.jinshi.lock().await.is_connected() {
            "connected".to_string()
        } else {
            "enabled".to_string()
        }
    } else {
        "disabled".to_string()
    };
    data_source_health.insert("jinshi".to_string(), jinshi_status);
    data_source_health.insert("db".to_string(), "ready".to_string());
    data_source_health.insert("feed".to_string(), state.feed_source.clone());

    Ok(ApiResponse::ok(MarketOverview {
        futures_sectors,
        a_stock_indices,
        market_breadth,
        watchlist_move_count: 0,
        data_source_health,
        updated_at: now.to_rfc3339(),
    }))
}

fn build_futures_sector_briefs(cache: &crate::services::QuoteCache) -> Vec<MarketSectorBrief> {
    let catalog = sectors::build_catalog("core", &HashMap::new());
    catalog
        .into_iter()
        .map(|sector| {
            let pcts: Vec<f64> = sector
                .products
                .iter()
                .filter_map(|p| {
                    cache
                        .get(&p.symbol)
                        .filter(|q| q.prev_close > 0.0)
                        .map(|q| q.change_pct)
                })
                .collect();
            let pct_chg = if pcts.is_empty() {
                None
            } else {
                Some(pcts.iter().sum::<f64>() / pcts.len() as f64)
            };
            MarketSectorBrief {
                code: sector.code,
                name: sector.name,
                pct_chg,
            }
        })
        .collect()
}

fn index_brief_from_db(db: &crate::db::Database, code: &str, name: &str) -> MarketIndexBrief {
    let (close, pct_chg) = db
        .get_stock_index_daily_bars(code, 1)
        .ok()
        .and_then(|bars| bars.last().cloned())
        .map(|bar| (bar.close, bar.pct_chg))
        .unwrap_or((None, None));
    MarketIndexBrief {
        code: code.to_string(),
        name: name.to_string(),
        close,
        pct_chg,
    }
}

#[tauri::command]
pub async fn list_market_assets(
    state: State<'_, Arc<AppState>>,
    query: MarketAssetQuery,
) -> Result<ApiResponse<MarketAssetSearchResult>, String> {
    match fetch_market_assets(&state, &query).await {
        Ok((assets, total)) => Ok(ApiResponse::ok(MarketAssetSearchResult { assets, total })),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

async fn fetch_market_assets(
    state: &AppState,
    query: &MarketAssetQuery,
) -> Result<(Vec<MarketAsset>, i64), crate::error::AppError> {
    let mut assets: Vec<MarketAsset> = Vec::new();

    // Futures
    let include_futures = query
        .market
        .as_deref()
        .map(|m| m == "futures" || m == "all" || m.is_empty())
        .unwrap_or(true);
    if include_futures {
        let contracts = match state.db.get_contracts(None) {
            Ok(list) if !list.is_empty() => list,
            _ => match state.akshare.get_contracts().await {
                Ok(list) => {
                    let _ = state.db.save_contracts(&list);
                    list
                }
                Err(e) => {
                    log::debug!("list_market_assets futures contracts: {e}");
                    Vec::new()
                }
            },
        };
        for c in contracts {
            assets.push(futures_asset_from_contract(state, &c));
        }
    }

    // A-share
    let include_stocks = query
        .market
        .as_deref()
        .map(|m| m == "stock" || m == "all" || m.is_empty())
        .unwrap_or(true);
    if include_stocks {
        let db_query = query.query.as_deref();
        let db_industry = query.industry.as_deref();
        let limit = query.limit.unwrap_or(500).max(1);
        if state.db.count_stock_symbols().unwrap_or(0) == 0 {
            match state.stock_provider.list_symbols().await {
                Ok(symbols) if !symbols.is_empty() => {
                    let _ = state.db.save_stock_symbols(&symbols);
                }
                Ok(_) => {}
                Err(e) => log::debug!("list_market_assets live stock symbols: {e}"),
            }
        }
        match state.db.list_stock_symbols(db_query, db_industry, limit) {
            Ok(symbols) => {
                for s in symbols {
                    assets.push(stock_asset_from_symbol(state, &s));
                }
            }
            Err(e) => log::debug!("list_market_assets stock symbols: {e}"),
        }
    }

    apply_asset_filters(&mut assets, query);
    let total = assets.len() as i64;
    apply_asset_sort(&mut assets, query);

    let offset = query.offset.unwrap_or(0).max(0) as usize;
    let limit = query.limit.unwrap_or(100).max(1) as usize;
    let paged: Vec<MarketAsset> = assets.into_iter().skip(offset).take(limit).collect();

    Ok((paged, total))
}

fn futures_asset_from_contract(state: &AppState, c: &Contract) -> MarketAsset {
    let sym_lower = c.symbol.to_lowercase();
    let quote = state
        .quote_cache
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(&sym_lower)
        .cloned();

    let (price, change_pct, change_amount, updated_at, quality) = match quote {
        Some(q) => {
            let updated = q.timestamp.clone();
            let quality = if is_recent(&updated, LIVE_STALE_AFTER_SECS) {
                "live".to_string()
            } else {
                "stale".to_string()
            };
            (
                Some(q.last_price),
                Some(q.change_pct),
                if q.prev_close > 0.0 {
                    Some(q.last_price - q.prev_close)
                } else {
                    None
                },
                updated,
                quality,
            )
        }
        None => {
            let updated = Utc::now().to_rfc3339();
            (None, None, None, updated, "pending".to_string())
        }
    };

    let sector = sectors::get_sector_by_symbol(&c.symbol)
        .map(|s| s.code)
        .or_else(|| Some(c.product.clone()));

    MarketAsset {
        symbol: c.symbol.clone(),
        name: c.name.clone(),
        market: "futures".to_string(),
        sector,
        industry: None,
        category: Some(c.product.clone()),
        exchange: Some(c.exchange.clone()),
        price,
        change_pct,
        change_amount,
        turnover: None,
        volume: None,
        sparkline: None,
        quality,
        source: "quote_cache".to_string(),
        updated_at,
        watched: state.db.is_in_watchlist(&c.symbol, "futures").ok(),
        position_qty: None,
        position_side: None,
    }
}

fn stock_asset_from_symbol(state: &AppState, s: &StockSymbol) -> MarketAsset {
    let ts_code = s.ts_code.clone();
    let latest = state
        .db
        .get_stock_daily_bars(&ts_code, "none", 1)
        .ok()
        .and_then(|bars| bars.last().cloned());

    let (price, change_pct, change_amount, turnover, volume, updated_at) = match latest {
        Some(bar) => {
            let price = bar.close;
            let change_pct = bar.pct_chg;
            let change_amount = if let (Some(c), Some(p)) = (bar.close, bar.pre_close) {
                Some(c - p)
            } else {
                None
            };
            let turnover = bar.amount;
            let volume = bar.volume;
            let updated = bar.updated_at.clone();
            (price, change_pct, change_amount, turnover, volume, updated)
        }
        None => (None, None, None, None, None, s.updated_at.clone()),
    };

    MarketAsset {
        symbol: ts_code.clone(),
        name: s.name.clone(),
        market: "stock".to_string(),
        sector: s.market.clone(),
        industry: s.industry.clone(),
        category: s.industry.clone(),
        exchange: Some(s.exchange.clone()),
        price,
        change_pct,
        change_amount,
        turnover,
        volume,
        sparkline: None,
        quality: "history".to_string(),
        source: "db".to_string(),
        updated_at,
        watched: state.db.is_in_watchlist(&ts_code, "stock").ok(),
        position_qty: None,
        position_side: None,
    }
}

async fn ensure_stock_bars(state: &AppState, ts_code: &str) -> Option<crate::models::StockBar> {
    if let Some(bar) = state
        .db
        .get_stock_daily_bars(ts_code, "none", 1)
        .ok()
        .and_then(|bars| bars.last().cloned())
    {
        return Some(bar);
    }
    let req = StockBarsRequest {
        code: ts_code.to_string(),
        adjustment: "none".to_string(),
        start_date: None,
        end_date: None,
        limit: 60,
    };
    match state.stock_provider.list_stock_bars(req).await {
        Ok(bars) if !bars.is_empty() => {
            let _ = state.db.save_stock_daily_bars(&bars);
            bars.last().cloned()
        }
        Ok(_) => None,
        Err(e) => {
            log::debug!("ensure_stock_bars {ts_code}: {e}");
            None
        }
    }
}

fn is_recent(timestamp: &str, max_age_secs: i64) -> bool {
    crate::models::parse_dt(timestamp)
        .map(|dt| (Utc::now() - dt).num_seconds() <= max_age_secs)
        .unwrap_or(false)
}

fn apply_asset_filters(assets: &mut Vec<MarketAsset>, query: &MarketAssetQuery) {
    let q_lower = query.query.as_deref().map(|s| s.to_lowercase());
    let sector_lower = query.sector.as_deref().map(|s| s.to_lowercase());
    let industry_lower = query.industry.as_deref().map(|s| s.to_lowercase());
    let quality_lower = query.quality.as_deref().map(|s| s.to_lowercase());
    let min_turnover = query.min_turnover.unwrap_or(0.0);

    assets.retain(|a| {
        if let Some(q) = &q_lower {
            let symbol_match = a.symbol.to_lowercase().contains(q);
            let name_match = a.name.to_lowercase().contains(q);
            if !symbol_match && !name_match {
                return false;
            }
        }
        if let Some(s) = &sector_lower {
            let a_sector = a.sector.as_deref().unwrap_or("").to_lowercase();
            let a_market = if a.market == "stock" {
                a.sector.as_deref().unwrap_or("").to_lowercase()
            } else {
                a_sector.clone()
            };
            if !a_sector.contains(s) && !a_market.contains(s) && a.market != s.as_str() {
                return false;
            }
        }
        if let Some(i) = &industry_lower {
            let a_industry = a.industry.as_deref().unwrap_or("").to_lowercase();
            let a_category = a.category.as_deref().unwrap_or("").to_lowercase();
            if !a_industry.contains(i) && !a_category.contains(i) {
                return false;
            }
        }
        if let Some(q) = &quality_lower {
            if a.quality.to_lowercase() != *q {
                return false;
            }
        }
        if min_turnover > 0.0 {
            if let Some(t) = a.turnover {
                if t < min_turnover {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    });

    if query.watched == Some(true) {
        assets.retain(|asset| asset.watched == Some(true));
    }
}

fn apply_asset_sort(assets: &mut [MarketAsset], query: &MarketAssetQuery) {
    let sort_by = query.sort_by.as_deref().unwrap_or("updated_at");
    let desc = query.sort_desc.unwrap_or(true);

    match sort_by {
        "change_pct" => assets.sort_by(|a, b| {
            let av = a.change_pct.unwrap_or(f64::NEG_INFINITY);
            let bv = b.change_pct.unwrap_or(f64::NEG_INFINITY);
            if desc {
                bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
            }
        }),
        "turnover" => assets.sort_by(|a, b| {
            let av = a.turnover.unwrap_or(f64::NEG_INFINITY);
            let bv = b.turnover.unwrap_or(f64::NEG_INFINITY);
            if desc {
                bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
            }
        }),
        "volume" => assets.sort_by(|a, b| {
            let av = a.volume.unwrap_or(f64::NEG_INFINITY);
            let bv = b.volume.unwrap_or(f64::NEG_INFINITY);
            if desc {
                bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
            }
        }),
        _ => assets.sort_by(|a, b| {
            let ord = b.updated_at.cmp(&a.updated_at);
            if desc {
                ord
            } else {
                ord.reverse()
            }
        }),
    }
}

#[tauri::command]
pub async fn get_market_leaderboard(
    state: State<'_, Arc<AppState>>,
    query: MarketLeaderboardQuery,
) -> Result<ApiResponse<MarketLeaderboard>, String> {
    let category = query.category.to_lowercase();
    let label = match category.as_str() {
        "gainers" => "涨幅榜",
        "losers" => "跌幅榜",
        "turnover" => "成交额榜",
        "volume_spike" => "放量榜",
        _ => &query.category,
    }
    .to_string();

    let base_query = MarketAssetQuery {
        market: query.market.clone(),
        query: None,
        sector: None,
        industry: None,
        quality: None,
        watched: None,
        min_turnover: None,
        sort_by: Some(
            match category.as_str() {
                "gainers" => "change_pct",
                "losers" => "change_pct",
                "turnover" => "turnover",
                "volume_spike" => "volume",
                _ => "change_pct",
            }
            .to_string(),
        ),
        sort_desc: Some(category != "losers"),
        limit: Some(1000),
        offset: Some(0),
    };

    match fetch_market_assets(&state, &base_query).await {
        Ok((mut assets, _total)) => {
            let limit = query.limit.unwrap_or(20).clamp(1, 100) as usize;
            if category == "losers" {
                assets.reverse();
            }
            assets.truncate(limit);
            Ok(ApiResponse::ok(MarketLeaderboard {
                category: query.category,
                label,
                assets,
                updated_at: Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}

#[tauri::command]
pub async fn get_asset_sparkline(
    state: State<'_, Arc<AppState>>,
    query: AssetSparklineQuery,
) -> Result<ApiResponse<Vec<f64>>, String> {
    let points = query.points.unwrap_or(7).clamp(1, 365) as usize;
    let symbol = query.symbol.clone();
    let now = Utc::now();

    let closes: Vec<f64> = match query.market.to_lowercase().as_str() {
        "futures" => {
            let sym = symbol.to_lowercase();
            let start = now - Duration::days(points as i64 * 2 + 5);
            match state
                .db
                .get_klines(&sym, "1d", start, now, points as i64 * 2)
            {
                Ok(klines) => klines
                    .into_iter()
                    .map(|k| k.close)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .take(points)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect(),
                Err(_) => Vec::new(),
            }
        }
        "stock" => match state
            .db
            .get_stock_daily_bars(&symbol, "none", points as i64)
        {
            Ok(bars) if !bars.is_empty() => bars
                .into_iter()
                .filter_map(|b| b.close)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .take(points)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            _ => {
                let _ = ensure_stock_bars(&state, &symbol).await;
                state
                    .db
                    .get_stock_daily_bars(&symbol, "none", points as i64)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|b| b.close)
                    .collect::<Vec<_>>()
            }
        },
        _ => Vec::new(),
    };

    Ok(ApiResponse::ok(closes))
}

#[tauri::command]
pub async fn search_assets(
    state: State<'_, Arc<AppState>>,
    query: String,
    limit: Option<i64>,
) -> Result<ApiResponse<MarketAssetSearchResult>, String> {
    let q = MarketAssetQuery {
        market: None,
        query: Some(query),
        sector: None,
        industry: None,
        quality: None,
        watched: None,
        min_turnover: None,
        sort_by: None,
        sort_desc: None,
        limit,
        offset: Some(0),
    };
    match fetch_market_assets(&state, &q).await {
        Ok((assets, total)) => Ok(ApiResponse::ok(MarketAssetSearchResult { assets, total })),
        Err(e) => Ok(ApiResponse::err(e.to_string())),
    }
}
