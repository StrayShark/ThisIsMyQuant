use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::config::UserPreferences;
use crate::error::{AppError, AppResult};
use crate::models::{
    AnalysisReport, CalendarEvent, Contract, DimensionFact, FollowupMessage, KLine, LiquiditySnapshot,
    NewsClassification, NewsItemView, NewsClassificationView, NewsRecord, Tick, dt_to_iso,
};
use crate::engine::dimensions;

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
        Ok(())
    }

    pub fn save_klines(&self, klines: &[KLine]) -> AppResult<usize> {
        if klines.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO klines VALUES (?,?,?,?,?,?,?,?,?)",
        )?;
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
            count += conn.execute("DELETE FROM news_classifications WHERE news_id=?", params![id])?;
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
            params![
                symbol,
                interval,
                dt_to_iso(start),
                dt_to_iso(end),
                limit
            ],
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

    pub fn get_latest_liquidity_map(&self) -> AppResult<std::collections::HashMap<String, LiquiditySnapshot>> {
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

    pub fn get_dimension_facts(
        &self,
        symbol: &str,
        limit: i64,
    ) -> AppResult<Vec<DimensionFact>> {
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
        let json: Option<String> = stmt
            .query_row(params![cache_key], |row| row.get(0))
            .ok();
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

    pub fn load_user_preferences(&self) -> AppResult<Option<UserPreferences>> {
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

    pub fn save_user_preferences(&self, prefs: &UserPreferences) -> AppResult<()> {
        let json = serde_json::to_string(prefs)
            .map_err(|e| AppError::Msg(format!("serialize preferences: {e}")))?;
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO app_preferences (id, json, updated_at) VALUES ('default', ?1, ?2)",
            params![json, dt_to_iso(Utc::now())],
        )?;
        Ok(())
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

    pub fn list_llm_credentials(
        &self,
    ) -> AppResult<Vec<(String, String, String, String)>> {
        let conn = self.conn.lock().map_err(|e| AppError::Msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT provider, api_key_encrypted, base_url, model FROM llm_credentials ORDER BY provider",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
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
        conn.execute("DELETE FROM llm_credentials WHERE provider = ?1", params![provider])?;
        Ok(())
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
    Ok((item, classification.unwrap_or(NewsClassificationView {
        symbol: String::new(),
        dimension_code: String::new(),
        dimension_label: String::new(),
        confidence: 0.0,
        method: String::new(),
    })))
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
