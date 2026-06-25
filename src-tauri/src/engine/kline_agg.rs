use chrono::{DateTime, NaiveDate, Utc};

use crate::models::{KLine, parse_dt};

/// 将小时/分钟 K 线按自然日聚合为日 K。
pub fn aggregate_to_daily(klines: &[KLine], symbol: &str) -> Vec<KLine> {
    let mut by_day: std::collections::BTreeMap<NaiveDate, KLine> = std::collections::BTreeMap::new();
    for k in klines {
        let Some(ts) = parse_dt(&k.start_time) else {
            continue;
        };
        let day = ts.date_naive();
        by_day
            .entry(day)
            .and_modify(|bar| {
                bar.high = bar.high.max(k.high);
                bar.low = bar.low.min(k.low);
                bar.close = k.close;
                bar.volume += k.volume;
                bar.turnover += k.turnover;
            })
            .or_insert_with(|| KLine {
                symbol: symbol.to_string(),
                interval: "1d".into(),
                open: k.open,
                high: k.high,
                low: k.low,
                close: k.close,
                volume: k.volume,
                turnover: k.turnover,
                start_time: day.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc3339(),
            });
    }
    by_day.into_values().collect()
}

/// 合并两组日 K，同日期取较新来源（后者覆盖）。
pub fn merge_daily(mut base: Vec<KLine>, supplement: Vec<KLine>) -> Vec<KLine> {
    let mut map: std::collections::BTreeMap<String, KLine> = std::collections::BTreeMap::new();
    for k in base.drain(..) {
        let key = day_key(&k.start_time);
        map.insert(key, k);
    }
    for k in supplement {
        let key = day_key(&k.start_time);
        map.insert(key, k);
    }
    map.into_values().collect()
}

fn day_key(start_time: &str) -> String {
    parse_dt(start_time)
        .map(|t| t.date_naive().to_string())
        .unwrap_or_else(|| start_time.to_string())
}

pub fn latest_bar_time(klines: &[KLine]) -> Option<DateTime<Utc>> {
    klines
        .iter()
        .filter_map(|k| parse_dt(&k.start_time))
        .max()
}
