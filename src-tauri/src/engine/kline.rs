use chrono::{DateTime, Utc};

use crate::models::{KLine, KlineBarData, KlineUpdateEvent, Tick, parse_dt};

const INTERVALS: [&str; 5] = ["1m", "5m", "15m", "30m", "1h"];

fn interval_seconds(interval: &str) -> i64 {
    match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        _ => 60,
    }
}

fn bar_start(ts: DateTime<Utc>, seconds: i64) -> DateTime<Utc> {
    let epoch = ts.timestamp();
    let start = epoch - epoch % seconds;
    DateTime::from_timestamp(start, 0).unwrap_or(ts)
}

pub struct KlineAggregator {
    current: std::collections::HashMap<(String, String), KLine>,
}

impl KlineAggregator {
    pub fn new() -> Self {
        Self {
            current: std::collections::HashMap::new(),
        }
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

    fn update(&mut self, tick: &Tick, interval: &str, ts: DateTime<Utc>) -> Option<KlineUpdateEvent> {
        let seconds = interval_seconds(interval);
        let start = bar_start(ts, seconds);
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
