use chrono::{DateTime, Utc};

use crate::models::{parse_dt, KLine, KlineBarData, KlineUpdateEvent, Tick};

const INTERVALS: [&str; 6] = ["1m", "5m", "15m", "30m", "1h", "1d"];

fn interval_seconds(interval: &str) -> i64 {
    match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "1d" => 86400,
        _ => 60,
    }
}

fn bar_start(ts: DateTime<Utc>, interval: &str) -> DateTime<Utc> {
    if interval == "1d" {
        return bar_start_shanghai_day(ts);
    }
    let seconds = interval_seconds(interval);
    let epoch = ts.timestamp();
    let start = epoch - epoch % seconds;
    DateTime::from_timestamp(start, 0).unwrap_or(ts)
}

/// 按 UTC+8 自然日对齐日 K（与内盘日 K 展示习惯一致）。
fn bar_start_shanghai_day(ts: DateTime<Utc>) -> DateTime<Utc> {
    const OFFSET: i64 = 8 * 3600;
    let adj = ts.timestamp() + OFFSET;
    let day = adj - adj % 86400;
    DateTime::from_timestamp(day - OFFSET, 0).unwrap_or(ts)
}

#[derive(Default)]
pub struct KlineAggregator {
    current: std::collections::HashMap<(String, String), KLine>,
}

impl KlineAggregator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_tick(&mut self, tick: &Tick) -> Vec<KlineUpdateEvent> {
        let ts = parse_dt(&tick.timestamp).unwrap_or_else(Utc::now);
        let mut events = Vec::new();
        for interval in INTERVALS {
            if let Some(ev) = self.update(tick, interval, ts) {
                events.push(ev);
            }
        }
        events
    }

    pub fn current_bar(&self, symbol: &str, interval: &str) -> Option<KLine> {
        self.current
            .get(&(symbol.to_lowercase(), interval.to_string()))
            .cloned()
    }

    fn update(
        &mut self,
        tick: &Tick,
        interval: &str,
        ts: DateTime<Utc>,
    ) -> Option<KlineUpdateEvent> {
        let start = bar_start(ts, interval);
        let key = (tick.symbol.clone(), interval.to_string());
        let current = self.current.get(&key).cloned();

        if current.is_none() || {
            let c = current.as_ref().unwrap();
            parse_dt(&c.start_time).map(|t| start > t).unwrap_or(true)
        } {
            if let Some(prev) = current {
                if let Some(ev) = kline_to_event(&prev) {
                    // final bar emitted implicitly via update path — skip duplicate
                    let _ = ev;
                }
            }
            self.current.insert(
                key.clone(),
                KLine {
                    symbol: tick.symbol.clone(),
                    interval: interval.to_string(),
                    open: tick.last_price,
                    high: tick.last_price,
                    low: tick.last_price,
                    close: tick.last_price,
                    volume: tick.volume,
                    turnover: tick.last_price * tick.volume as f64,
                    start_time: start.to_rfc3339(),
                },
            );
        } else if let Some(cur) = self.current.get_mut(&key) {
            cur.high = cur.high.max(tick.last_price);
            cur.low = cur.low.min(tick.last_price);
            cur.close = tick.last_price;
            cur.volume += tick.volume;
            cur.turnover += tick.last_price * tick.volume as f64;
        }

        self.current.get(&key).and_then(kline_to_event)
    }
}

fn kline_to_event(k: &KLine) -> Option<KlineUpdateEvent> {
    let ts = parse_dt(&k.start_time)?;
    Some(KlineUpdateEvent {
        msg_type: "kline".into(),
        symbol: k.symbol.clone(),
        interval: k.interval.clone(),
        data: KlineBarData {
            t: ts.timestamp_millis(),
            o: k.open,
            h: k.high,
            l: k.low,
            c: k.close,
            v: k.volume,
        },
    })
}
