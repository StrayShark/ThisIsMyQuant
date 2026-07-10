use crate::error::AppResult;
use crate::models::{StockBar, StockFinancialMetric, StockValuationSnapshot};

/// 股票因子计算（A3 阶段实现基础技术/估值/财务因子）

pub struct StockFactorInputs {
    pub bars: Vec<StockBar>,
    pub financial: Option<StockFinancialMetric>,
    pub valuation: Option<StockValuationSnapshot>,
}

#[derive(Debug, Clone, Default)]
pub struct StockFactorScores {
    pub momentum: Option<f64>,
    pub quality: Option<f64>,
    pub valuation: Option<f64>,
    pub growth: Option<f64>,
    pub volatility: Option<f64>,
    pub liquidity: Option<f64>,
    pub capital_flow: Option<f64>,
    pub score: Option<f64>,
}

pub fn compute_factors(inputs: &StockFactorInputs) -> AppResult<StockFactorScores> {
    let mut scores = StockFactorScores::default();

    scores.momentum = compute_momentum(&inputs.bars);
    scores.volatility = compute_volatility(&inputs.bars);
    scores.liquidity = compute_liquidity(&inputs.bars);
    scores.quality = compute_quality(inputs.financial.as_ref());
    scores.growth = compute_growth(inputs.financial.as_ref());
    scores.valuation = compute_valuation(inputs.valuation.as_ref());

    // 简单等权合成（P1 可扩展为可配置权重）
    let values: Vec<f64> = vec![
        scores.momentum,
        scores.quality,
        scores.valuation,
        scores.growth,
        scores.volatility,
        scores.liquidity,
    ]
    .into_iter()
    .flatten()
    .collect();
    if !values.is_empty() {
        scores.score = Some(values.iter().sum::<f64>() / values.len() as f64);
    }

    Ok(scores)
}

fn compute_momentum(bars: &[StockBar]) -> Option<f64> {
    if bars.len() < 20 {
        return None;
    }
    let latest = bars.last()?.close?;
    let prev = bars[bars.len() - 20].close?;
    if prev == 0.0 {
        return None;
    }
    Some(((latest - prev) / prev) * 100.0)
}

fn compute_volatility(bars: &[StockBar]) -> Option<f64> {
    if bars.len() < 20 {
        return None;
    }
    let returns: Vec<f64> = bars
        .windows(2)
        .filter_map(|w| {
            let prev = w[0].close?;
            let curr = w[1].close?;
            if prev == 0.0 {
                None
            } else {
                Some((curr - prev) / prev)
            }
        })
        .collect();
    if returns.len() < 2 {
        return None;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
    Some(variance.sqrt() * 100.0)
}

fn compute_liquidity(bars: &[StockBar]) -> Option<f64> {
    let recent = bars
        .iter()
        .rev()
        .take(20)
        .filter_map(|b| b.amount)
        .sum::<f64>();
    if recent <= 0.0 {
        return None;
    }
    // 归一化：日成交额 1 亿为 50 分
    Some((recent / 20.0 / 1e8).tanh() * 50.0)
}

fn compute_quality(fin: Option<&StockFinancialMetric>) -> Option<f64> {
    let fin = fin?;
    let roe = fin.roe?;
    // ROE 10% 为 50 分
    Some((roe / 10.0 * 50.0).clamp(0.0, 100.0))
}

fn compute_growth(fin: Option<&StockFinancialMetric>) -> Option<f64> {
    let fin = fin?;
    let np_yoy = fin.net_profit_yoy?;
    // 净利润同比 20% 为 50 分
    Some((np_yoy / 20.0 * 50.0).clamp(0.0, 100.0))
}

fn compute_valuation(val: Option<&StockValuationSnapshot>) -> Option<f64> {
    let val = val?;
    let pe = val.pe_ttm?;
    if pe <= 0.0 {
        return None;
    }
    // PE 越低分越高：PE 5 为 100 分，PE 50 为 0 分
    let score = (50.0 - pe) / 45.0 * 100.0;
    Some(score.clamp(0.0, 100.0))
}

/// 股票筛选条件（A3 阶段支持基础字段过滤）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StockScreenerCriteria {
    pub min_pe_ttm: Option<f64>,
    pub max_pe_ttm: Option<f64>,
    pub min_pb: Option<f64>,
    pub max_pb: Option<f64>,
    pub min_roe: Option<f64>,
    pub min_market_cap: Option<f64>,
    pub max_market_cap: Option<f64>,
    pub industries: Option<Vec<String>>,
}

