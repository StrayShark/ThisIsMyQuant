use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::models::{Contract, KLine, Tick, dt_to_iso, parse_dt};

const SINA_BASE: &str = "https://stock2.finance.sina.com.cn/futures/api/json.php";

#[derive(Clone)]
pub struct AkshareClient {
    http: Client,
    connected: bool,
}

impl AkshareClient {
    pub fn new(http: Client) -> Self {
        Self {
            http,
            connected: true,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.connected
    }

    pub fn akshare_symbol(symbol: &str) -> String {
        let re = Regex::new(r"(?i)^([a-z]+)").unwrap();
        if let Some(caps) = re.captures(symbol.trim()) {
            format!("{}0", caps[1].to_uppercase())
        } else {
            symbol.to_uppercase()
        }
    }

    pub fn interval_to_period(interval: &str) -> Option<&'static str> {
        match interval {
            "1m" => Some("1"),
            "5m" => Some("5"),
            "15m" => Some("15"),
            "30m" => Some("30"),
            "1h" => Some("60"),
            "1d" => None,
            _ => None,
        }
    }

    pub async fn get_contracts(&self) -> AppResult<Vec<Contract>> {
        Ok(crate::engine::sectors::all_contracts())
    }

    pub async fn get_history(
        &self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AppResult<Vec<KLine>> {
        let ak_sym = Self::akshare_symbol(symbol);
        let sym = symbol.to_lowercase();
        if interval == "1d" {
            self.fetch_daily(&ak_sym, &sym, start, end).await
        } else if let Some(period) = Self::interval_to_period(interval) {
            self.fetch_minute(&ak_sym, &sym, interval, period, start, end)
                .await
        } else {
            Err(AppError::Msg(format!("unsupported interval: {interval}")))
        }
    }

    pub async fn fetch_latest_tick(&self, symbol: &str) -> AppResult<Option<Tick>> {
        let ak_sym = Self::akshare_symbol(symbol);
        let url = format!(
            "{SINA_BASE}/InnerFuturesNewService.getFewMinLine?symbol={ak_sym}&type=1"
        );
        let resp = self.http.get(&url).send().await?;
        let rows: Vec<Value> = resp.json().await?;
        let last = rows.last().ok_or_else(|| AppError::Msg("empty minute data".into()))?;
        let ts = parse_ts_field(last.get("d").and_then(|v| v.as_str()))?;
        let price = parse_f64(last.get("c"))?;
        let vol = parse_i64(last.get("v")).unwrap_or(1).max(1);
        let oi = parse_i64(last.get("p")).unwrap_or(0);
        Ok(Some(Tick {
            symbol: symbol.to_lowercase(),
            last_price: price,
            volume: vol,
            open_interest: oi,
            bid_price: 0.0,
            bid_volume: 0,
            ask_price: 0.0,
            ask_volume: 0,
            timestamp: dt_to_iso(ts),
        }))
    }

    async fn fetch_daily(
        &self,
        ak_sym: &str,
        sym: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AppResult<Vec<KLine>> {
        let url = format!(
            "{SINA_BASE}/IndexService.getInnerFuturesDailyKLine?symbol={ak_sym}"
        );
        let resp = self.http.get(&url).send().await?;
        let rows: Vec<Vec<Value>> = resp.json().await.unwrap_or_default();
        let mut out = Vec::new();
        for row in rows {
            if row.len() < 6 {
                continue;
            }
            let ts = parse_ts_field(row[0].as_str())?;
            if ts < start || ts > end {
                continue;
            }
            out.push(KLine {
                symbol: sym.to_string(),
                interval: "1d".into(),
                open: parse_f64(Some(&row[1]))?,
                high: parse_f64(Some(&row[2]))?,
                low: parse_f64(Some(&row[3]))?,
                close: parse_f64(Some(&row[4]))?,
                volume: parse_i64(Some(&row[5])).unwrap_or(0),
                turnover: 0.0,
                start_time: dt_to_iso(ts),
            });
        }

        // 新浪 RB0 日线可能停更，用 60 分钟线聚合补最近数据
        let stale = crate::engine::kline_agg::latest_bar_time(&out)
            .map(|t| (end - t).num_days() > 7)
            .unwrap_or(true);
        if stale {
            let supplement_start = crate::engine::kline_agg::latest_bar_time(&out)
                .unwrap_or(start)
                .max(start - chrono::Duration::days(1));
            if let Ok(hourly) = self
                .fetch_minute(ak_sym, sym, "1h", "60", supplement_start, end)
                .await
            {
                let aggregated = crate::engine::kline_agg::aggregate_to_daily(&hourly, sym);
                out = crate::engine::kline_agg::merge_daily(out, aggregated);
            }
        }

        out.retain(|k| {
            parse_dt(&k.start_time)
                .map(|t| t >= start && t <= end)
                .unwrap_or(false)
        });
        Ok(out)
    }

    async fn fetch_minute(
        &self,
        ak_sym: &str,
        sym: &str,
        interval: &str,
        period: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AppResult<Vec<KLine>> {
        let url = format!(
            "{SINA_BASE}/InnerFuturesNewService.getFewMinLine?symbol={ak_sym}&type={period}"
        );
        let resp = self.http.get(&url).send().await?;
        let rows: Vec<Value> = resp.json().await?;
        let mut out = Vec::new();
        for row in rows {
            let ts = parse_ts_field(row.get("d").and_then(|v| v.as_str()))?;
            if ts < start || ts > end {
                continue;
            }
            out.push(KLine {
                symbol: sym.to_string(),
                interval: interval.to_string(),
                open: parse_f64(row.get("o"))?,
                high: parse_f64(row.get("h"))?,
                low: parse_f64(row.get("l"))?,
                close: parse_f64(row.get("c"))?,
                volume: parse_i64(row.get("v")).unwrap_or(0),
                turnover: 0.0,
                start_time: dt_to_iso(ts),
            });
        }
        Ok(out)
    }
}

fn parse_ts_field(raw: Option<&str>) -> AppResult<DateTime<Utc>> {
    let s = raw.ok_or_else(|| AppError::Msg("missing timestamp".into()))?;
    parse_dt(s).ok_or_else(|| AppError::Msg(format!("bad timestamp: {s}")))
}

fn parse_f64(v: Option<&Value>) -> AppResult<f64> {
    match v {
        Some(Value::Number(n)) => n
            .as_f64()
            .ok_or_else(|| AppError::Msg("bad number".into())),
        Some(Value::String(s)) => s
            .parse()
            .map_err(|_| AppError::Msg(format!("bad float: {s}"))),
        _ => Err(AppError::Msg("missing float".into())),
    }
}

fn parse_i64(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(s)) => s.parse().ok(),
        _ => None,
    }
}
