//! 异动检测：滑动窗口涨跌幅 + 冷却防抖。

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;

use crate::models::Tick;

#[derive(Clone, Debug)]
pub struct AnomalyConfig {
    pub enabled: bool,
    /// 窗口内涨跌幅阈值（百分比，如 1.5 表示 1.5%）
    pub price_pct_threshold: f64,
    /// 滑动窗口秒数
    pub window_secs: i64,
    /// 同品种触发冷却秒数
    pub cooldown_secs: u64,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            price_pct_threshold: 1.5,
            window_secs: 300,
            cooldown_secs: 900,
        }
    }
}

struct SymbolState {
    samples: Vec<(i64, f64)>,
    last_trigger_ts: i64,
}

pub struct AnomalyDetector {
    config: Mutex<AnomalyConfig>,
    states: Mutex<HashMap<String, SymbolState>>,
}

impl AnomalyDetector {
    pub fn new(config: AnomalyConfig) -> Self {
        Self {
            config: Mutex::new(config),
            states: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_config(&self, config: AnomalyConfig) {
        if let Ok(mut c) = self.config.lock() {
            *c = config;
        }
    }

    pub fn config(&self) -> AnomalyConfig {
        self.config.lock().map(|c| c.clone()).unwrap_or_default()
    }

    /// 记录 Tick 并返回异动原因（若触发）。
    pub fn on_tick(&self, tick: &Tick) -> Option<String> {
        let config = self.config.lock().ok()?.clone();
        if !config.enabled {
            return None;
        }
        let now = Utc::now().timestamp();
        let sym = tick.symbol.to_lowercase();
        let price = tick.last_price;
        if price <= 0.0 {
            return None;
        }

        let mut states = self.states.lock().ok()?;
        let state = states.entry(sym.clone()).or_insert_with(|| SymbolState {
            samples: Vec::new(),
            last_trigger_ts: 0,
        });

        state.samples.push((now, price));
        let cutoff = now - config.window_secs;
        state.samples.retain(|(ts, _)| *ts >= cutoff);

        if state.samples.len() < 2 {
            return None;
        }

        let oldest_price = state.samples.first()?.1;
        if oldest_price <= 0.0 {
            return None;
        }
        let pct = (price - oldest_price) / oldest_price * 100.0;
        if pct.abs() < config.price_pct_threshold {
            return None;
        }

        if (now - state.last_trigger_ts) < config.cooldown_secs as i64 {
            return None;
        }

        state.last_trigger_ts = now;
        let window_min = config.window_secs / 60;
        let direction = if pct > 0.0 { "上涨" } else { "下跌" };
        Some(format!(
            "{window_min}分钟内{direction}{:.2}%（{:.2} → {:.2}）",
            pct.abs(),
            oldest_price,
            price
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Tick;

    fn tick(sym: &str, price: f64, ts_offset: i64) -> Tick {
        let ts = Utc::now() + chrono::Duration::seconds(ts_offset);
        Tick {
            symbol: sym.into(),
            last_price: price,
            volume: 100,
            open_interest: 0,
            bid_price: price,
            bid_volume: 0,
            ask_price: price,
            ask_volume: 0,
            timestamp: ts.to_rfc3339(),
        }
    }

    #[test]
    fn triggers_on_large_move() {
        let det = AnomalyDetector::new(AnomalyConfig {
            enabled: true,
            price_pct_threshold: 1.0,
            window_secs: 600,
            cooldown_secs: 0,
        });
        let _ = det.on_tick(&tick("rb0", 100.0, -200));
        let reason = det.on_tick(&tick("rb0", 102.0, 0));
        assert!(reason.is_some(), "expected anomaly, got {:?}", reason);
    }

    #[test]
    fn respects_cooldown() {
        let det = AnomalyDetector::new(AnomalyConfig {
            enabled: true,
            price_pct_threshold: 0.5,
            window_secs: 600,
            cooldown_secs: 3600,
        });
        let _ = det.on_tick(&tick("au0", 500.0, -100));
        assert!(det.on_tick(&tick("au0", 510.0, 0)).is_some());
        assert!(det.on_tick(&tick("au0", 520.0, 1)).is_none());
    }
}