pub fn matches_criteria(
    criteria: &StockScreenerCriteria,
    _bar: Option<&StockBar>,
    fin: Option<&StockFinancialMetric>,
    val: Option<&StockValuationSnapshot>,
) -> bool {
    if let Some(pe) = criteria.min_pe_ttm {
        if val.and_then(|v| v.pe_ttm).map_or(true, |v| v < pe) {
            return false;
        }
    }
    if let Some(pe) = criteria.max_pe_ttm {
        if val.and_then(|v| v.pe_ttm).map_or(false, |v| v > pe) {
            return false;
        }
    }
    if let Some(pb) = criteria.min_pb {
        if val.and_then(|v| v.pb).map_or(true, |v| v < pb) {
            return false;
        }
    }
    if let Some(pb) = criteria.max_pb {
        if val.and_then(|v| v.pb).map_or(false, |v| v > pb) {
            return false;
        }
    }
    if let Some(roe) = criteria.min_roe {
        if fin.and_then(|f| f.roe).map_or(true, |v| v < roe) {
            return false;
        }
    }
    if let Some(cap) = criteria.min_market_cap {
        if val.and_then(|v| v.market_cap).map_or(true, |v| v < cap) {
            return false;
        }
    }
    if let Some(cap) = criteria.max_market_cap {
        if val.and_then(|v| v.market_cap).map_or(false, |v| v > cap) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bar(close: f64) -> StockBar {
        StockBar {
            ts_code: "600000.SH".to_string(),
            trade_date: "20260101".to_string(),
            open: Some(close * 0.99),
            high: Some(close * 1.01),
            low: Some(close * 0.98),
            close: Some(close),
            pre_close: Some(close * 0.99),
            pct_chg: Some(1.0),
            volume: Some(1e6),
            amount: Some(1e8),
            turnover_rate: Some(0.01),
            adj_factor: None,
            adjustment: "none".to_string(),
            source: "test".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn momentum_computation() {
        let bars: Vec<StockBar> = (0..25).map(|i| sample_bar(10.0 + i as f64 * 0.1)).collect();
        let inputs = StockFactorInputs {
            bars,
            financial: None,
            valuation: None,
        };
        let scores = compute_factors(&inputs).unwrap();
        assert!(scores.momentum.is_some());
        assert!(scores.score.is_some());
    }

    #[test]
    fn screener_matches_pe() {
        let criteria = StockScreenerCriteria {
            min_pe_ttm: Some(5.0),
            max_pe_ttm: Some(15.0),
            min_pb: None,
            max_pb: None,
            min_roe: None,
            min_market_cap: None,
            max_market_cap: None,
            industries: None,
        };
        let val = StockValuationSnapshot {
            ts_code: "600000.SH".to_string(),
            trade_date: "20260102".to_string(),
            pe_ttm: Some(10.0),
            pb: Some(0.5),
            ps_ttm: None,
            dividend_yield: None,
            market_cap: Some(1e11),
            float_market_cap: None,
            pe_percentile: None,
            pb_percentile: None,
            source: "test".to_string(),
            updated_at: "2026-01-02T00:00:00Z".to_string(),
        };
        assert!(matches_criteria(&criteria, None, None, Some(&val)));

        let val2 = StockValuationSnapshot {
            pe_ttm: Some(20.0),
            ..val.clone()
        };
        assert!(!matches_criteria(&criteria, None, None, Some(&val2)));
    }
}
