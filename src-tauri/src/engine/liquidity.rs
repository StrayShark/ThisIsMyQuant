//! 品种流动性评分与 tier 判定。

use chrono::Utc;

use crate::config::LiquidityConfig;
use crate::engine::sectors::LiquidityTier;
use crate::models::{KLine, LiquiditySnapshot};

/// 20 日日均成交量与成交额。
pub fn compute_metrics(klines: &[KLine]) -> (f64, f64) {
    if klines.is_empty() {
        return (0.0, 0.0);
    }
    let take = klines.len().min(20);
    let recent = &klines[klines.len() - take..];
    let n = recent.len() as f64;
    let volume_20d = recent.iter().map(|k| k.volume as f64).sum::<f64>() / n;
    let turnover_20d = recent.iter().map(|k| k.turnover).sum::<f64>() / n;
    (volume_20d, turnover_20d)
}

/// 0–1 综合流动性得分（展示用）。
pub fn normalized_score(volume_20d: f64, turnover_20d: f64) -> f64 {
    let v = (volume_20d / 50_000.0).clamp(0.0, 1.0);
    let t = (turnover_20d / 1_000_000_000.0).clamp(0.0, 1.0);
    0.5 * v + 0.5 * t
}

/// 结合静态默认 tier 与量化指标得出最终 tier。
pub fn resolve_tier(
    volume_20d: f64,
    turnover_20d: f64,
    default: LiquidityTier,
    cfg: &LiquidityConfig,
) -> LiquidityTier {
    if default == LiquidityTier::Excluded {
        return LiquidityTier::Excluded;
    }

    // 无数据时保留静态默认
    if volume_20d <= 0.0 && turnover_20d <= 0.0 {
        return default;
    }

    // 长期无成交
    if volume_20d < 1.0 && turnover_20d < 1.0 {
        return LiquidityTier::Excluded;
    }

    let passes = volume_20d >= cfg.min_volume_20d || turnover_20d >= cfg.min_turnover_20d;
    if passes {
        return LiquidityTier::Core;
    }

    match default {
        LiquidityTier::Core => LiquidityTier::Watch,
        LiquidityTier::Watch => LiquidityTier::Watch,
        LiquidityTier::Excluded => LiquidityTier::Excluded,
    }
}

pub fn build_snapshot(
    symbol: &str,
    klines: &[KLine],
    default: LiquidityTier,
    cfg: &LiquidityConfig,
) -> LiquiditySnapshot {
    let (volume_20d, turnover_20d) = compute_metrics(klines);
    let tier = resolve_tier(volume_20d, turnover_20d, default, cfg);
    LiquiditySnapshot {
        symbol: symbol.to_uppercase(),
        volume_20d,
        turnover_20d,
        score: normalized_score(volume_20d, turnover_20d),
        tier: tier.as_str().into(),
        scored_at: Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::KLine;

    fn cfg() -> LiquidityConfig {
        LiquidityConfig {
            min_volume_20d: 5000.0,
            min_turnover_20d: 500_000_000.0,
        }
    }

    fn kline(vol: i64, turnover: f64) -> KLine {
        KLine {
            symbol: "rb0".into(),
            interval: "1d".into(),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: vol,
            turnover,
            start_time: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn high_volume_is_core() {
        let klines: Vec<_> = (0..20).map(|_| kline(20_000, 800_000_000.0)).collect();
        let snap = build_snapshot("RB0", &klines, LiquidityTier::Core, &cfg());
        assert_eq!(snap.tier, "core");
    }

    #[test]
    fn low_volume_demotes_core_to_watch() {
        let klines: Vec<_> = (0..20).map(|_| kline(100, 1_000_000.0)).collect();
        let snap = build_snapshot("AP0", &klines, LiquidityTier::Core, &cfg());
        assert_eq!(snap.tier, "watch");
    }
}
