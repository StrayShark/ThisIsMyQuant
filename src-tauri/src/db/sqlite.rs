use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::config::UserPreferences;
use crate::engine::dimensions;
use crate::error::{AppError, AppResult};
use crate::models::parse_dt;
use crate::models::{
    dt_to_iso, AnalysisReport, CalendarEvent, Contract, DataDomain, DataDomainTimeRange,
    DatabaseDomainSummary, DatabaseSummary, DatabaseTableStats, DimensionFact, FollowupMessage,
    KLine, LiquiditySnapshot, NewsClassification, NewsClassificationView, NewsItemView, NewsRecord,
    SimAccount, SimContractRule, SimEquitySnapshot, SimJournalEntry, SimOrder, SimPosition,
    SimRiskEvent, SimRiskRule, SimTrade, StockBar, StockBoard, StockBoardMember,
    StockBoardSnapshot, StockFactorSnapshot, StockFinancialMetric, StockIndexBar,
    StockMarketBreadth, StockPaperAccount, StockPaperOrder, StockPaperPosition, StockPaperTrade,
    StockScreenResult, StockScreenTemplate, StockSymbol, StockValuationSnapshot, StockWatchlist,
    Tick, WatchlistGroup, WatchlistItem, WatchlistSummary,
};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Msg(e.to_string()))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        log::info!("SQLite connected: {}", path.display());
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init_schema(&self) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS contracts (
                symbol TEXT PRIMARY KEY,
                exchange TEXT, name TEXT, product TEXT,
                multiplier REAL, margin_ratio REAL,
                listing_date TEXT, expiry_date TEXT, updated_at TEXT
            );
            CREATE TABLE IF NOT EXISTS klines (
                symbol TEXT, interval TEXT, start_time TEXT,
                open REAL, high REAL, low REAL, close REAL,
                volume INTEGER, turnover REAL,
                PRIMARY KEY (symbol, interval, start_time)
            );
            CREATE INDEX IF NOT EXISTS idx_klines_symbol_interval_time
                ON klines(symbol, interval, start_time);
            CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY, symbol TEXT, trigger TEXT,
                provider TEXT, prompt_version TEXT, context_summary TEXT,
                content TEXT, created_at TEXT, tags TEXT,
                dimension_summary TEXT, news_ids TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_reports_symbol_created
                ON reports(symbol, created_at);
            CREATE TABLE IF NOT EXISTS liquidity_snapshots (
                symbol TEXT NOT NULL,
                scored_at TEXT NOT NULL,
                volume_20d REAL,
                turnover_20d REAL,
                score REAL,
                tier TEXT NOT NULL,
                PRIMARY KEY (symbol, scored_at)
            );
            CREATE INDEX IF NOT EXISTS idx_liq_symbol_time
                ON liquidity_snapshots(symbol, scored_at DESC);
            CREATE TABLE IF NOT EXISTS analysis_dimensions (
                code TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                description TEXT
            );
            CREATE TABLE IF NOT EXISTS news_items (
                id TEXT PRIMARY KEY,
                source TEXT DEFAULT 'jin10',
                category_id INTEGER,
                title TEXT NOT NULL,
                summary TEXT,
                url TEXT,
                display_time TEXT NOT NULL,
                content_hash TEXT UNIQUE,
                ingested_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_news_time ON news_items(display_time);
            CREATE INDEX IF NOT EXISTS idx_news_hash ON news_items(content_hash);
            CREATE TABLE IF NOT EXISTS news_classifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                news_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                dimension_code TEXT NOT NULL,
                confidence REAL NOT NULL,
                method TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(news_id, symbol, dimension_code)
            );
            CREATE INDEX IF NOT EXISTS idx_nc_symbol_dim
                ON news_classifications(symbol, dimension_code, created_at);
            CREATE TABLE IF NOT EXISTS dimension_facts (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                dimension_code TEXT NOT NULL,
                fact TEXT NOT NULL,
                source_news_id TEXT,
                source_report_id TEXT,
                valid_until TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_df_symbol_created
                ON dimension_facts(symbol, created_at DESC);
            CREATE TABLE IF NOT EXISTS followup_messages (
                id TEXT PRIMARY KEY,
                report_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                question TEXT NOT NULL,
                answer TEXT NOT NULL,
                provider TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_followup_report
                ON followup_messages(report_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_followup_symbol
                ON followup_messages(symbol, created_at DESC);
            CREATE TABLE IF NOT EXISTS calendar_cache (
                cache_key TEXT PRIMARY KEY,
                events_json TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                error_message TEXT
            );
            CREATE TABLE IF NOT EXISTS ticks (
                symbol TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                last_price REAL,
                volume INTEGER,
                open_interest INTEGER,
                bid_price REAL,
                ask_price REAL,
                PRIMARY KEY (symbol, timestamp)
            );
            CREATE INDEX IF NOT EXISTS idx_ticks_symbol_time
                ON ticks(symbol, timestamp DESC);
            CREATE TABLE IF NOT EXISTS app_preferences (
                id TEXT PRIMARY KEY DEFAULT 'default',
                json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS app_secrets (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS llm_credentials (
                provider TEXT PRIMARY KEY,
                api_key_encrypted TEXT NOT NULL,
                base_url TEXT NOT NULL,
                model TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sim_accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'CNY',
                initial_balance REAL NOT NULL,
                cash_balance REAL NOT NULL,
                equity REAL NOT NULL,
                margin_used REAL NOT NULL DEFAULT 0,
                realized_pnl REAL NOT NULL DEFAULT 0,
                unrealized_pnl REAL NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sim_contract_rules (
                symbol TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                exchange TEXT NOT NULL,
                contract_multiplier REAL NOT NULL,
                price_tick REAL NOT NULL,
                margin_rate_long REAL NOT NULL,
                margin_rate_short REAL NOT NULL,
                commission_mode TEXT NOT NULL,
                commission_open REAL NOT NULL,
                commission_close REAL NOT NULL,
                commission_close_today REAL NOT NULL,
                min_order_qty INTEGER NOT NULL DEFAULT 0,
                lot_size INTEGER NOT NULL DEFAULT 1,
                max_order_qty INTEGER NOT NULL DEFAULT 0,
                daily_price_limit_up REAL NOT NULL DEFAULT 0,
                daily_price_limit_down REAL NOT NULL DEFAULT 0,
                default_slippage_ticks REAL NOT NULL DEFAULT 0,
                is_custom INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sim_risk_rules (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                scope TEXT NOT NULL,
                symbol TEXT,
                rule_type TEXT NOT NULL,
                threshold REAL NOT NULL,
                action TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sim_risk_rules_account
                ON sim_risk_rules(account_id);
            CREATE TABLE IF NOT EXISTS sim_risk_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                rule_id TEXT NOT NULL,
                triggered_at TEXT NOT NULL,
                description TEXT NOT NULL,
                action_taken TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sim_risk_events_account
                ON sim_risk_events(account_id, triggered_at DESC);
            CREATE TABLE IF NOT EXISTS sim_orders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                side TEXT NOT NULL,
                offset TEXT NOT NULL,
                order_type TEXT NOT NULL,
                price REAL,
                trigger_price REAL,
                stop_loss_price REAL,
                take_profit_price REAL,
                oco_group_id TEXT,
                parent_order_id TEXT,
                tif TEXT,
                quantity INTEGER NOT NULL,
                filled_quantity INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                reason TEXT,
                source TEXT NOT NULL DEFAULT 'manual',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_sim_orders_account
                ON sim_orders(account_id, status, created_at DESC);
            CREATE TABLE IF NOT EXISTS sim_trades (
                id TEXT PRIMARY KEY,
                order_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                side TEXT NOT NULL,
                offset TEXT NOT NULL,
                price REAL NOT NULL,
                quantity INTEGER NOT NULL,
                commission REAL NOT NULL,
                slippage REAL NOT NULL DEFAULT 0,
                realized_pnl REAL NOT NULL DEFAULT 0,
                traded_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sim_trades_account
                ON sim_trades(account_id, traded_at DESC);
            CREATE TABLE IF NOT EXISTS sim_positions (
                account_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                position_side TEXT NOT NULL,
                today_qty INTEGER NOT NULL DEFAULT 0,
                history_qty INTEGER NOT NULL DEFAULT 0,
                total_qty INTEGER NOT NULL DEFAULT 0,
                avg_price REAL NOT NULL,
                margin REAL NOT NULL,
                unrealized_pnl REAL NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (account_id, symbol, position_side)
            );
            CREATE TABLE IF NOT EXISTS sim_equity_snapshots (
                account_id TEXT NOT NULL,
                snapshot_at TEXT NOT NULL,
                equity REAL NOT NULL,
                cash_balance REAL NOT NULL,
                margin_used REAL NOT NULL,
                realized_pnl REAL NOT NULL,
                unrealized_pnl REAL NOT NULL,
                risk_ratio REAL NOT NULL,
                PRIMARY KEY (account_id, snapshot_at)
            );
            CREATE INDEX IF NOT EXISTS idx_sim_equity_account
                ON sim_equity_snapshots(account_id, snapshot_at DESC);
            CREATE TABLE IF NOT EXISTS sim_journal_entries (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                symbol TEXT,
                trade_id TEXT,
                report_id TEXT,
                title TEXT NOT NULL,
                thesis TEXT,
                execution_review TEXT,
                emotion_tags TEXT,
                score INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sim_journal_account
                ON sim_journal_entries(account_id, created_at DESC);
            CREATE TABLE IF NOT EXISTS sim_replay_sessions (
                id TEXT PRIMARY KEY DEFAULT 'default',
                symbol TEXT NOT NULL,
                interval TEXT NOT NULL,
                replay_date TEXT NOT NULL,
                current_index INTEGER NOT NULL DEFAULT 0,
                speed INTEGER NOT NULL DEFAULT 1,
                running INTEGER NOT NULL DEFAULT 0,
                account_id TEXT,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS stock_symbols (
                ts_code TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                exchange TEXT NOT NULL,
                market TEXT,
                industry TEXT,
                list_date TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_stock_symbols_industry ON stock_symbols(industry);
            CREATE INDEX IF NOT EXISTS idx_stock_symbols_name ON stock_symbols(name);
            CREATE INDEX IF NOT EXISTS idx_stock_symbols_exchange ON stock_symbols(exchange);
            CREATE TABLE IF NOT EXISTS stock_daily_bars (
                ts_code TEXT NOT NULL,
                trade_date TEXT NOT NULL,
                open REAL,
                high REAL,
                low REAL,
                close REAL,
                pre_close REAL,
                pct_chg REAL,
                volume REAL,
                amount REAL,
                turnover_rate REAL,
                adj_factor REAL,
                adjustment TEXT NOT NULL DEFAULT 'none',
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (ts_code, trade_date, adjustment)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_daily_bars_date ON stock_daily_bars(trade_date);
            CREATE INDEX IF NOT EXISTS idx_stock_daily_bars_ts ON stock_daily_bars(ts_code, trade_date DESC);
            CREATE TABLE IF NOT EXISTS stock_index_daily_bars (
                index_code TEXT NOT NULL,
                trade_date TEXT NOT NULL,
                open REAL,
                high REAL,
                low REAL,
                close REAL,
                pct_chg REAL,
                volume REAL,
                amount REAL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (index_code, trade_date)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_index_daily_bars_date ON stock_index_daily_bars(trade_date);
            CREATE TABLE IF NOT EXISTS stock_boards (
                board_code TEXT PRIMARY KEY,
                board_name TEXT NOT NULL,
                board_type TEXT NOT NULL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_stock_boards_type ON stock_boards(board_type);
            CREATE TABLE IF NOT EXISTS stock_board_members (
                board_code TEXT NOT NULL,
                ts_code TEXT NOT NULL,
                weight REAL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (board_code, ts_code)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_board_members_ts ON stock_board_members(ts_code);
            CREATE TABLE IF NOT EXISTS stock_board_snapshots (
                board_code TEXT NOT NULL,
                trade_date TEXT NOT NULL,
                pct_chg REAL,
                amount REAL,
                turnover_rate REAL,
                net_flow REAL,
                up_count INTEGER,
                down_count INTEGER,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (board_code, trade_date)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_board_snapshots_date ON stock_board_snapshots(trade_date);
            CREATE TABLE IF NOT EXISTS stock_financial_metrics (
                ts_code TEXT NOT NULL,
                report_period TEXT NOT NULL,
                report_type TEXT,
                revenue REAL,
                revenue_yoy REAL,
                net_profit REAL,
                net_profit_yoy REAL,
                roe REAL,
                gross_margin REAL,
                debt_ratio REAL,
                operating_cash_flow REAL,
                eps REAL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (ts_code, report_period)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_financial_metrics_period ON stock_financial_metrics(ts_code, report_period DESC);
            CREATE TABLE IF NOT EXISTS stock_valuation_snapshots (
                ts_code TEXT NOT NULL,
                trade_date TEXT NOT NULL,
                pe_ttm REAL,
                pb REAL,
                ps_ttm REAL,
                dividend_yield REAL,
                market_cap REAL,
                float_market_cap REAL,
                pe_percentile REAL,
                pb_percentile REAL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (ts_code, trade_date)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_valuation_snapshots_date ON stock_valuation_snapshots(ts_code, trade_date DESC);
            CREATE TABLE IF NOT EXISTS stock_factor_snapshots (
                ts_code TEXT NOT NULL,
                factor_date TEXT NOT NULL,
                momentum REAL,
                quality REAL,
                valuation REAL,
                growth REAL,
                volatility REAL,
                liquidity REAL,
                capital_flow REAL,
                score REAL,
                factor_version TEXT NOT NULL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (ts_code, factor_date, factor_version)
            );
            CREATE INDEX IF NOT EXISTS idx_stock_factor_snapshots_date ON stock_factor_snapshots(ts_code, factor_date DESC);
            CREATE TABLE IF NOT EXISTS stock_screen_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                criteria_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS stock_screen_results (
                id TEXT PRIMARY KEY,
                template_id TEXT,
                name TEXT NOT NULL,
                criteria_json TEXT NOT NULL,
                result_json TEXT NOT NULL,
                trade_date TEXT,
                report_period TEXT,
                source_summary TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_stock_screen_results_created ON stock_screen_results(created_at DESC);
            CREATE TABLE IF NOT EXISTS stock_watchlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                symbols_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS watchlist_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS watchlist_items (
                id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                notes TEXT,
                alert_price REAL,
                alert_pct REAL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (group_id) REFERENCES watchlist_groups(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_watchlist_items_group
                ON watchlist_items(group_id, sort_order);
            CREATE INDEX IF NOT EXISTS idx_watchlist_items_symbol_type
                ON watchlist_items(symbol, asset_type);
            CREATE TABLE IF NOT EXISTS stock_paper_accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                initial_balance REAL NOT NULL,
                cash_balance REAL NOT NULL,
                market_value REAL NOT NULL DEFAULT 0,
                total_equity REAL NOT NULL,
                total_cost REAL NOT NULL DEFAULT 0,
                realized_pnl REAL NOT NULL DEFAULT 0,
                unrealized_pnl REAL NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS stock_paper_orders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                ts_code TEXT NOT NULL,
                name TEXT NOT NULL,
                side TEXT NOT NULL,
                order_type TEXT NOT NULL,
                price REAL,
                quantity INTEGER NOT NULL,
                filled_quantity INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_stock_paper_orders_account
                ON stock_paper_orders(account_id, status, created_at DESC);
            CREATE TABLE IF NOT EXISTS stock_paper_positions (
                account_id TEXT NOT NULL,
                ts_code TEXT NOT NULL,
                name TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 0,
                available_quantity INTEGER NOT NULL DEFAULT 0,
                avg_cost REAL NOT NULL DEFAULT 0,
                total_cost REAL NOT NULL DEFAULT 0,
                market_value REAL NOT NULL DEFAULT 0,
                unrealized_pnl REAL NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (account_id, ts_code)
            );
            CREATE TABLE IF NOT EXISTS stock_paper_trades (
                id TEXT PRIMARY KEY,
                order_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                ts_code TEXT NOT NULL,
                name TEXT NOT NULL,
                side TEXT NOT NULL,
                price REAL NOT NULL,
                quantity INTEGER NOT NULL,
                commission REAL NOT NULL DEFAULT 0,
                traded_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_stock_paper_trades_account
                ON stock_paper_trades(account_id, traded_at DESC);
        ",
        )?;
        for (code, label, desc) in dimensions::seed_rows() {
            conn.execute(
                "INSERT OR IGNORE INTO analysis_dimensions VALUES (?,?,?)",
                params![code, label, desc],
            )?;
        }
        let _ = conn.execute("ALTER TABLE reports ADD COLUMN dimension_summary TEXT", []);
        let _ = conn.execute("ALTER TABLE reports ADD COLUMN news_ids TEXT", []);
        let _ = conn.execute("ALTER TABLE reports ADD COLUMN anomaly_reason TEXT", []);
        // 模拟盘 schema 增量扩展（忽略已存在列的错误）
        let _ = conn.execute(
            "ALTER TABLE sim_contract_rules ADD COLUMN min_order_qty INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE sim_contract_rules ADD COLUMN lot_size INTEGER NOT NULL DEFAULT 1",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE sim_contract_rules ADD COLUMN max_order_qty INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute("ALTER TABLE sim_contract_rules ADD COLUMN daily_price_limit_up REAL NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE sim_contract_rules ADD COLUMN daily_price_limit_down REAL NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE sim_contract_rules ADD COLUMN default_slippage_ticks REAL NOT NULL DEFAULT 0", []);
        let _ = conn.execute(
            "ALTER TABLE sim_contract_rules ADD COLUMN is_custom INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute("ALTER TABLE sim_orders ADD COLUMN stop_loss_price REAL", []);
        let _ = conn.execute(
            "ALTER TABLE sim_orders ADD COLUMN take_profit_price REAL",
            [],
        );
        let _ = conn.execute("ALTER TABLE sim_orders ADD COLUMN oco_group_id TEXT", []);
        let _ = conn.execute("ALTER TABLE sim_orders ADD COLUMN parent_order_id TEXT", []);
        let _ = conn.execute("ALTER TABLE sim_orders ADD COLUMN tif TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE sim_orders ADD COLUMN condition_operator TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE sim_orders ADD COLUMN trailing_distance_ticks REAL",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE sim_orders ADD COLUMN trailing_reference_price REAL",
            [],
        );
        // 统一自选：初始化默认分组并迁移旧 stock_watchlists
        let _ = Self::init_watchlist_defaults(&conn);
        Ok(())
    }

    fn init_watchlist_defaults(conn: &Connection) -> AppResult<()> {
        let now = dt_to_iso(Utc::now());
        let defaults = vec![
            ("wl-all", "全部", 0),
            ("wl-futures", "期货", 1),
            ("wl-stocks", "A股", 2),
            ("wl-focus", "重点观察", 3),
        ];
        for (id, name, order) in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO watchlist_groups (id, name, sort_order, created_at, updated_at) VALUES (?,?,?,?,?)",
                params![id, name, order, now, now],
            )?;
        }
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM watchlist_items", [], |row| row.get(0))?;
        if count == 0 {
            let mut stmt = conn.prepare(
                "SELECT id, name, symbols_json, created_at, updated_at FROM stock_watchlists",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?;
            for row in rows {
                let (id, name, symbols_json, created_at, updated_at) = row?;
                let group_id = format!("wl-migrate-{}", id);
                conn.execute(
                    "INSERT OR IGNORE INTO watchlist_groups (id, name, sort_order, created_at, updated_at) VALUES (?,?,?,?,?)",
                    params![group_id, name, 100, created_at, updated_at],
                )?;
                let symbols: Vec<String> = serde_json::from_str(&symbols_json).unwrap_or_default();
                for (idx, symbol) in symbols.iter().enumerate() {
                    let item_id = uuid::Uuid::new_v4().to_string();
                    conn.execute(
                        "INSERT INTO watchlist_items (id, group_id, asset_type, symbol, name, sort_order, created_at, updated_at) VALUES (?,?,?,?,?,?,?,?)",
                        params![item_id, group_id, "stock", symbol, symbol, idx as i64, created_at, updated_at],
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn save_klines(&self, klines: &[KLine]) -> AppResult<usize> {
        if klines.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare("INSERT OR REPLACE INTO klines VALUES (?,?,?,?,?,?,?,?,?)")?;
        for k in klines {
            stmt.execute(params![
                k.symbol,
                k.interval,
                k.start_time,
                k.open,
                k.high,
                k.low,
                k.close,
                k.volume,
                k.turnover,
            ])?;
        }
        Ok(klines.len())
    }

    pub fn save_tick(&self, tick: &Tick) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO ticks VALUES (?,?,?,?,?,?,?)",
            params![
                tick.symbol,
                tick.timestamp,
                tick.last_price,
                tick.volume,
                tick.open_interest,
                tick.bid_price,
                tick.ask_price,
            ],
        )?;
        Ok(())
    }

    pub fn purge_old_klines(&self, keep_days: i64) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(keep_days)).to_rfc3339();
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let n = conn.execute("DELETE FROM klines WHERE start_time < ?", params![cutoff])?;
        Ok(n)
    }

    pub fn purge_old_ticks(&self, keep_days: i64) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(keep_days)).to_rfc3339();
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let n = conn.execute("DELETE FROM ticks WHERE timestamp < ?", params![cutoff])?;
        Ok(n)
    }

    pub fn get_news_by_id(&self, id: &str) -> AppResult<Option<NewsRecord>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, category_id, title, summary, url, display_time, content_hash, ingested_at
             FROM news_items WHERE id=? LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(NewsRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                category_id: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                url: row.get(5)?,
                display_time: row.get(6)?,
                content_hash: row.get(7)?,
                ingested_at: row.get(8)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete_classifications_for_news(&self, news_ids: &[String]) -> AppResult<usize> {
        if news_ids.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut count = 0usize;
        for id in news_ids {
            count += conn.execute(
                "DELETE FROM news_classifications WHERE news_id=?",
                params![id],
            )?;
        }
        Ok(count)
    }

    pub fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: i64,
    ) -> AppResult<Vec<KLine>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT symbol, interval, start_time, open, high, low, close, volume, turnover
             FROM klines
             WHERE symbol=? AND interval=? AND start_time BETWEEN ? AND ?
             ORDER BY start_time ASC LIMIT ?",
        )?;
        let rows = stmt.query_map(
            params![symbol, interval, dt_to_iso(start), dt_to_iso(end), limit],
            |row| {
                Ok(KLine {
                    symbol: row.get(0)?,
                    interval: row.get(1)?,
                    start_time: row.get(2)?,
                    open: row.get(3)?,
                    high: row.get(4)?,
                    low: row.get(5)?,
                    close: row.get(6)?,
                    volume: row.get(7)?,
                    turnover: row.get(8)?,
                })
            },
        )?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_report(&self, report: &AnalysisReport) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let tags = serde_json::to_string(&report.tags)?;
        let dimension_summary = report
            .dimension_summary
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let news_ids = serde_json::to_string(&report.news_ids)?;
        conn.execute(
            "INSERT OR REPLACE INTO reports (id,symbol,trigger,provider,prompt_version,context_summary,content,created_at,tags,dimension_summary,news_ids,anomaly_reason) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                report.id,
                report.symbol,
                report.trigger,
                report.provider,
                report.prompt_version,
                report.context_summary,
                report.content,
                report.created_at,
                tags,
                dimension_summary,
                news_ids,
                report.anomaly_reason,
            ],
        )?;
        Ok(())
    }

    pub fn get_reports(
        &self,
        symbol: Option<&str>,
        trigger: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<AnalysisReport>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut query = String::from(
            "SELECT id,symbol,trigger,provider,prompt_version,context_summary,content,created_at,tags,dimension_summary,news_ids,anomaly_reason FROM reports",
        );
        let mut conditions = Vec::new();
        let mut bind: Vec<String> = Vec::new();
        if let Some(s) = symbol {
            conditions.push("symbol=?");
            bind.push(s.to_string());
        }
        if let Some(t) = trigger {
            conditions.push("trigger=?");
            bind.push(t.to_string());
        }
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        query.push_str(" ORDER BY created_at DESC LIMIT ?");

        let mut stmt = conn.prepare(&query)?;
        let rows = match bind.len() {
            0 => stmt.query_map(params![limit], row_to_report)?,
            1 => stmt.query_map(params![bind[0].as_str(), limit], row_to_report)?,
            2 => stmt.query_map(
                params![bind[0].as_str(), bind[1].as_str(), limit],
                row_to_report,
            )?,
            _ => return Ok(vec![]),
        };
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_report(&self, id: &str) -> AppResult<Option<AnalysisReport>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id,symbol,trigger,provider,prompt_version,context_summary,content,created_at,tags,dimension_summary,news_ids,anomaly_reason FROM reports WHERE id=?",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_report)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_contracts(&self, contracts: &[Contract]) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let now = dt_to_iso(Utc::now());
        for c in contracts {
            conn.execute(
                "INSERT OR REPLACE INTO contracts VALUES (?,?,?,?,?,?,?,?,?)",
                params![
                    c.symbol,
                    c.exchange,
                    c.name,
                    c.product,
                    c.multiplier,
                    c.margin_ratio,
                    c.listing_date,
                    c.expiry_date,
                    now,
                ],
            )?;
        }
        Ok(())
    }

    pub fn get_contracts(&self, exchange: Option<&str>) -> AppResult<Vec<Contract>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        if let Some(ex) = exchange {
            let mut stmt = conn.prepare(
                "SELECT symbol,exchange,name,product,multiplier,margin_ratio,listing_date,expiry_date FROM contracts WHERE exchange=?",
            )?;
            let rows = stmt.query_map(params![ex], row_to_contract)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        } else {
            let mut stmt = conn.prepare(
                "SELECT symbol,exchange,name,product,multiplier,margin_ratio,listing_date,expiry_date FROM contracts",
            )?;
            let rows = stmt.query_map([], row_to_contract)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
    }

    pub fn save_liquidity_snapshot(&self, snap: &LiquiditySnapshot) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO liquidity_snapshots VALUES (?,?,?,?,?,?)",
            params![
                snap.symbol.to_uppercase(),
                snap.scored_at,
                snap.volume_20d,
                snap.turnover_20d,
                snap.score,
                snap.tier,
            ],
        )?;
        Ok(())
    }

    pub fn get_latest_liquidity_map(
        &self,
    ) -> AppResult<std::collections::HashMap<String, LiquiditySnapshot>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT l.symbol, l.volume_20d, l.turnover_20d, l.score, l.tier, l.scored_at
             FROM liquidity_snapshots l
             INNER JOIN (
               SELECT symbol, MAX(scored_at) AS max_at
               FROM liquidity_snapshots GROUP BY symbol
             ) latest ON l.symbol = latest.symbol AND l.scored_at = latest.max_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(LiquiditySnapshot {
                symbol: row.get(0)?,
                volume_20d: row.get(1)?,
                turnover_20d: row.get(2)?,
                score: row.get(3)?,
                tier: row.get(4)?,
                scored_at: row.get(5)?,
            })
        })?;
        let mut map = std::collections::HashMap::new();
        for row in rows.flatten() {
            map.insert(row.symbol.to_uppercase(), row);
        }
        Ok(map)
    }

    pub fn news_hash_exists(&self, hash: &str) -> AppResult<bool> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT 1 FROM news_items WHERE content_hash=? LIMIT 1")?;
        let mut rows = stmt.query_map(params![hash], |_| Ok(()))?;
        Ok(rows.next().transpose()?.is_some())
    }

    pub fn save_news(&self, item: &NewsRecord) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO news_items VALUES (?,?,?,?,?,?,?,?,?)",
            params![
                item.id,
                item.source,
                item.category_id,
                item.title,
                item.summary,
                item.url,
                item.display_time,
                item.content_hash,
                item.ingested_at,
            ],
        )?;
        Ok(())
    }

    pub fn save_classifications(&self, labels: &[NewsClassification]) -> AppResult<usize> {
        if labels.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut count = 0usize;
        for label in labels {
            let n = conn.execute(
                "INSERT INTO news_classifications
                 (news_id, symbol, dimension_code, confidence, method, created_at)
                 VALUES (?,?,?,?,?,?)
                 ON CONFLICT(news_id, symbol, dimension_code) DO UPDATE SET
                   confidence = CASE
                     WHEN excluded.confidence > confidence THEN excluded.confidence
                     ELSE confidence
                   END,
                   method = CASE
                     WHEN excluded.confidence > confidence THEN excluded.method
                     WHEN excluded.confidence = confidence AND excluded.method = 'llm' THEN excluded.method
                     ELSE method
                   END",
                params![
                    label.news_id,
                    label.symbol.to_uppercase(),
                    label.dimension_code,
                    label.confidence,
                    label.method,
                    label.created_at,
                ],
            )?;
            count += n;
        }
        Ok(count)
    }

    pub fn get_news_for_symbol(
        &self,
        symbol: &str,
        dimension: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<NewsItemView>> {
        let sym = symbol.to_uppercase();
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let (query, bind_dim) = if let Some(dim) = dimension {
            (
                "SELECT DISTINCT n.id, n.source, n.category_id, n.title, n.summary, n.url,
                        n.display_time, c.symbol, c.dimension_code, c.confidence, c.method
                 FROM news_items n
                 INNER JOIN news_classifications c ON c.news_id = n.id
                 WHERE c.symbol=? AND c.dimension_code=?
                 ORDER BY n.display_time DESC LIMIT ?",
                Some(dim.to_string()),
            )
        } else {
            (
                "SELECT DISTINCT n.id, n.source, n.category_id, n.title, n.summary, n.url,
                        n.display_time, c.symbol, c.dimension_code, c.confidence, c.method
                 FROM news_items n
                 INNER JOIN news_classifications c ON c.news_id = n.id
                 WHERE c.symbol=?
                 ORDER BY n.display_time DESC LIMIT ?",
                None,
            )
        };

        let mut stmt = conn.prepare(query)?;
        let rows: Vec<(NewsItemView, NewsClassificationView)> = if let Some(dim) = bind_dim {
            stmt.query_map(params![sym, dim, limit], row_to_news_joined)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map(params![sym, limit], row_to_news_joined)?
                .filter_map(|r| r.ok())
                .collect()
        };

        Ok(merge_news_rows(rows))
    }

    pub fn get_latest_news(&self, limit: i64) -> AppResult<Vec<NewsItemView>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT n.id, n.source, n.category_id, n.title, n.summary, n.url, n.display_time,
                    c.symbol, c.dimension_code, c.confidence, c.method
             FROM news_items n
             LEFT JOIN news_classifications c ON c.news_id = n.id
             ORDER BY n.display_time DESC LIMIT ?",
        )?;
        let rows: Vec<(NewsItemView, NewsClassificationView)> = stmt
            .query_map(params![limit * 3], row_to_news_joined)?
            .filter_map(|r| r.ok())
            .collect();
        let mut merged = merge_news_rows(rows);
        merged.truncate(limit as usize);
        Ok(merged)
    }

    pub fn get_unclassified_news(&self, limit: i64) -> AppResult<Vec<NewsRecord>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, category_id, title, summary, url, display_time, content_hash, ingested_at
             FROM news_items n
             WHERE NOT EXISTS (SELECT 1 FROM news_classifications c WHERE c.news_id = n.id)
             ORDER BY n.display_time DESC LIMIT ?",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(NewsRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                category_id: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                url: row.get(5)?,
                display_time: row.get(6)?,
                content_hash: row.get(7)?,
                ingested_at: row.get(8)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn replace_report_facts(&self, report_id: &str, facts: &[DimensionFact]) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "DELETE FROM dimension_facts WHERE source_report_id=?",
            params![report_id],
        )?;
        for f in facts {
            conn.execute(
                "INSERT OR REPLACE INTO dimension_facts
                 (id, symbol, dimension_code, fact, source_news_id, source_report_id, valid_until, created_at)
                 VALUES (?,?,?,?,?,?,?,?)",
                params![
                    f.id,
                    f.symbol,
                    f.dimension_code,
                    f.fact,
                    f.source_news_id,
                    f.source_report_id,
                    f.valid_until,
                    f.created_at,
                ],
            )?;
        }
        Ok(())
    }

    pub fn get_dimension_facts(&self, symbol: &str, limit: i64) -> AppResult<Vec<DimensionFact>> {
        let sym = symbol.to_uppercase();
        let now = dt_to_iso(Utc::now());
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, symbol, dimension_code, fact, source_news_id, source_report_id, valid_until, created_at
             FROM dimension_facts
             WHERE symbol=? AND (valid_until IS NULL OR valid_until >= ?)
             ORDER BY created_at DESC LIMIT ?",
        )?;
        let rows = stmt.query_map(params![sym, now, limit], row_to_dimension_fact)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_calendar_cache(
        &self,
        cache_key: &str,
        events: &[CalendarEvent],
        error_message: Option<&str>,
    ) -> AppResult<()> {
        let json = serde_json::to_string(events).map_err(|e| AppError::Msg(e.to_string()))?;
        let now = dt_to_iso(Utc::now());
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT INTO calendar_cache (cache_key, events_json, fetched_at, error_message)
             VALUES (?,?,?,?)
             ON CONFLICT(cache_key) DO UPDATE SET
               events_json=excluded.events_json,
               fetched_at=excluded.fetched_at,
               error_message=excluded.error_message",
            params![cache_key, json, now, error_message],
        )?;
        Ok(())
    }

    pub fn load_calendar_cache(&self, cache_key: &str) -> AppResult<Option<Vec<CalendarEvent>>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT events_json FROM calendar_cache WHERE cache_key=? ORDER BY fetched_at DESC LIMIT 1",
        )?;
        let json: Option<String> = stmt.query_row(params![cache_key], |row| row.get(0)).ok();
        match json {
            Some(raw) => {
                let events: Vec<CalendarEvent> =
                    serde_json::from_str(&raw).map_err(|e| AppError::Msg(e.to_string()))?;
                Ok(Some(events))
            }
            None => Ok(None),
        }
    }

    pub fn get_news_by_ids(&self, ids: &[String]) -> AppResult<Vec<NewsItemView>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let placeholders: Vec<String> = ids.iter().map(|_| "?".into()).collect();
        let query = format!(
            "SELECT n.id, n.source, n.category_id, n.title, n.summary, n.url, n.display_time,
                    c.symbol, c.dimension_code, c.confidence, c.method
             FROM news_items n
             LEFT JOIN news_classifications c ON c.news_id = n.id
             WHERE n.id IN ({})
             ORDER BY n.display_time DESC",
            placeholders.join(",")
        );
        let mut stmt = conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let rows: Vec<(NewsItemView, NewsClassificationView)> = stmt
            .query_map(params.as_slice(), row_to_news_joined)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(merge_news_rows(rows))
    }

    pub fn save_followup(&self, msg: &FollowupMessage) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO followup_messages
             (id, report_id, symbol, question, answer, provider, created_at)
             VALUES (?,?,?,?,?,?,?)",
            params![
                msg.id,
                msg.report_id,
                msg.symbol,
                msg.question,
                msg.answer,
                msg.provider,
                msg.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_followups(
        &self,
        report_id: Option<&str>,
        symbol: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<FollowupMessage>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut query = String::from(
            "SELECT id, report_id, symbol, question, answer, provider, created_at
             FROM followup_messages",
        );
        let mut conditions = Vec::new();
        let mut bind: Vec<String> = Vec::new();
        if let Some(rid) = report_id {
            conditions.push("report_id=?");
            bind.push(rid.to_string());
        }
        if let Some(sym) = symbol {
            conditions.push("symbol=?");
            bind.push(sym.to_uppercase());
        }
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        query.push_str(" ORDER BY created_at ASC LIMIT ?");

        let mut stmt = conn.prepare(&query)?;
        let rows = match bind.len() {
            0 => stmt.query_map(params![limit], row_to_followup)?,
            1 => stmt.query_map(params![bind[0].as_str(), limit], row_to_followup)?,
            2 => stmt.query_map(
                params![bind[0].as_str(), bind[1].as_str(), limit],
                row_to_followup,
            )?,
            _ => return Ok(vec![]),
        };
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 旧版 SQLite 存储，仅用于首次迁移至 JSON。
    pub fn load_user_preferences_legacy(&self) -> AppResult<Option<UserPreferences>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT json FROM app_preferences WHERE id = 'default'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            let prefs: UserPreferences = serde_json::from_str(&json)
                .map_err(|e| AppError::Msg(format!("invalid preferences json: {e}")))?;
            Ok(Some(prefs))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_create_app_secret(&self, key: &str) -> AppResult<String> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT value FROM app_secrets WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            return Ok(row.get(0)?);
        }
        let value = format!("{}{}", uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
        conn.execute(
            "INSERT INTO app_secrets (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, dt_to_iso(Utc::now())],
        )?;
        Ok(value)
    }

    pub fn list_llm_credentials(&self) -> AppResult<Vec<(String, String, String, String)>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT provider, api_key_encrypted, base_url, model FROM llm_credentials ORDER BY provider",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upsert_llm_credential(
        &self,
        provider: &str,
        api_key_encrypted: &str,
        base_url: &str,
        model: &str,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO llm_credentials (provider, api_key_encrypted, base_url, model, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                provider,
                api_key_encrypted,
                base_url,
                model,
                dt_to_iso(Utc::now())
            ],
        )?;
        Ok(())
    }

    pub fn delete_llm_credential(&self, provider: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "DELETE FROM llm_credentials WHERE provider = ?1",
            params![provider],
        )?;
        Ok(())
    }

    pub fn save_sim_account(&self, account: &SimAccount) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_accounts
             (id, name, currency, initial_balance, cash_balance, equity, margin_used, realized_pnl, unrealized_pnl, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                account.id,
                account.name,
                account.currency,
                account.initial_balance,
                account.cash_balance,
                account.equity,
                account.margin_used,
                account.realized_pnl,
                account.unrealized_pnl,
                account.status,
                account.created_at,
                account.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_accounts(&self) -> AppResult<Vec<SimAccount>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, currency, initial_balance, cash_balance, equity, margin_used, realized_pnl, unrealized_pnl, status, created_at, updated_at
             FROM sim_accounts ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SimAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                currency: row.get(2)?,
                initial_balance: row.get(3)?,
                cash_balance: row.get(4)?,
                equity: row.get(5)?,
                margin_used: row.get(6)?,
                realized_pnl: row.get(7)?,
                unrealized_pnl: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_sim_account(&self, id: &str) -> AppResult<Option<SimAccount>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, currency, initial_balance, cash_balance, equity, margin_used, realized_pnl, unrealized_pnl, status, created_at, updated_at
             FROM sim_accounts WHERE id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(SimAccount {
                id: row.get(0)?,
                name: row.get(1)?,
                currency: row.get(2)?,
                initial_balance: row.get(3)?,
                cash_balance: row.get(4)?,
                equity: row.get(5)?,
                margin_used: row.get(6)?,
                realized_pnl: row.get(7)?,
                unrealized_pnl: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_sim_contract_rule(&self, rule: &SimContractRule) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_contract_rules
             (symbol, name, exchange, contract_multiplier, price_tick, margin_rate_long, margin_rate_short, commission_mode, commission_open, commission_close, commission_close_today, min_order_qty, lot_size, max_order_qty, daily_price_limit_up, daily_price_limit_down, default_slippage_ticks, is_custom, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                rule.symbol,
                rule.name,
                rule.exchange,
                rule.contract_multiplier,
                rule.price_tick,
                rule.margin_rate_long,
                rule.margin_rate_short,
                rule.commission_mode,
                rule.commission_open,
                rule.commission_close,
                rule.commission_close_today,
                rule.min_order_qty,
                rule.lot_size,
                rule.max_order_qty,
                rule.daily_price_limit_up,
                rule.daily_price_limit_down,
                rule.default_slippage_ticks,
                rule.is_custom,
                rule.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete_sim_contract_rule(&self, symbol: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "DELETE FROM sim_contract_rules WHERE symbol = ?1",
            params![symbol],
        )?;
        Ok(())
    }

    pub fn list_sim_contract_rules(&self) -> AppResult<Vec<SimContractRule>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT symbol, name, exchange, contract_multiplier, price_tick, margin_rate_long, margin_rate_short, commission_mode, commission_open, commission_close, commission_close_today, min_order_qty, lot_size, max_order_qty, daily_price_limit_up, daily_price_limit_down, default_slippage_ticks, is_custom, updated_at
             FROM sim_contract_rules ORDER BY symbol",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SimContractRule {
                symbol: row.get(0)?,
                name: row.get(1)?,
                exchange: row.get(2)?,
                contract_multiplier: row.get(3)?,
                price_tick: row.get(4)?,
                margin_rate_long: row.get(5)?,
                margin_rate_short: row.get(6)?,
                commission_mode: row.get(7)?,
                commission_open: row.get(8)?,
                commission_close: row.get(9)?,
                commission_close_today: row.get(10)?,
                min_order_qty: row.get(11)?,
                lot_size: row.get(12)?,
                max_order_qty: row.get(13)?,
                daily_price_limit_up: row.get(14)?,
                daily_price_limit_down: row.get(15)?,
                default_slippage_ticks: row.get(16)?,
                is_custom: row.get::<_, i64>(17)? != 0,
                updated_at: row.get(18)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_sim_contract_rule(&self, symbol: &str) -> AppResult<Option<SimContractRule>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT symbol, name, exchange, contract_multiplier, price_tick, margin_rate_long, margin_rate_short, commission_mode, commission_open, commission_close, commission_close_today, min_order_qty, lot_size, max_order_qty, daily_price_limit_up, daily_price_limit_down, default_slippage_ticks, is_custom, updated_at
             FROM sim_contract_rules WHERE symbol = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![symbol], |row| {
            Ok(SimContractRule {
                symbol: row.get(0)?,
                name: row.get(1)?,
                exchange: row.get(2)?,
                contract_multiplier: row.get(3)?,
                price_tick: row.get(4)?,
                margin_rate_long: row.get(5)?,
                margin_rate_short: row.get(6)?,
                commission_mode: row.get(7)?,
                commission_open: row.get(8)?,
                commission_close: row.get(9)?,
                commission_close_today: row.get(10)?,
                min_order_qty: row.get(11)?,
                lot_size: row.get(12)?,
                max_order_qty: row.get(13)?,
                daily_price_limit_up: row.get(14)?,
                daily_price_limit_down: row.get(15)?,
                default_slippage_ticks: row.get(16)?,
                is_custom: row.get::<_, i64>(17)? != 0,
                updated_at: row.get(18)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_sim_order(&self, order: &SimOrder) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_orders
             (id, account_id, symbol, name, side, offset, order_type, price, trigger_price, stop_loss_price, take_profit_price, oco_group_id, parent_order_id, tif, condition_operator, trailing_distance_ticks, trailing_reference_price, quantity, filled_quantity, status, reason, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
            params![
                order.id,
                order.account_id,
                order.symbol,
                order.name,
                order.side,
                order.offset,
                order.order_type,
                order.price,
                order.trigger_price,
                order.stop_loss_price,
                order.take_profit_price,
                order.oco_group_id,
                order.parent_order_id,
                order.tif,
                order.condition_operator,
                order.trailing_distance_ticks,
                order.trailing_reference_price,
                order.quantity,
                order.filled_quantity,
                order.status,
                order.reason,
                order.source,
                order.created_at,
                order.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_orders(
        &self,
        account_id: Option<&str>,
        status: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<SimOrder>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let sql = if account_id.is_some() && status.is_some() {
            "SELECT id, account_id, symbol, name, side, offset, order_type, price, trigger_price, stop_loss_price, take_profit_price, oco_group_id, parent_order_id, tif, condition_operator, trailing_distance_ticks, trailing_reference_price, quantity, filled_quantity, status, reason, source, created_at, updated_at
             FROM sim_orders WHERE account_id = ?1 AND status = ?2 ORDER BY created_at DESC LIMIT ?3"
        } else if account_id.is_some() {
            "SELECT id, account_id, symbol, name, side, offset, order_type, price, trigger_price, stop_loss_price, take_profit_price, oco_group_id, parent_order_id, tif, condition_operator, trailing_distance_ticks, trailing_reference_price, quantity, filled_quantity, status, reason, source, created_at, updated_at
             FROM sim_orders WHERE account_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        } else if status.is_some() {
            "SELECT id, account_id, symbol, name, side, offset, order_type, price, trigger_price, stop_loss_price, take_profit_price, oco_group_id, parent_order_id, tif, condition_operator, trailing_distance_ticks, trailing_reference_price, quantity, filled_quantity, status, reason, source, created_at, updated_at
             FROM sim_orders WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2"
        } else {
            "SELECT id, account_id, symbol, name, side, offset, order_type, price, trigger_price, stop_loss_price, take_profit_price, oco_group_id, parent_order_id, tif, condition_operator, trailing_distance_ticks, trailing_reference_price, quantity, filled_quantity, status, reason, source, created_at, updated_at
             FROM sim_orders ORDER BY created_at DESC LIMIT ?1"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = match (account_id, status) {
            (Some(a), Some(s)) => stmt.query_map(params![a, s, limit], row_to_sim_order)?,
            (Some(a), None) => stmt.query_map(params![a, limit], row_to_sim_order)?,
            (None, Some(s)) => stmt.query_map(params![s, limit], row_to_sim_order)?,
            (None, None) => stmt.query_map(params![limit], row_to_sim_order)?,
        };
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_sim_order(&self, id: &str) -> AppResult<Option<SimOrder>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, symbol, name, side, offset, order_type, price, trigger_price, stop_loss_price, take_profit_price, oco_group_id, parent_order_id, tif, condition_operator, trailing_distance_ticks, trailing_reference_price, quantity, filled_quantity, status, reason, source, created_at, updated_at
             FROM sim_orders WHERE id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_sim_order)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_sim_trade(&self, trade: &SimTrade) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_trades
             (id, order_id, account_id, symbol, name, side, offset, price, quantity, commission, slippage, realized_pnl, traded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                trade.id,
                trade.order_id,
                trade.account_id,
                trade.symbol,
                trade.name,
                trade.side,
                trade.offset,
                trade.price,
                trade.quantity,
                trade.commission,
                trade.slippage,
                trade.realized_pnl,
                trade.traded_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_trades(
        &self,
        account_id: Option<&str>,
        symbol: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<SimTrade>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match (account_id, symbol) {
            (Some(a), Some(s)) => (
                "SELECT id, order_id, account_id, symbol, name, side, offset, price, quantity, commission, slippage, realized_pnl, traded_at
                 FROM sim_trades WHERE account_id = ?1 AND symbol = ?2 ORDER BY traded_at DESC LIMIT ?3",
                vec![Box::new(a.to_string()), Box::new(s.to_string()), Box::new(limit)],
            ),
            (Some(a), None) => (
                "SELECT id, order_id, account_id, symbol, name, side, offset, price, quantity, commission, slippage, realized_pnl, traded_at
                 FROM sim_trades WHERE account_id = ?1 ORDER BY traded_at DESC LIMIT ?2",
                vec![Box::new(a.to_string()), Box::new(limit)],
            ),
            (None, Some(s)) => (
                "SELECT id, order_id, account_id, symbol, name, side, offset, price, quantity, commission, slippage, realized_pnl, traded_at
                 FROM sim_trades WHERE symbol = ?1 ORDER BY traded_at DESC LIMIT ?2",
                vec![Box::new(s.to_string()), Box::new(limit)],
            ),
            (None, None) => (
                "SELECT id, order_id, account_id, symbol, name, side, offset, price, quantity, commission, slippage, realized_pnl, traded_at
                 FROM sim_trades ORDER BY traded_at DESC LIMIT ?1",
                vec![Box::new(limit)],
            ),
        };
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(&*param_refs, row_to_sim_trade)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_sim_position(&self, position: &SimPosition) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_positions
             (account_id, symbol, name, position_side, today_qty, history_qty, total_qty, avg_price, margin, unrealized_pnl, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                position.account_id,
                position.symbol,
                position.name,
                position.position_side,
                position.today_qty,
                position.history_qty,
                position.total_qty,
                position.avg_price,
                position.margin,
                position.unrealized_pnl,
                position.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete_sim_position(
        &self,
        account_id: &str,
        symbol: &str,
        position_side: &str,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "DELETE FROM sim_positions WHERE account_id = ?1 AND symbol = ?2 AND position_side = ?3",
            params![account_id, symbol, position_side],
        )?;
        Ok(())
    }

    pub fn save_sim_risk_rule(&self, rule: &SimRiskRule) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_risk_rules
             (id, account_id, scope, symbol, rule_type, threshold, action, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                rule.id,
                rule.account_id,
                rule.scope,
                rule.symbol,
                rule.rule_type,
                rule.threshold,
                rule.action,
                rule.enabled,
                rule.created_at,
                rule.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_risk_rules(&self, account_id: Option<&str>) -> AppResult<Vec<SimRiskRule>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let sql = if account_id.is_some() {
            "SELECT id, account_id, scope, symbol, rule_type, threshold, action, enabled, created_at, updated_at
             FROM sim_risk_rules WHERE account_id = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, account_id, scope, symbol, rule_type, threshold, action, enabled, created_at, updated_at
             FROM sim_risk_rules ORDER BY created_at DESC"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = if let Some(a) = account_id {
            stmt.query_map(params![a], row_to_sim_risk_rule)?
        } else {
            stmt.query_map([], row_to_sim_risk_rule)?
        };
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_sim_risk_rule(&self, id: &str) -> AppResult<Option<SimRiskRule>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, scope, symbol, rule_type, threshold, action, enabled, created_at, updated_at
             FROM sim_risk_rules WHERE id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_sim_risk_rule)?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete_sim_risk_rule(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute("DELETE FROM sim_risk_rules WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn save_sim_risk_event(&self, event: &SimRiskEvent) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_risk_events
             (id, account_id, rule_id, triggered_at, description, action_taken)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.id,
                event.account_id,
                event.rule_id,
                event.triggered_at,
                event.description,
                event.action_taken,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_risk_events(
        &self,
        account_id: &str,
        limit: i64,
    ) -> AppResult<Vec<SimRiskEvent>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, rule_id, triggered_at, description, action_taken
             FROM sim_risk_events WHERE account_id = ?1 ORDER BY triggered_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![account_id, limit], row_to_sim_risk_event)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn list_sim_positions(&self, account_id: Option<&str>) -> AppResult<Vec<SimPosition>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(a) = account_id {
            (
                "SELECT account_id, symbol, name, position_side, today_qty, history_qty, total_qty, avg_price, margin, unrealized_pnl, updated_at
                 FROM sim_positions WHERE account_id = ?1 ORDER BY symbol",
                vec![Box::new(a.to_string())],
            )
        } else {
            (
                "SELECT account_id, symbol, name, position_side, today_qty, history_qty, total_qty, avg_price, margin, unrealized_pnl, updated_at
                 FROM sim_positions ORDER BY account_id, symbol",
                vec![],
            )
        };
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(&*param_refs, row_to_sim_position)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_sim_equity_snapshot(&self, snapshot: &SimEquitySnapshot) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_equity_snapshots
             (account_id, snapshot_at, equity, cash_balance, margin_used, realized_pnl, unrealized_pnl, risk_ratio)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot.account_id,
                snapshot.snapshot_at,
                snapshot.equity,
                snapshot.cash_balance,
                snapshot.margin_used,
                snapshot.realized_pnl,
                snapshot.unrealized_pnl,
                snapshot.risk_ratio,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_equity_snapshots(
        &self,
        account_id: &str,
        limit: i64,
    ) -> AppResult<Vec<SimEquitySnapshot>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT account_id, snapshot_at, equity, cash_balance, margin_used, realized_pnl, unrealized_pnl, risk_ratio
             FROM sim_equity_snapshots WHERE account_id = ?1 ORDER BY snapshot_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![account_id, limit], |row| {
            Ok(SimEquitySnapshot {
                account_id: row.get(0)?,
                snapshot_at: row.get(1)?,
                equity: row.get(2)?,
                cash_balance: row.get(3)?,
                margin_used: row.get(4)?,
                realized_pnl: row.get(5)?,
                unrealized_pnl: row.get(6)?,
                risk_ratio: row.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_sim_journal_entry(&self, entry: &SimJournalEntry) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_journal_entries
             (id, account_id, symbol, trade_id, report_id, title, thesis, execution_review, emotion_tags, score, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry.id,
                entry.account_id,
                entry.symbol,
                entry.trade_id,
                entry.report_id,
                entry.title,
                entry.thesis,
                entry.execution_review,
                entry.emotion_tags,
                entry.score,
                entry.created_at,
                entry.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_sim_journal_entries(
        &self,
        account_id: Option<&str>,
        symbol: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<SimJournalEntry>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match (account_id, symbol) {
            (Some(a), Some(s)) => (
                "SELECT id, account_id, symbol, trade_id, report_id, title, thesis, execution_review, emotion_tags, score, created_at, updated_at
                 FROM sim_journal_entries WHERE account_id = ?1 AND symbol = ?2 ORDER BY created_at DESC LIMIT ?3",
                vec![Box::new(a.to_string()), Box::new(s.to_string()), Box::new(limit)],
            ),
            (Some(a), None) => (
                "SELECT id, account_id, symbol, trade_id, report_id, title, thesis, execution_review, emotion_tags, score, created_at, updated_at
                 FROM sim_journal_entries WHERE account_id = ?1 ORDER BY created_at DESC LIMIT ?2",
                vec![Box::new(a.to_string()), Box::new(limit)],
            ),
            (None, Some(s)) => (
                "SELECT id, account_id, symbol, trade_id, report_id, title, thesis, execution_review, emotion_tags, score, created_at, updated_at
                 FROM sim_journal_entries WHERE symbol = ?1 ORDER BY created_at DESC LIMIT ?2",
                vec![Box::new(s.to_string()), Box::new(limit)],
            ),
            (None, None) => (
                "SELECT id, account_id, symbol, trade_id, report_id, title, thesis, execution_review, emotion_tags, score, created_at, updated_at
                 FROM sim_journal_entries ORDER BY created_at DESC LIMIT ?1",
                vec![Box::new(limit)],
            ),
        };
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(&*param_refs, row_to_sim_journal)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_replay_session(&self, session: &crate::models::ReplaySession) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO sim_replay_sessions (id, symbol, interval, replay_date, current_index, speed, running, account_id, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session.id,
                session.symbol,
                session.interval,
                session.replay_date,
                session.current_index,
                session.speed,
                if session.running { 1 } else { 0 },
                session.account_id.as_deref().unwrap_or(""),
                session.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn load_replay_session(&self) -> AppResult<Option<crate::models::ReplaySession>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, symbol, interval, replay_date, current_index, speed, running, account_id, updated_at FROM sim_replay_sessions WHERE id='default' LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(crate::models::ReplaySession {
                id: row.get(0)?,
                symbol: row.get(1)?,
                interval: row.get(2)?,
                replay_date: row.get(3)?,
                current_index: row.get(4)?,
                speed: row.get(5)?,
                running: row.get::<_, i32>(6)? != 0,
                account_id: {
                    let s: String = row.get(7)?;
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                },
                updated_at: row.get(8)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_database_summary(&self, path: &str) -> AppResult<DatabaseSummary> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut total_size: i64 = 0;
        if let Ok(meta) = std::fs::metadata(path) {
            total_size = meta.len() as i64;
        }
        let mut stmt =
            conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
        let names: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        let mut tables = Vec::new();
        for name in names {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", name), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            tables.push(DatabaseTableStats {
                name,
                row_count: count,
                size_bytes: 0,
                last_updated: None,
            });
        }
        Ok(DatabaseSummary {
            path: path.to_string(),
            total_size_bytes: total_size,
            tables,
        })
    }

    pub fn get_database_domain_summary(&self, path: &str) -> AppResult<DatabaseDomainSummary> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let total_size: i64 = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);
        let specs = domain_specs();
        let mut domains = Vec::with_capacity(specs.len());
        for spec in specs {
            let (record_count, size_bytes, time_range, last_updated) =
                domain_stats(&conn, spec.tables)?;
            let quality = quality_for(record_count, last_updated.as_deref()).to_string();
            domains.push(DataDomain {
                code: spec.code.to_string(),
                name: spec.name.to_string(),
                description: spec.description.to_string(),
                record_count,
                size_bytes,
                time_range,
                last_updated,
                source: spec.source.to_string(),
                quality,
            });
        }
        Ok(DatabaseDomainSummary {
            path: path.to_string(),
            total_size_bytes: total_size,
            domains,
            updated_at: dt_to_iso(Utc::now()),
        })
    }

    pub fn get_recent_klines(&self, limit: i64) -> AppResult<Vec<KLine>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT symbol, interval, start_time, open, high, low, close, volume, turnover
             FROM klines ORDER BY start_time DESC LIMIT ?",
        )?;
        let mut rows: Vec<KLine> = stmt
            .query_map(params![limit], |row| {
                Ok(KLine {
                    symbol: row.get(0)?,
                    interval: row.get(1)?,
                    start_time: row.get(2)?,
                    open: row.get(3)?,
                    high: row.get(4)?,
                    low: row.get(5)?,
                    close: row.get(6)?,
                    volume: row.get(7)?,
                    turnover: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        rows.reverse();
        Ok(rows)
    }

    pub fn get_recent_news_records(&self, limit: i64) -> AppResult<Vec<NewsRecord>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, category_id, title, summary, url, display_time, content_hash,
                    ingested_at
             FROM news_items ORDER BY display_time DESC LIMIT ?",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(NewsRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                category_id: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                url: row.get(5)?,
                display_time: row.get(6)?,
                content_hash: row.get(7)?,
                ingested_at: row.get(8)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_domain_record_count(&self, code: &str) -> AppResult<i64> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let specs = domain_specs();
        let spec = specs
            .into_iter()
            .find(|s| s.code == code)
            .ok_or_else(|| AppError::Msg(format!("unknown domain: {code}")))?;
        let mut total = 0i64;
        for (table, _) in spec.tables {
            total += table_count(&conn, table)?;
        }
        Ok(total)
    }

    pub fn purge_old_news(&self, days: i64) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();
        let mut conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM news_classifications WHERE news_id IN
             (SELECT id FROM news_items WHERE display_time < ?)",
            params![cutoff],
        )?;
        let n = tx.execute(
            "DELETE FROM news_items WHERE display_time < ?",
            params![cutoff],
        )?;
        tx.commit()?;
        Ok(n)
    }

    pub fn purge_old_calendar_cache(&self, days: i64) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let n = conn.execute(
            "DELETE FROM calendar_cache WHERE fetched_at < ?",
            params![cutoff],
        )?;
        Ok(n)
    }

    pub fn purge_old_reports(&self, days: i64) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();
        let mut conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM followup_messages WHERE report_id IN
             (SELECT id FROM reports WHERE created_at < ?)",
            params![cutoff],
        )?;
        tx.execute(
            "DELETE FROM dimension_facts WHERE source_report_id IN
             (SELECT id FROM reports WHERE created_at < ?)",
            params![cutoff],
        )?;
        let n = tx.execute("DELETE FROM reports WHERE created_at < ?", params![cutoff])?;
        tx.commit()?;
        Ok(n)
    }

    pub fn purge_old_klines_except_daily(&self, keep_days: i64) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(keep_days)).to_rfc3339();
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let n = conn.execute(
            "DELETE FROM klines WHERE interval != '1d' AND start_time < ?",
            params![cutoff],
        )?;
        Ok(n)
    }
}

struct DomainSpec {
    code: &'static str,
    name: &'static str,
    description: &'static str,
    source: &'static str,
    tables: &'static [(&'static str, Option<&'static str>)],
}

fn domain_specs() -> Vec<DomainSpec> {
    vec![
        DomainSpec {
            code: "quotes",
            name: "行情/报价",
            description: "实时 tick、主力连续报价数据",
            source: "akshare/sina",
            tables: &[("ticks", Some("timestamp"))],
        },
        DomainSpec {
            code: "klines",
            name: "K 线",
            description: "期货与股票多周期 K 线数据",
            source: "akshare",
            tables: &[("klines", Some("start_time"))],
        },
        DomainSpec {
            code: "news",
            name: "资讯",
            description: "金十新闻与分类标签",
            source: "jin10",
            tables: &[("news_items", Some("display_time"))],
        },
        DomainSpec {
            code: "calendar",
            name: "日历",
            description: "财经日历事件缓存",
            source: "jin10",
            tables: &[("calendar_cache", Some("fetched_at"))],
        },
        DomainSpec {
            code: "reports",
            name: "报告",
            description: "LLM 生成的品种研报、复盘与跟进",
            source: "llm",
            tables: &[("reports", Some("created_at"))],
        },
        DomainSpec {
            code: "simulation",
            name: "模拟交易",
            description: "模拟账户、委托、成交、持仓、资金曲线与交易日志",
            source: "local",
            tables: &[
                ("sim_accounts", Some("updated_at")),
                ("sim_contract_rules", Some("updated_at")),
                ("sim_risk_rules", Some("updated_at")),
                ("sim_risk_events", Some("triggered_at")),
                ("sim_orders", Some("updated_at")),
                ("sim_trades", Some("traded_at")),
                ("sim_positions", Some("updated_at")),
                ("sim_equity_snapshots", Some("snapshot_at")),
                ("sim_journal_entries", Some("updated_at")),
                ("sim_replay_sessions", Some("updated_at")),
            ],
        },
        DomainSpec {
            code: "watchlist",
            name: "自选",
            description: "统一自选分组与标的",
            source: "local",
            tables: &[
                ("watchlist_groups", Some("updated_at")),
                ("watchlist_items", Some("updated_at")),
            ],
        },
        DomainSpec {
            code: "stocks",
            name: "A 股数据",
            description: "A 股标的、日线、板块、财务、估值与因子数据",
            source: "akshare/tushare",
            tables: &[
                ("stock_symbols", Some("updated_at")),
                ("stock_daily_bars", Some("trade_date")),
                ("stock_index_daily_bars", Some("trade_date")),
                ("stock_boards", Some("updated_at")),
                ("stock_board_members", Some("updated_at")),
                ("stock_board_snapshots", Some("trade_date")),
                ("stock_financial_metrics", Some("updated_at")),
                ("stock_valuation_snapshots", Some("trade_date")),
                ("stock_factor_snapshots", Some("factor_date")),
                ("stock_screen_templates", Some("updated_at")),
                ("stock_screen_results", Some("created_at")),
                ("stock_watchlists", Some("updated_at")),
            ],
        },
        DomainSpec {
            code: "settings",
            name: "配置",
            description: "应用偏好、密钥与 LLM 凭据",
            source: "local",
            tables: &[
                ("app_preferences", Some("updated_at")),
                ("app_secrets", Some("updated_at")),
                ("llm_credentials", Some("updated_at")),
            ],
        },
    ]
}

fn table_count(conn: &Connection, table: &str) -> AppResult<i64> {
    let count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    Ok(count)
}

fn estimate_table_size(conn: &Connection, table: &str, count: i64) -> AppResult<i64> {
    if count == 0 {
        return Ok(0);
    }
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let types: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(2))?
        .filter_map(|r| r.ok())
        .collect();
    let bytes_per_row: i64 = types
        .iter()
        .map(|t| {
            let upper = t.to_uppercase();
            if upper.contains("INT") || upper.contains("REAL") || upper.contains("NUM") {
                8
            } else if upper.contains("BLOB") {
                128
            } else if upper.contains("TEXT") {
                64
            } else {
                32
            }
        })
        .sum();
    Ok(count * bytes_per_row.max(1))
}

fn domain_stats(
    conn: &Connection,
    tables: &[(&str, Option<&str>)],
) -> AppResult<(i64, i64, Option<DataDomainTimeRange>, Option<String>)> {
    let mut record_count = 0i64;
    let mut size_bytes = 0i64;
    let mut min_time: Option<String> = None;
    let mut max_time: Option<String> = None;
    for (table, time_col) in tables {
        let count = table_count(conn, table)?;
        record_count += count;
        size_bytes += estimate_table_size(conn, table, count)?;
        if let Some(col) = time_col {
            let row = conn.query_row(
                &format!("SELECT MIN({col}), MAX({col}) FROM {table}"),
                [],
                |row| {
                    let mn: Option<String> = row.get(0)?;
                    let mx: Option<String> = row.get(1)?;
                    Ok((mn, mx))
                },
            );
            if let Ok((mn, mx)) = row {
                if let Some(mn) = mn {
                    if min_time.as_ref().map(|cur| mn < *cur).unwrap_or(true) {
                        min_time = Some(mn);
                    }
                }
                if let Some(mx) = mx {
                    if max_time.as_ref().map(|cur| mx > *cur).unwrap_or(true) {
                        max_time = Some(mx.clone());
                    }
                }
            }
        }
    }
    let time_range = if min_time.is_some() || max_time.is_some() {
        Some(DataDomainTimeRange {
            start: min_time,
            end: max_time.clone(),
        })
    } else {
        None
    };
    Ok((record_count, size_bytes, time_range, max_time))
}

fn quality_for(count: i64, last_updated: Option<&str>) -> &'static str {
    if count == 0 {
        return "pending";
    }
    let Some(ts) = last_updated else {
        return "pending";
    };
    let Some(dt) = parse_dt(ts) else {
        return "error";
    };
    let age = Utc::now().signed_duration_since(dt);
    if age.num_minutes() < 5 {
        "live"
    } else if age.num_hours() < 24 {
        "stale"
    } else {
        "error"
    }
}

fn row_to_report(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnalysisReport> {
    let tags_raw: String = row.get(8)?;
    let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();
    let dimension_summary: Option<serde_json::Value> = row
        .get::<_, Option<String>>(9)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
    let news_ids: Vec<String> = row
        .get::<_, Option<String>>(10)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Ok(AnalysisReport {
        id: row.get(0)?,
        symbol: row.get(1)?,
        trigger: row.get(2)?,
        provider: row.get(3)?,
        prompt_version: row.get(4)?,
        context_summary: row.get(5)?,
        content: row.get(6)?,
        created_at: row.get(7)?,
        tags,
        dimension_summary,
        news_ids,
        anomaly_reason: row.get(11).ok(),
    })
}

fn row_to_dimension_fact(row: &rusqlite::Row<'_>) -> rusqlite::Result<DimensionFact> {
    Ok(DimensionFact {
        id: row.get(0)?,
        symbol: row.get(1)?,
        dimension_code: row.get(2)?,
        fact: row.get(3)?,
        source_news_id: row.get(4)?,
        source_report_id: row.get(5)?,
        valid_until: row.get(6)?,
        created_at: row.get(7)?,
    })
}

fn row_to_followup(row: &rusqlite::Row<'_>) -> rusqlite::Result<FollowupMessage> {
    Ok(FollowupMessage {
        id: row.get(0)?,
        report_id: row.get(1)?,
        symbol: row.get(2)?,
        question: row.get(3)?,
        answer: row.get(4)?,
        provider: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn row_to_contract(row: &rusqlite::Row<'_>) -> rusqlite::Result<Contract> {
    Ok(Contract {
        symbol: row.get(0)?,
        exchange: row.get(1)?,
        name: row.get(2)?,
        product: row.get(3)?,
        multiplier: row.get(4)?,
        margin_ratio: row.get(5)?,
        listing_date: row.get(6)?,
        expiry_date: row.get(7)?,
    })
}

fn row_to_news_joined(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(NewsItemView, NewsClassificationView)> {
    let id: String = row.get(0)?;
    let item = NewsItemView {
        id: id.clone(),
        source: row.get(1)?,
        category_id: row.get(2)?,
        title: row.get(3)?,
        summary: row.get(4)?,
        url: row.get(5)?,
        display_time: row.get(6)?,
        classifications: vec![],
    };
    let symbol: Option<String> = row.get(7).ok();
    let dim: Option<String> = row.get(8).ok();
    let classification = match (symbol, dim) {
        (Some(symbol), Some(dimension_code)) => {
            let confidence: f32 = row.get(9).unwrap_or(0.0);
            let method: String = row.get(10).unwrap_or_else(|_| "rule".into());
            Some(NewsClassificationView {
                symbol,
                dimension_code: dimension_code.clone(),
                dimension_label: dimensions::dimension_label(&dimension_code).to_string(),
                confidence,
                method,
            })
        }
        _ => None,
    };
    Ok((
        item,
        classification.unwrap_or(NewsClassificationView {
            symbol: String::new(),
            dimension_code: String::new(),
            dimension_label: String::new(),
            confidence: 0.0,
            method: String::new(),
        }),
    ))
}

fn merge_news_rows(rows: Vec<(NewsItemView, NewsClassificationView)>) -> Vec<NewsItemView> {
    use std::collections::HashMap;
    let mut map: HashMap<String, NewsItemView> = HashMap::new();
    for (item, cls) in rows {
        let entry = map.entry(item.id.clone()).or_insert_with(|| NewsItemView {
            classifications: vec![],
            ..item.clone()
        });
        if !cls.dimension_code.is_empty()
            && !entry
                .classifications
                .iter()
                .any(|c| c.symbol == cls.symbol && c.dimension_code == cls.dimension_code)
        {
            entry.classifications.push(cls);
        }
    }
    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| b.display_time.cmp(&a.display_time));
    out
}

fn row_to_sim_order(row: &rusqlite::Row<'_>) -> rusqlite::Result<SimOrder> {
    Ok(SimOrder {
        id: row.get(0)?,
        account_id: row.get(1)?,
        symbol: row.get(2)?,
        name: row.get(3)?,
        side: row.get(4)?,
        offset: row.get(5)?,
        order_type: row.get(6)?,
        price: row.get(7).ok(),
        trigger_price: row.get(8).ok(),
        stop_loss_price: row.get(9).ok(),
        take_profit_price: row.get(10).ok(),
        oco_group_id: row.get(11).ok(),
        parent_order_id: row.get(12).ok(),
        tif: row.get(13).ok(),
        condition_operator: row.get(14).ok(),
        trailing_distance_ticks: row.get(15).ok(),
        trailing_reference_price: row.get(16).ok(),
        quantity: row.get(17)?,
        filled_quantity: row.get(18)?,
        status: row.get(19)?,
        reason: row.get(20).ok(),
        source: row.get(21)?,
        created_at: row.get(22)?,
        updated_at: row.get(23)?,
    })
}

fn row_to_sim_risk_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<SimRiskRule> {
    Ok(SimRiskRule {
        id: row.get(0)?,
        account_id: row.get(1)?,
        scope: row.get(2)?,
        symbol: row.get(3).ok(),
        rule_type: row.get(4)?,
        threshold: row.get(5)?,
        action: row.get(6)?,
        enabled: row.get::<_, i64>(7)? != 0,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_sim_risk_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<SimRiskEvent> {
    Ok(SimRiskEvent {
        id: row.get(0)?,
        account_id: row.get(1)?,
        rule_id: row.get(2)?,
        triggered_at: row.get(3)?,
        description: row.get(4)?,
        action_taken: row.get(5)?,
    })
}

fn row_to_sim_trade(row: &rusqlite::Row<'_>) -> rusqlite::Result<SimTrade> {
    Ok(SimTrade {
        id: row.get(0)?,
        order_id: row.get(1)?,
        account_id: row.get(2)?,
        symbol: row.get(3)?,
        name: row.get(4)?,
        side: row.get(5)?,
        offset: row.get(6)?,
        price: row.get(7)?,
        quantity: row.get(8)?,
        commission: row.get(9)?,
        slippage: row.get(10)?,
        realized_pnl: row.get(11)?,
        traded_at: row.get(12)?,
    })
}

fn row_to_sim_position(row: &rusqlite::Row<'_>) -> rusqlite::Result<SimPosition> {
    Ok(SimPosition {
        account_id: row.get(0)?,
        symbol: row.get(1)?,
        name: row.get(2)?,
        position_side: row.get(3)?,
        today_qty: row.get(4)?,
        history_qty: row.get(5)?,
        total_qty: row.get(6)?,
        avg_price: row.get(7)?,
        margin: row.get(8)?,
        unrealized_pnl: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_sim_journal(row: &rusqlite::Row<'_>) -> rusqlite::Result<SimJournalEntry> {
    Ok(SimJournalEntry {
        id: row.get(0)?,
        account_id: row.get(1)?,
        symbol: row.get(2).ok(),
        trade_id: row.get(3).ok(),
        report_id: row.get(4).ok(),
        title: row.get(5)?,
        thesis: row.get(6).ok(),
        execution_review: row.get(7).ok(),
        emotion_tags: row.get(8).ok(),
        score: row.get(9).ok(),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

impl Database {
    // =========================================================================
    // A 股（股票）数据访问方法
    // =========================================================================

    pub fn save_stock_symbols(&self, symbols: &[StockSymbol]) -> AppResult<usize> {
        if symbols.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_symbols
             (ts_code, symbol, name, exchange, market, industry, list_date, status, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)"
        )?;
        for s in symbols {
            stmt.execute(params![
                s.ts_code,
                s.symbol,
                s.name,
                s.exchange,
                s.market,
                s.industry,
                s.list_date,
                s.status,
                s.source,
                s.updated_at,
            ])?;
        }
        Ok(symbols.len())
    }

    pub fn list_stock_symbols(
        &self,
        query: Option<&str>,
        industry: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<StockSymbol>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut sql = String::from(
            "SELECT ts_code, symbol, name, exchange, market, industry, list_date, status, source, updated_at
             FROM stock_symbols WHERE status='active'"
        );
        let mut conditions = Vec::new();
        let mut bind: Vec<String> = Vec::new();
        if let Some(q) = query {
            conditions.push("(ts_code LIKE ? OR symbol LIKE ? OR name LIKE ?)");
            let pattern = format!("%{}%", q);
            bind.push(pattern.clone());
            bind.push(pattern.clone());
            bind.push(pattern);
        }
        if let Some(ind) = industry {
            conditions.push("industry=?");
            bind.push(ind.to_string());
        }
        if !conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY ts_code LIMIT ?");

        let mut stmt = conn.prepare(&sql)?;
        let params_vec: Vec<&dyn rusqlite::ToSql> = bind
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .chain(std::iter::once(&limit as &dyn rusqlite::ToSql))
            .collect();
        let rows = stmt.query_map(params_vec.as_slice(), row_to_stock_symbol)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_stock_symbol(&self, ts_code: &str) -> AppResult<Option<StockSymbol>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT ts_code, symbol, name, exchange, market, industry, list_date, status, source, updated_at
             FROM stock_symbols WHERE ts_code=? LIMIT 1"
        )?;
        let mut rows = stmt.query_map(params![ts_code], row_to_stock_symbol)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_stock_daily_bars(&self, bars: &[StockBar]) -> AppResult<usize> {
        if bars.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_daily_bars
             (ts_code, trade_date, open, high, low, close, pre_close, pct_chg, volume, amount, turnover_rate, adj_factor, adjustment, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"
        )?;
        for b in bars {
            stmt.execute(params![
                b.ts_code,
                b.trade_date,
                b.open,
                b.high,
                b.low,
                b.close,
                b.pre_close,
                b.pct_chg,
                b.volume,
                b.amount,
                b.turnover_rate,
                b.adj_factor,
                b.adjustment,
                b.source,
                b.updated_at,
            ])?;
        }
        Ok(bars.len())
    }

    pub fn get_stock_daily_bars(
        &self,
        ts_code: &str,
        adjustment: &str,
        limit: i64,
    ) -> AppResult<Vec<StockBar>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT ts_code, trade_date, open, high, low, close, pre_close, pct_chg, volume, amount, turnover_rate, adj_factor, adjustment, source, updated_at
             FROM stock_daily_bars
             WHERE ts_code=? AND adjustment=?
             ORDER BY trade_date DESC LIMIT ?"
        )?;
        let rows = stmt.query_map(params![ts_code, adjustment, limit], row_to_stock_bar)?;
        let mut out: Vec<StockBar> = rows.filter_map(|r| r.ok()).collect();
        out.reverse();
        Ok(out)
    }

    pub fn save_stock_index_daily_bars(&self, bars: &[StockIndexBar]) -> AppResult<usize> {
        if bars.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_index_daily_bars
             (index_code, trade_date, open, high, low, close, pct_chg, volume, amount, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?)"
        )?;
        for b in bars {
            stmt.execute(params![
                b.index_code,
                b.trade_date,
                b.open,
                b.high,
                b.low,
                b.close,
                b.pct_chg,
                b.volume,
                b.amount,
                b.source,
                b.updated_at,
            ])?;
        }
        Ok(bars.len())
    }

    pub fn get_stock_index_daily_bars(
        &self,
        index_code: &str,
        limit: i64,
    ) -> AppResult<Vec<StockIndexBar>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT index_code, trade_date, open, high, low, close, pct_chg, volume, amount, source, updated_at
             FROM stock_index_daily_bars
             WHERE index_code=?
             ORDER BY trade_date DESC LIMIT ?"
        )?;
        let rows = stmt.query_map(params![index_code, limit], row_to_stock_index_bar)?;
        let mut out: Vec<StockIndexBar> = rows.filter_map(|r| r.ok()).collect();
        out.reverse();
        Ok(out)
    }

    pub fn save_stock_boards(&self, boards: &[StockBoard]) -> AppResult<usize> {
        if boards.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_boards (board_code, board_name, board_type, source, updated_at)
             VALUES (?,?,?,?,?)"
        )?;
        for b in boards {
            stmt.execute(params![
                b.board_code,
                b.board_name,
                b.board_type,
                b.source,
                b.updated_at
            ])?;
        }
        Ok(boards.len())
    }

    pub fn list_stock_boards(&self, board_type: Option<&str>) -> AppResult<Vec<StockBoard>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let (sql, params_bind): (String, Vec<String>) = if let Some(bt) = board_type {
            (
                "SELECT board_code, board_name, board_type, source, updated_at
                 FROM stock_boards WHERE board_type=? ORDER BY board_name"
                    .to_string(),
                vec![bt.to_string()],
            )
        } else {
            (
                "SELECT board_code, board_name, board_type, source, updated_at
                 FROM stock_boards ORDER BY board_type, board_name"
                    .to_string(),
                vec![],
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = params_bind
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        let rows = if params.is_empty() {
            stmt.query_map([], row_to_stock_board)?
        } else {
            stmt.query_map(params.as_slice(), row_to_stock_board)?
        };
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_stock_board_members(&self, members: &[StockBoardMember]) -> AppResult<usize> {
        if members.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_board_members (board_code, ts_code, weight, source, updated_at)
             VALUES (?,?,?,?,?)"
        )?;
        for m in members {
            stmt.execute(params![
                m.board_code,
                m.ts_code,
                m.weight,
                m.source,
                m.updated_at
            ])?;
        }
        Ok(members.len())
    }

    pub fn list_stock_board_members(&self, board_code: &str) -> AppResult<Vec<StockBoardMember>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT board_code, ts_code, weight, source, updated_at
             FROM stock_board_members WHERE board_code=?",
        )?;
        let rows = stmt.query_map(params![board_code], row_to_stock_board_member)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_stock_board_snapshots(&self, snapshots: &[StockBoardSnapshot]) -> AppResult<usize> {
        if snapshots.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_board_snapshots
             (board_code, trade_date, pct_chg, amount, turnover_rate, net_flow, up_count, down_count, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)"
        )?;
        for s in snapshots {
            stmt.execute(params![
                s.board_code,
                s.trade_date,
                s.pct_chg,
                s.amount,
                s.turnover_rate,
                s.net_flow,
                s.up_count,
                s.down_count,
                s.source,
                s.updated_at,
            ])?;
        }
        Ok(snapshots.len())
    }

    pub fn get_stock_board_snapshot(
        &self,
        board_code: &str,
        trade_date: Option<&str>,
    ) -> AppResult<Option<StockBoardSnapshot>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let (sql, params_bind): (String, Vec<String>) = if let Some(d) = trade_date {
            (
                "SELECT board_code, trade_date, pct_chg, amount, turnover_rate, net_flow, up_count, down_count, source, updated_at
                 FROM stock_board_snapshots WHERE board_code=? AND trade_date=? LIMIT 1".to_string(),
                vec![board_code.to_string(), d.to_string()],
            )
        } else {
            (
                "SELECT board_code, trade_date, pct_chg, amount, turnover_rate, net_flow, up_count, down_count, source, updated_at
                 FROM stock_board_snapshots WHERE board_code=?
                 ORDER BY trade_date DESC LIMIT 1".to_string(),
                vec![board_code.to_string()],
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = params_bind
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        let mut rows = stmt.query_map(params.as_slice(), row_to_stock_board_snapshot)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_stock_financial_metrics(
        &self,
        metrics: &[StockFinancialMetric],
    ) -> AppResult<usize> {
        if metrics.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_financial_metrics
             (ts_code, report_period, report_type, revenue, revenue_yoy, net_profit, net_profit_yoy, roe, gross_margin, debt_ratio, operating_cash_flow, eps, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)"
        )?;
        for m in metrics {
            stmt.execute(params![
                m.ts_code,
                m.report_period,
                m.report_type,
                m.revenue,
                m.revenue_yoy,
                m.net_profit,
                m.net_profit_yoy,
                m.roe,
                m.gross_margin,
                m.debt_ratio,
                m.operating_cash_flow,
                m.eps,
                m.source,
                m.updated_at,
            ])?;
        }
        Ok(metrics.len())
    }

    pub fn get_stock_financial_metrics(
        &self,
        ts_code: &str,
        limit: i64,
    ) -> AppResult<Vec<StockFinancialMetric>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT ts_code, report_period, report_type, revenue, revenue_yoy, net_profit, net_profit_yoy, roe, gross_margin, debt_ratio, operating_cash_flow, eps, source, updated_at
             FROM stock_financial_metrics
             WHERE ts_code=?
             ORDER BY report_period DESC LIMIT ?"
        )?;
        let rows = stmt.query_map(params![ts_code, limit], row_to_stock_financial_metric)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_stock_valuation_snapshots(
        &self,
        snaps: &[StockValuationSnapshot],
    ) -> AppResult<usize> {
        if snaps.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_valuation_snapshots
             (ts_code, trade_date, pe_ttm, pb, ps_ttm, dividend_yield, market_cap, float_market_cap, pe_percentile, pb_percentile, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?)"
        )?;
        for v in snaps {
            stmt.execute(params![
                v.ts_code,
                v.trade_date,
                v.pe_ttm,
                v.pb,
                v.ps_ttm,
                v.dividend_yield,
                v.market_cap,
                v.float_market_cap,
                v.pe_percentile,
                v.pb_percentile,
                v.source,
                v.updated_at,
            ])?;
        }
        Ok(snaps.len())
    }

    pub fn get_stock_valuation_snapshots(
        &self,
        ts_code: &str,
        limit: i64,
    ) -> AppResult<Vec<StockValuationSnapshot>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT ts_code, trade_date, pe_ttm, pb, ps_ttm, dividend_yield, market_cap, float_market_cap, pe_percentile, pb_percentile, source, updated_at
             FROM stock_valuation_snapshots
             WHERE ts_code=?
             ORDER BY trade_date DESC LIMIT ?"
        )?;
        let rows = stmt.query_map(params![ts_code, limit], row_to_stock_valuation_snapshot)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_latest_stock_valuation(
        &self,
        ts_code: &str,
    ) -> AppResult<Option<StockValuationSnapshot>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT ts_code, trade_date, pe_ttm, pb, ps_ttm, dividend_yield, market_cap, float_market_cap, pe_percentile, pb_percentile, source, updated_at
             FROM stock_valuation_snapshots
             WHERE ts_code=? ORDER BY trade_date DESC LIMIT 1"
        )?;
        let mut rows = stmt.query_map(params![ts_code], row_to_stock_valuation_snapshot)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_stock_factor_snapshots(&self, snaps: &[StockFactorSnapshot]) -> AppResult<usize> {
        if snaps.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO stock_factor_snapshots
             (ts_code, factor_date, momentum, quality, valuation, growth, volatility, liquidity, capital_flow, score, factor_version, source, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)"
        )?;
        for f in snaps {
            stmt.execute(params![
                f.ts_code,
                f.factor_date,
                f.momentum,
                f.quality,
                f.valuation,
                f.growth,
                f.volatility,
                f.liquidity,
                f.capital_flow,
                f.score,
                f.factor_version,
                f.source,
                f.updated_at,
            ])?;
        }
        Ok(snaps.len())
    }

    pub fn save_stock_screen_template(&self, template: &StockScreenTemplate) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_screen_templates (id, name, criteria_json, created_at, updated_at)
             VALUES (?,?,?,?,?)",
            params![
                template.id,
                template.name,
                template.criteria_json,
                template.created_at,
                template.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_stock_screen_templates(&self) -> AppResult<Vec<StockScreenTemplate>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, criteria_json, created_at, updated_at
             FROM stock_screen_templates ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], row_to_stock_screen_template)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_stock_screen_template(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute("DELETE FROM stock_screen_templates WHERE id=?", params![id])?;
        Ok(())
    }

    pub fn save_stock_screen_result(&self, result: &StockScreenResult) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_screen_results
             (id, template_id, name, criteria_json, result_json, trade_date, report_period, source_summary, created_at)
             VALUES (?,?,?,?,?,?,?,?,?)",
            params![
                result.id,
                result.template_id,
                result.name,
                result.criteria_json,
                result.result_json,
                result.trade_date,
                result.report_period,
                result.source_summary,
                result.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn save_stock_watchlist(&self, watchlist: &StockWatchlist) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let symbols_json = serde_json::to_string(&watchlist.symbols)?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_watchlists (id, name, symbols_json, created_at, updated_at)
             VALUES (?,?,?,?,?)",
            params![
                watchlist.id,
                watchlist.name,
                symbols_json,
                watchlist.created_at,
                watchlist.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_stock_watchlists(&self) -> AppResult<Vec<StockWatchlist>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, symbols_json, created_at, updated_at
             FROM stock_watchlists ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], row_to_stock_watchlist)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_stock_watchlist(&self, id: &str) -> AppResult<Option<StockWatchlist>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, symbols_json, created_at, updated_at
             FROM stock_watchlists WHERE id=? LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_stock_watchlist)?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete_stock_watchlist(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute("DELETE FROM stock_watchlists WHERE id=?", params![id])?;
        Ok(())
    }

    // ============================================================================
    // 统一自选（CMC 重构）
    // ============================================================================

    pub fn list_watchlist_groups(&self) -> AppResult<Vec<WatchlistGroup>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, sort_order, created_at, updated_at
             FROM watchlist_groups ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WatchlistGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_watchlist_group(&self, group: &WatchlistGroup) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO watchlist_groups (id, name, sort_order, created_at, updated_at)
             VALUES (?,?,?,?,?)",
            params![
                group.id,
                group.name,
                group.sort_order,
                group.created_at,
                group.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete_watchlist_group(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute("DELETE FROM watchlist_items WHERE group_id=?", params![id])?;
        conn.execute("DELETE FROM watchlist_groups WHERE id=?", params![id])?;
        Ok(())
    }

    pub fn list_watchlist_items(&self, group_id: Option<&str>) -> AppResult<Vec<WatchlistItem>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, group_id, asset_type, symbol, name, notes, alert_price, alert_pct, sort_order, created_at, updated_at
             FROM watchlist_items WHERE (group_id = ?1 OR ?1 IS NULL)
             ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt.query_map(params![group_id], |row| {
            Ok(WatchlistItem {
                id: row.get(0)?,
                group_id: row.get(1)?,
                asset_type: row.get(2)?,
                symbol: row.get(3)?,
                name: row.get(4)?,
                notes: row.get(5)?,
                alert_price: row.get(6)?,
                alert_pct: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_watchlist_item(&self, item: &WatchlistItem) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO watchlist_items
             (id, group_id, asset_type, symbol, name, notes, alert_price, alert_pct, sort_order, created_at, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            params![
                item.id,
                item.group_id,
                item.asset_type,
                item.symbol,
                item.name,
                item.notes,
                item.alert_price,
                item.alert_pct,
                item.sort_order,
                item.created_at,
                item.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete_watchlist_item(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute("DELETE FROM watchlist_items WHERE id=?", params![id])?;
        Ok(())
    }

    pub fn get_watchlist_summary(&self) -> AppResult<WatchlistSummary> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let total_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM watchlist_items", [], |row| row.get(0))?;
        let futures_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM watchlist_items WHERE asset_type=?",
            params!["futures"],
            |row| row.get(0),
        )?;
        let stock_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM watchlist_items WHERE asset_type=?",
            params!["stock"],
            |row| row.get(0),
        )?;
        Ok(WatchlistSummary {
            total_count,
            futures_count,
            stock_count,
            move_count: 0,
            event_count: 0,
        })
    }

    pub fn is_in_watchlist(&self, symbol: &str, asset_type: &str) -> AppResult<bool> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM watchlist_items WHERE symbol=? AND asset_type=? LIMIT 1",
            params![symbol, asset_type],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn count_stock_symbols(&self) -> AppResult<i64> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM stock_symbols")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_latest_stock_daily_bar_date(&self) -> AppResult<Option<String>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT MAX(trade_date) FROM stock_daily_bars")?;
        let date: Option<String> = stmt.query_row([], |row| row.get(0))?;
        Ok(date)
    }

    // 汇总市场宽度：基于最新一日的 stock_daily_bars
    pub fn get_stock_market_breadth(&self) -> AppResult<StockMarketBreadth> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let trade_date: Option<String> =
            conn.query_row("SELECT MAX(trade_date) FROM stock_daily_bars", [], |row| {
                row.get::<_, Option<String>>(0)
            })?;
        let trade_date = match trade_date {
            Some(d) => d,
            None => {
                return Ok(StockMarketBreadth {
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
                    updated_at: dt_to_iso(Utc::now()),
                });
            }
        };

        let mut stmt = conn.prepare(
            "SELECT
                COALESCE(SUM(CASE WHEN pct_chg > 0 THEN 1 ELSE 0 END), 0) AS up_count,
                COALESCE(SUM(CASE WHEN pct_chg < 0 THEN 1 ELSE 0 END), 0) AS down_count,
                COALESCE(SUM(CASE WHEN pct_chg = 0 OR pct_chg IS NULL THEN 1 ELSE 0 END), 0) AS flat_count,
                COALESCE(SUM(CASE WHEN pct_chg >= 9.5 THEN 1 ELSE 0 END), 0) AS limit_up,
                COALESCE(SUM(CASE WHEN pct_chg <= -9.5 THEN 1 ELSE 0 END), 0) AS limit_down,
                COALESCE(SUM(amount), 0) AS total_amount
             FROM stock_daily_bars WHERE trade_date=? AND adjustment='none'"
        )?;
        let (up, down, flat, limit_up, limit_down, total_amount): (i64, i64, i64, i64, i64, f64) =
            stmt.query_row(params![trade_date], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?;

        let prev_date: Option<String> = conn.query_row(
            "SELECT MAX(trade_date) FROM stock_daily_bars WHERE trade_date < ?",
            params![trade_date],
            |row| row.get::<_, Option<String>>(0),
        )?;
        let prev_amount: Option<f64> = if let Some(pd) = prev_date {
            conn.query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM stock_daily_bars WHERE trade_date=? AND adjustment='none'",
                params![pd],
                |row| row.get(0),
            )
            .optional()?
        } else {
            None
        };
        let amount_change_pct = if let Some(prev) = prev_amount {
            if prev > 0.0 {
                Some((total_amount - prev) / prev * 100.0)
            } else {
                None
            }
        } else {
            None
        };

        Ok(StockMarketBreadth {
            trade_date: Some(trade_date),
            up_count: up,
            down_count: down,
            flat_count: flat,
            limit_up_count: limit_up,
            limit_down_count: limit_down,
            total_amount: Some(total_amount),
            prev_amount,
            amount_change_pct,
            source: "local".to_string(),
            updated_at: dt_to_iso(Utc::now()),
        })
    }
}

fn row_to_stock_symbol(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockSymbol> {
    Ok(StockSymbol {
        ts_code: row.get(0)?,
        symbol: row.get(1)?,
        name: row.get(2)?,
        exchange: row.get(3)?,
        market: row.get(4)?,
        industry: row.get(5)?,
        list_date: row.get(6)?,
        status: row.get(7)?,
        source: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_stock_bar(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockBar> {
    Ok(StockBar {
        ts_code: row.get(0)?,
        trade_date: row.get(1)?,
        open: row.get(2)?,
        high: row.get(3)?,
        low: row.get(4)?,
        close: row.get(5)?,
        pre_close: row.get(6)?,
        pct_chg: row.get(7)?,
        volume: row.get(8)?,
        amount: row.get(9)?,
        turnover_rate: row.get(10)?,
        adj_factor: row.get(11)?,
        adjustment: row.get(12)?,
        source: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn row_to_stock_index_bar(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockIndexBar> {
    Ok(StockIndexBar {
        index_code: row.get(0)?,
        trade_date: row.get(1)?,
        open: row.get(2)?,
        high: row.get(3)?,
        low: row.get(4)?,
        close: row.get(5)?,
        pct_chg: row.get(6)?,
        volume: row.get(7)?,
        amount: row.get(8)?,
        source: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_stock_board(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockBoard> {
    Ok(StockBoard {
        board_code: row.get(0)?,
        board_name: row.get(1)?,
        board_type: row.get(2)?,
        source: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn row_to_stock_board_member(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockBoardMember> {
    Ok(StockBoardMember {
        board_code: row.get(0)?,
        ts_code: row.get(1)?,
        weight: row.get(2)?,
        source: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn row_to_stock_board_snapshot(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockBoardSnapshot> {
    Ok(StockBoardSnapshot {
        board_code: row.get(0)?,
        trade_date: row.get(1)?,
        pct_chg: row.get(2)?,
        amount: row.get(3)?,
        turnover_rate: row.get(4)?,
        net_flow: row.get(5)?,
        up_count: row.get(6)?,
        down_count: row.get(7)?,
        source: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_stock_financial_metric(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StockFinancialMetric> {
    Ok(StockFinancialMetric {
        ts_code: row.get(0)?,
        report_period: row.get(1)?,
        report_type: row.get(2)?,
        revenue: row.get(3)?,
        revenue_yoy: row.get(4)?,
        net_profit: row.get(5)?,
        net_profit_yoy: row.get(6)?,
        roe: row.get(7)?,
        gross_margin: row.get(8)?,
        debt_ratio: row.get(9)?,
        operating_cash_flow: row.get(10)?,
        eps: row.get(11)?,
        source: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn row_to_stock_valuation_snapshot(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StockValuationSnapshot> {
    Ok(StockValuationSnapshot {
        ts_code: row.get(0)?,
        trade_date: row.get(1)?,
        pe_ttm: row.get(2)?,
        pb: row.get(3)?,
        ps_ttm: row.get(4)?,
        dividend_yield: row.get(5)?,
        market_cap: row.get(6)?,
        float_market_cap: row.get(7)?,
        pe_percentile: row.get(8)?,
        pb_percentile: row.get(9)?,
        source: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn row_to_stock_screen_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockScreenTemplate> {
    Ok(StockScreenTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        criteria_json: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn row_to_stock_watchlist(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockWatchlist> {
    let symbols_json: String = row.get(2)?;
    let symbols = serde_json::from_str(&symbols_json).unwrap_or_default();
    Ok(StockWatchlist {
        id: row.get(0)?,
        name: row.get(1)?,
        symbols,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[cfg(test)]
mod stock_tests {
    use super::*;
    use crate::models::{
        StockBar, StockBoard, StockBoardMember, StockBoardSnapshot, StockFinancialMetric,
        StockIndexBar, StockPaperAccount, StockPaperOrder, StockPaperPosition, StockPaperTrade,
        StockScreenTemplate, StockSymbol, StockValuationSnapshot, StockWatchlist, WatchlistGroup,
        WatchlistItem,
    };

    fn temp_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Database::open(&path).unwrap();
        db.init_schema().unwrap();
        db
    }

    #[test]
    fn stock_symbols_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let symbols = vec![StockSymbol {
            ts_code: "600000.SH".to_string(),
            symbol: "600000".to_string(),
            name: "浦发银行".to_string(),
            exchange: "SH".to_string(),
            market: Some("主板".to_string()),
            industry: Some("银行".to_string()),
            list_date: Some("1999-11-10".to_string()),
            status: "active".to_string(),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_symbols(&symbols).unwrap(), 1);
        let fetched = db.list_stock_symbols(None, None, 10).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].ts_code, "600000.SH");

        let by_query = db.list_stock_symbols(Some("浦发"), None, 10).unwrap();
        assert_eq!(by_query.len(), 1);

        let by_industry = db.list_stock_symbols(None, Some("银行"), 10).unwrap();
        assert_eq!(by_industry.len(), 1);
    }

    #[test]
    fn stock_daily_bars_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let bars = vec![
            StockBar {
                ts_code: "600000.SH".to_string(),
                trade_date: "20260101".to_string(),
                open: Some(10.0),
                high: Some(10.5),
                low: Some(9.8),
                close: Some(10.2),
                pre_close: Some(10.0),
                pct_chg: Some(2.0),
                volume: Some(10000.0),
                amount: Some(102000.0),
                turnover_rate: Some(0.01),
                adj_factor: None,
                adjustment: "none".to_string(),
                source: "test".to_string(),
                updated_at: now.clone(),
            },
            StockBar {
                ts_code: "600000.SH".to_string(),
                trade_date: "20260102".to_string(),
                open: Some(10.2),
                high: Some(10.3),
                low: Some(10.0),
                close: Some(10.1),
                pre_close: Some(10.2),
                pct_chg: Some(-0.98),
                volume: Some(8000.0),
                amount: Some(80800.0),
                turnover_rate: Some(0.008),
                adj_factor: None,
                adjustment: "none".to_string(),
                source: "test".to_string(),
                updated_at: now.clone(),
            },
        ];
        assert_eq!(db.save_stock_daily_bars(&bars).unwrap(), 2);
        let fetched = db.get_stock_daily_bars("600000.SH", "none", 10).unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].trade_date, "20260101");
        assert_eq!(fetched[1].trade_date, "20260102");
    }

    #[test]
    fn stock_index_bars_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let bars = vec![StockIndexBar {
            index_code: "000001.SH".to_string(),
            trade_date: "20260102".to_string(),
            open: Some(3000.0),
            high: Some(3010.0),
            low: Some(2990.0),
            close: Some(3005.0),
            pct_chg: Some(0.17),
            volume: Some(1000000.0),
            amount: Some(1e12),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_index_daily_bars(&bars).unwrap(), 1);
        let fetched = db.get_stock_index_daily_bars("000001.SH", 10).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].close, Some(3005.0));
    }

    #[test]
    fn stock_boards_and_members_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let boards = vec![StockBoard {
            board_code: "BK0475".to_string(),
            board_name: "银行".to_string(),
            board_type: "industry".to_string(),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_boards(&boards).unwrap(), 1);
        let members = vec![StockBoardMember {
            board_code: "BK0475".to_string(),
            ts_code: "600000.SH".to_string(),
            weight: Some(0.15),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_board_members(&members).unwrap(), 1);

        let fetched_boards = db.list_stock_boards(Some("industry")).unwrap();
        assert_eq!(fetched_boards.len(), 1);

        let fetched_members = db.list_stock_board_members("BK0475").unwrap();
        assert_eq!(fetched_members.len(), 1);
        assert_eq!(fetched_members[0].ts_code, "600000.SH");
    }

    #[test]
    fn stock_board_snapshot_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let snaps = vec![StockBoardSnapshot {
            board_code: "BK0475".to_string(),
            trade_date: "20260102".to_string(),
            pct_chg: Some(1.2),
            amount: Some(1e10),
            turnover_rate: Some(0.8),
            net_flow: Some(1e9),
            up_count: Some(35),
            down_count: Some(2),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_board_snapshots(&snaps).unwrap(), 1);
        let fetched = db.get_stock_board_snapshot("BK0475", None).unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().up_count, Some(35));
    }

    #[test]
    fn stock_market_breadth_aggregation() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let bars = vec![
            StockBar {
                ts_code: "600000.SH".to_string(),
                trade_date: "20260102".to_string(),
                open: Some(10.0),
                high: Some(10.5),
                low: Some(9.8),
                close: Some(10.2),
                pre_close: Some(10.0),
                pct_chg: Some(2.0),
                volume: Some(10000.0),
                amount: Some(100000.0),
                turnover_rate: Some(0.01),
                adj_factor: None,
                adjustment: "none".to_string(),
                source: "test".to_string(),
                updated_at: now.clone(),
            },
            StockBar {
                ts_code: "000001.SZ".to_string(),
                trade_date: "20260102".to_string(),
                open: Some(10.0),
                high: Some(10.1),
                low: Some(9.9),
                close: Some(9.95),
                pre_close: Some(10.0),
                pct_chg: Some(-0.5),
                volume: Some(5000.0),
                amount: Some(50000.0),
                turnover_rate: Some(0.005),
                adj_factor: None,
                adjustment: "none".to_string(),
                source: "test".to_string(),
                updated_at: now.clone(),
            },
        ];
        db.save_stock_daily_bars(&bars).unwrap();
        let breadth = db.get_stock_market_breadth().unwrap();
        assert_eq!(breadth.up_count, 1);
        assert_eq!(breadth.down_count, 1);
        assert_eq!(breadth.flat_count, 0);
        assert_eq!(breadth.total_amount, Some(150000.0));
    }

    #[test]
    fn stock_financial_and_valuation_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let metrics = vec![StockFinancialMetric {
            ts_code: "600000.SH".to_string(),
            report_period: "2025-12-31".to_string(),
            report_type: Some("年报".to_string()),
            revenue: Some(1e11),
            revenue_yoy: Some(-3.5),
            net_profit: Some(4e10),
            net_profit_yoy: Some(1.2),
            roe: Some(7.8),
            gross_margin: None,
            debt_ratio: Some(92.1),
            operating_cash_flow: None,
            eps: Some(1.43),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_financial_metrics(&metrics).unwrap(), 1);
        let fetched = db.get_stock_financial_metrics("600000.SH", 10).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].report_period, "2025-12-31");

        let vals = vec![StockValuationSnapshot {
            ts_code: "600000.SH".to_string(),
            trade_date: "20260102".to_string(),
            pe_ttm: Some(4.5),
            pb: Some(0.42),
            ps_ttm: Some(1.8),
            dividend_yield: Some(5.2),
            market_cap: Some(2.85e11),
            float_market_cap: Some(2.85e11),
            pe_percentile: Some(12.5),
            pb_percentile: Some(8.3),
            source: "test".to_string(),
            updated_at: now.clone(),
        }];
        assert_eq!(db.save_stock_valuation_snapshots(&vals).unwrap(), 1);
        let latest = db.get_latest_stock_valuation("600000.SH").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().pe_ttm, Some(4.5));
    }

    #[test]
    fn stock_screen_template_delete() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let template = StockScreenTemplate {
            id: "tpl-1".to_string(),
            name: "低估值".to_string(),
            criteria_json: "{}".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.save_stock_screen_template(&template).unwrap();
        assert_eq!(db.list_stock_screen_templates().unwrap().len(), 1);
        db.delete_stock_screen_template("tpl-1").unwrap();
        assert!(db.list_stock_screen_templates().unwrap().is_empty());
    }

    #[test]
    fn stock_watchlist_crud() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let watchlist = StockWatchlist {
            id: "wl-1".to_string(),
            name: "测试池".to_string(),
            symbols: vec!["600000.SH".to_string()],
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.save_stock_watchlist(&watchlist).unwrap();
        let fetched = db.list_stock_watchlists().unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].symbols, vec!["600000.SH"]);
        db.delete_stock_watchlist("wl-1").unwrap();
        assert!(db.list_stock_watchlists().unwrap().is_empty());
    }

    #[test]
    fn unified_watchlist_crud_and_summary() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let group = WatchlistGroup {
            id: "group-1".to_string(),
            name: "重点观察".to_string(),
            sort_order: 1,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.save_watchlist_group(&group).unwrap();

        let item = WatchlistItem {
            id: "item-1".to_string(),
            group_id: group.id.clone(),
            asset_type: "futures".to_string(),
            symbol: "RB0".to_string(),
            name: "螺纹钢".to_string(),
            notes: Some("测试备注".to_string()),
            alert_price: None,
            alert_pct: Some(2.0),
            sort_order: 1,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.save_watchlist_item(&item).unwrap();

        let items = db.list_watchlist_items(Some(&group.id)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].symbol, "RB0");
        assert_eq!(items[0].notes.as_deref(), Some("测试备注"));
        assert!(db.is_in_watchlist("RB0", "futures").unwrap());

        let summary = db.get_watchlist_summary().unwrap();
        assert_eq!(summary.total_count, 1);
        assert_eq!(summary.futures_count, 1);
        assert_eq!(summary.stock_count, 0);

        db.delete_watchlist_item("item-1").unwrap();
        assert!(!db.is_in_watchlist("RB0", "futures").unwrap());
    }

    #[test]
    fn stock_paper_portfolio_roundtrip() {
        let db = temp_db();
        let now = dt_to_iso(Utc::now());
        let account = StockPaperAccount {
            id: "acc-1".to_string(),
            name: "测试组合".to_string(),
            initial_balance: 1_000_000.0,
            cash_balance: 900_000.0,
            market_value: 100_000.0,
            total_equity: 1_000_000.0,
            total_cost: 99_000.0,
            realized_pnl: 0.0,
            unrealized_pnl: 1_000.0,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.save_stock_paper_account(&account).unwrap();
        let fetched = db.get_stock_paper_account("acc-1").unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "测试组合");

        let order = StockPaperOrder {
            id: "order-1".to_string(),
            account_id: "acc-1".to_string(),
            ts_code: "600000.SH".to_string(),
            name: "浦发银行".to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            price: Some(10.0),
            quantity: 100,
            filled_quantity: 100,
            status: "filled".to_string(),
            reason: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        db.save_stock_paper_order(&order).unwrap();
        let orders = db.list_stock_paper_orders("acc-1").unwrap();
        assert_eq!(orders.len(), 1);

        let position = StockPaperPosition {
            account_id: "acc-1".to_string(),
            ts_code: "600000.SH".to_string(),
            name: "浦发银行".to_string(),
            quantity: 100,
            available_quantity: 100,
            avg_cost: 10.0,
            total_cost: 1000.0,
            market_value: 1020.0,
            unrealized_pnl: 20.0,
            updated_at: now.clone(),
        };
        db.save_stock_paper_position(&position).unwrap();
        let positions = db.list_stock_paper_positions("acc-1").unwrap();
        assert_eq!(positions.len(), 1);
        let pos = db.get_stock_paper_position("acc-1", "600000.SH").unwrap();
        assert!(pos.is_some());

        let trade = StockPaperTrade {
            id: "trade-1".to_string(),
            order_id: "order-1".to_string(),
            account_id: "acc-1".to_string(),
            ts_code: "600000.SH".to_string(),
            name: "浦发银行".to_string(),
            side: "buy".to_string(),
            price: 10.0,
            quantity: 100,
            commission: 5.0,
            traded_at: now.clone(),
        };
        db.save_stock_paper_trade(&trade).unwrap();
        let trades = db.list_stock_paper_trades("acc-1").unwrap();
        assert_eq!(trades.len(), 1);

        let accounts = db.list_stock_paper_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
    }
}

impl Database {
    // =========================================================================
    // A 股模拟组合数据访问方法
    // =========================================================================

    pub fn save_stock_paper_account(&self, account: &StockPaperAccount) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_paper_accounts
             (id, name, initial_balance, cash_balance, market_value, total_equity, total_cost, realized_pnl, unrealized_pnl, status, created_at, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                account.id,
                account.name,
                account.initial_balance,
                account.cash_balance,
                account.market_value,
                account.total_equity,
                account.total_cost,
                account.realized_pnl,
                account.unrealized_pnl,
                account.status,
                account.created_at,
                account.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_stock_paper_accounts(&self) -> AppResult<Vec<StockPaperAccount>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, initial_balance, cash_balance, market_value, total_equity, total_cost, realized_pnl, unrealized_pnl, status, created_at, updated_at
             FROM stock_paper_accounts ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], row_to_stock_paper_account)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_stock_paper_account(&self, id: &str) -> AppResult<Option<StockPaperAccount>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, initial_balance, cash_balance, market_value, total_equity, total_cost, realized_pnl, unrealized_pnl, status, created_at, updated_at
             FROM stock_paper_accounts WHERE id=? LIMIT 1"
        )?;
        let mut rows = stmt.query_map(params![id], row_to_stock_paper_account)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_stock_paper_order(&self, order: &StockPaperOrder) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_paper_orders
             (id, account_id, ts_code, name, side, order_type, price, quantity, filled_quantity, status, reason, created_at, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                order.id,
                order.account_id,
                order.ts_code,
                order.name,
                order.side,
                order.order_type,
                order.price,
                order.quantity,
                order.filled_quantity,
                order.status,
                order.reason,
                order.created_at,
                order.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_stock_paper_orders(&self, account_id: &str) -> AppResult<Vec<StockPaperOrder>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, account_id, ts_code, name, side, order_type, price, quantity, filled_quantity, status, reason, created_at, updated_at
             FROM stock_paper_orders WHERE account_id=? ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![account_id], row_to_stock_paper_order)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_stock_paper_position(&self, position: &StockPaperPosition) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_paper_positions
             (account_id, ts_code, name, quantity, available_quantity, avg_cost, total_cost, market_value, unrealized_pnl, updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)",
            params![
                position.account_id,
                position.ts_code,
                position.name,
                position.quantity,
                position.available_quantity,
                position.avg_cost,
                position.total_cost,
                position.market_value,
                position.unrealized_pnl,
                position.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_stock_paper_positions(
        &self,
        account_id: &str,
    ) -> AppResult<Vec<StockPaperPosition>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT account_id, ts_code, name, quantity, available_quantity, avg_cost, total_cost, market_value, unrealized_pnl, updated_at
             FROM stock_paper_positions WHERE account_id=? ORDER BY ts_code"
        )?;
        let rows = stmt.query_map(params![account_id], row_to_stock_paper_position)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_stock_paper_position(
        &self,
        account_id: &str,
        ts_code: &str,
    ) -> AppResult<Option<StockPaperPosition>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT account_id, ts_code, name, quantity, available_quantity, avg_cost, total_cost, market_value, unrealized_pnl, updated_at
             FROM stock_paper_positions WHERE account_id=? AND ts_code=? LIMIT 1"
        )?;
        let mut rows = stmt.query_map(params![account_id, ts_code], row_to_stock_paper_position)?;
        Ok(rows.next().transpose()?)
    }

    pub fn save_stock_paper_trade(&self, trade: &StockPaperTrade) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO stock_paper_trades
             (id, order_id, account_id, ts_code, name, side, price, quantity, commission, traded_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)",
            params![
                trade.id,
                trade.order_id,
                trade.account_id,
                trade.ts_code,
                trade.name,
                trade.side,
                trade.price,
                trade.quantity,
                trade.commission,
                trade.traded_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_stock_paper_trades(&self, account_id: &str) -> AppResult<Vec<StockPaperTrade>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, order_id, account_id, ts_code, name, side, price, quantity, commission, traded_at
             FROM stock_paper_trades WHERE account_id=? ORDER BY traded_at DESC"
        )?;
        let rows = stmt.query_map(params![account_id], row_to_stock_paper_trade)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn row_to_stock_paper_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockPaperAccount> {
    Ok(StockPaperAccount {
        id: row.get(0)?,
        name: row.get(1)?,
        initial_balance: row.get(2)?,
        cash_balance: row.get(3)?,
        market_value: row.get(4)?,
        total_equity: row.get(5)?,
        total_cost: row.get(6)?,
        realized_pnl: row.get(7)?,
        unrealized_pnl: row.get(8)?,
        status: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn row_to_stock_paper_order(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockPaperOrder> {
    Ok(StockPaperOrder {
        id: row.get(0)?,
        account_id: row.get(1)?,
        ts_code: row.get(2)?,
        name: row.get(3)?,
        side: row.get(4)?,
        order_type: row.get(5)?,
        price: row.get(6)?,
        quantity: row.get(7)?,
        filled_quantity: row.get(8)?,
        status: row.get(9)?,
        reason: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn row_to_stock_paper_position(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockPaperPosition> {
    Ok(StockPaperPosition {
        account_id: row.get(0)?,
        ts_code: row.get(1)?,
        name: row.get(2)?,
        quantity: row.get(3)?,
        available_quantity: row.get(4)?,
        avg_cost: row.get(5)?,
        total_cost: row.get(6)?,
        market_value: row.get(7)?,
        unrealized_pnl: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_stock_paper_trade(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockPaperTrade> {
    Ok(StockPaperTrade {
        id: row.get(0)?,
        order_id: row.get(1)?,
        account_id: row.get(2)?,
        ts_code: row.get(3)?,
        name: row.get(4)?,
        side: row.get(5)?,
        price: row.get(6)?,
        quantity: row.get(7)?,
        commission: row.get(8)?,
        traded_at: row.get(9)?,
    })
}
