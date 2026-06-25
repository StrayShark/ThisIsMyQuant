use std::collections::HashMap;

use crate::models::KLine;

pub fn summary(klines: &[KLine]) -> HashMap<String, f64> {
    if klines.is_empty() {
        return HashMap::new();
    }
    let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
    let volumes: Vec<f64> = klines.iter().map(|k| k.volume as f64).collect();
    let highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
    let lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
    let last = *closes.last().unwrap();
    let first = closes[0];
    let ma5 = ma(&closes, 5);
    let ma20 = ma(&closes, 20);
    let ma60 = ma(&closes, 60);
    let (dif, dea, hist) = macd(&closes);
    HashMap::from([
        ("last".into(), last),
        (
            "change_pct".into(),
            if first != 0.0 {
                (last - first) / first * 100.0
            } else {
                0.0
            },
        ),
        ("ma5".into(), *ma5.last().unwrap_or(&0.0)),
        ("ma20".into(), *ma20.last().unwrap_or(&0.0)),
        ("ma60".into(), *ma60.last().unwrap_or(&0.0)),
        ("macd_dif".into(), *dif.last().unwrap_or(&0.0)),
        ("macd_dea".into(), *dea.last().unwrap_or(&0.0)),
        ("macd_hist".into(), *hist.last().unwrap_or(&0.0)),
        (
            "avg_volume".into(),
            if volumes.len() >= 20 {
                volumes[volumes.len() - 20..].iter().sum::<f64>() / 20.0
            } else {
                volumes.iter().sum::<f64>() / volumes.len().max(1) as f64
            },
        ),
        (
            "max_high".into(),
            highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ),
        (
            "min_low".into(),
            lows.iter().cloned().fold(f64::INFINITY, f64::min),
        ),
    ])
}

fn ma(data: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![0.0; data.len()];
    if data.len() < period {
        return out;
    }
    for i in period - 1..data.len() {
        let sum: f64 = data[i + 1 - period..=i].iter().sum();
        out[i] = sum / period as f64;
    }
    out
}

fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() {
        return vec![];
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![0.0; data.len()];
    out[0] = data[0];
    for i in 1..data.len() {
        out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1];
    }
    out
}

fn macd(data: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    if data.is_empty() {
        return (vec![], vec![], vec![]);
    }
    let ema_fast = ema(data, 12);
    let ema_slow = ema(data, 26);
    let dif: Vec<f64> = ema_fast
        .iter()
        .zip(ema_slow.iter())
        .map(|(f, s)| f - s)
        .collect();
    let dea = ema(&dif, 9);
    let hist: Vec<f64> = dif.iter().zip(dea.iter()).map(|(d, e)| 2.0 * (d - e)).collect();
    (dif, dea, hist)
}
