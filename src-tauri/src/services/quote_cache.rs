//! 实时行情快照：昨收、最新价、日涨跌幅、进行中日 K。

use std::collections::HashMap;

use chrono::{Duration, Utc};

use crate::adapters::AkshareClient;
use crate::db::Database;
use crate::models::{KLine, QuoteCacheStatus, RealtimeQuote, Tick};

#[derive(Default)]
pub struct QuoteCache {
    quotes: HashMap<String, RealtimeQuote>,
    prev_close: HashMap<String, f64>,
}

impl QuoteCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, symbol: &str) -> Option<&RealtimeQuote> {
        self.quotes.get(&symbol.to_lowercase())
    }

    pub fn snapshot(&self, symbols: Option<&[String]>) -> Vec<RealtimeQuote> {
        match symbols {
            None => self.quotes.values().cloned().collect(),
            Some(syms) => syms.iter().filter_map(|s| self.get(s).cloned()).collect(),
        }
    }

    pub fn forming_daily(&self, symbol: &str) -> Option<KLine> {
        self.get(symbol).and_then(|q| q.forming_daily.clone())
    }

    pub fn prev_close(&self, symbol: &str) -> Option<f64> {
        self.prev_close.get(&symbol.to_lowercase()).copied()
    }

    pub fn set_prev_close(&mut self, symbol: &str, close: f64) {
        if close > 0.0 {
            self.prev_close.insert(symbol.to_lowercase(), close);
        }
    }

    pub fn update_from_tick(&mut self, tick: &Tick, prev_close: f64, forming_daily: Option<KLine>) {
        let sym = tick.symbol.to_lowercase();
        if prev_close > 0.0 {
            self.prev_close.insert(sym.clone(), prev_close);
        }
        let change_pct = if prev_close > 0.0 {
            (tick.last_price - prev_close) / prev_close * 100.0
        } else {
            0.0
        };
        self.quotes.insert(
            sym.clone(),
            RealtimeQuote {
                symbol: sym,
                last_price: tick.last_price,
                prev_close,
                change_pct,
                timestamp: tick.timestamp.clone(),
                forming_daily,
            },
        );
    }

    pub fn status(&self, stale_after_secs: i64) -> QuoteCacheStatus {
        let now = Utc::now();
        let mut newest: Option<chrono::DateTime<Utc>> = None;
        let mut max_age_secs: Option<i64> = None;
        let mut stale_count = 0usize;

        for q in self.quotes.values() {
            let Some(ts) = crate::models::parse_dt(&q.timestamp) else {
                stale_count += 1;
                continue;
            };
            newest = Some(newest.map(|n| n.max(ts)).unwrap_or(ts));
            let age = (now - ts).num_seconds().max(0);
            max_age_secs = Some(max_age_secs.map(|m| m.max(age)).unwrap_or(age));
            if age > stale_after_secs {
                stale_count += 1;
            }
        }

        QuoteCacheStatus {
            quote_count: self.quotes.len(),
            stale_count,
            stale_after_secs,
            newest_timestamp: newest.map(|t| t.to_rfc3339()),
            max_age_secs,
        }
    }
}

pub fn prev_close_from_klines(klines: &[KLine]) -> Option<f64> {
    if klines.is_empty() {
        return None;
    }
    let last = klines.last()?;
    if klines.len() == 1 {
        return Some(last.close);
    }

    // Sina daily history usually ends at the latest completed trading day.  Only
    // skip the last bar when it is already today's forming daily bar.
    let last_day = trading_day_key(&last.start_time)?;
    let today = trading_day_key(&Utc::now().to_rfc3339()).unwrap_or(last_day);
    if last_day >= today {
        Some(klines[klines.len() - 2].close)
    } else {
        Some(last.close)
    }
}

pub fn merge_forming_daily(klines: &mut Vec<KLine>, forming: &KLine) {
    if let Some(last) = klines.last_mut() {
        if same_trading_day(&last.start_time, &forming.start_time) {
            *last = forming.clone();
            return;
        }
    }
    klines.push(forming.clone());
}

pub fn same_trading_day(a: &str, b: &str) -> bool {
    trading_day_key(a) == trading_day_key(b)
}

pub fn trading_day_key(iso: &str) -> Option<i64> {
    let dt = crate::models::parse_dt(iso)?;
    let sh = dt.timestamp() + 8 * 3600;
    Some(sh - sh % 86400)
}

pub fn is_daily_klines_stale(klines: &[KLine]) -> bool {
    let Some(last) = klines.last() else {
        return true;
    };
    let Some(last_day) = trading_day_key(&last.start_time) else {
        return true;
    };
    let today = trading_day_key(&chrono::Utc::now().to_rfc3339()).unwrap_or(last_day);
    last_day < today
}

pub async fn resolve_prev_close(
    db: &Database,
    akshare: &AkshareClient,
    symbol: &str,
) -> Option<f64> {
    let end = Utc::now();
    let start = end - Duration::days(14);
    let sym = symbol.to_lowercase();

    if let Ok(klines) = db.get_klines(&sym, "1d", start, end, 10) {
        if !klines.is_empty() {
            return prev_close_from_klines(&klines);
        }
    }

    match akshare.get_history(&sym, "1d", start, end).await {
        Ok(klines) if !klines.is_empty() => {
            let _ = db.save_klines(&klines);
            prev_close_from_klines(&klines)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kline(close: f64, days_from_now: i64) -> KLine {
        KLine {
            symbol: "rb0".into(),
            interval: "1d".into(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 1,
            turnover: 0.0,
            start_time: (Utc::now() + Duration::days(days_from_now)).to_rfc3339(),
        }
    }

    #[test]
    fn prev_close_uses_latest_completed_daily_bar() {
        let klines = vec![kline(100.0, -2), kline(110.0, -1)];
        assert_eq!(prev_close_from_klines(&klines), Some(110.0));
    }

    #[test]
    fn prev_close_skips_today_forming_daily_bar() {
        let klines = vec![kline(100.0, -1), kline(110.0, 0)];
        assert_eq!(prev_close_from_klines(&klines), Some(100.0));
    }
}
