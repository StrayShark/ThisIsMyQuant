use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

use crate::error::AppResult;
use crate::models::{
    dt_to_iso, StockBar, StockBoard, StockBoardMember, StockFinancialMetric, StockIndexBar,
    StockSymbol, StockValuationSnapshot,
};

const EASTMONEY_CLIST: &str = "https://push2.eastmoney.com/api/qt/clist/get";
const EASTMONEY_STOCK: &str = "https://push2.eastmoney.com/api/qt/stock/get";
const SINA_DAILY_K: &str = "https://quotes.sina.cn/cn/api/quotes.php";

#[async_trait]
pub trait StockDataProvider: Send + Sync {
    async fn list_symbols(&self) -> AppResult<Vec<StockSymbol>>;
    async fn list_index_bars(&self, req: StockBarsRequest) -> AppResult<Vec<StockIndexBar>>;
    async fn list_stock_bars(&self, req: StockBarsRequest) -> AppResult<Vec<StockBar>>;
    async fn list_boards(&self) -> AppResult<Vec<StockBoard>>;
    async fn list_board_members(&self, board_code: &str) -> AppResult<Vec<StockBoardMember>>;
    async fn list_financial_metrics(&self, ts_code: &str) -> AppResult<Vec<StockFinancialMetric>>;
    async fn list_valuation_snapshots(
        &self,
        ts_code: &str,
    ) -> AppResult<Vec<StockValuationSnapshot>>;
}

#[derive(Debug, Clone)]
pub struct StockBarsRequest {
    pub code: String,
    pub adjustment: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: i64,
}

#[derive(Clone)]
pub struct AkshareStockProvider {
    http: Client,
}

impl AkshareStockProvider {
    pub fn new(http: Client) -> Self {
        Self { http }
    }

    pub fn is_ready(&self) -> bool {
        true
    }

    /// 把标准 ts_code 转为东方财富格式：
    /// 600000.SH -> 600000.SH（东财接口直接用 SH/SZ 后缀）
    /// 000001.SZ -> 000001.SZ
    pub fn em_code(ts_code: &str) -> String {
        if ts_code.contains('.') {
            ts_code.to_string()
        } else {
            let exchange = ts_code_exchange(ts_code);
            format!("{}.{}", ts_code, exchange)
        }
    }

    /// 把标准 ts_code 转为新浪格式：sh600000 / sz000001
    pub fn sina_code(ts_code: &str) -> String {
        let parts: Vec<&str> = ts_code.split('.').collect();
        if parts.len() == 2 {
            let prefix = match parts[1] {
                "SH" => "sh",
                "SZ" => "sz",
                "BJ" => "bj",
                _ => "sh",
            };
            format!("{}{}", prefix, parts[0])
        } else {
            format!("sh{}", ts_code)
        }
    }
}

#[async_trait]
impl StockDataProvider for AkshareStockProvider {
    async fn list_symbols(&self) -> AppResult<Vec<StockSymbol>> {
        let url = format!(
            "{EASTMONEY_CLIST}?pn=1&pz=6000&po=1&np=1&fltt=2&invt=2&fid=f12&fs=m:0+t:6,m:0+t:13,m:1+t:2,m:1+t:23,m:0+t:80,m:1+t:80,m:0+t:81+s:204"
        );
        let resp = self.http.get(&url).send().await?;
        let data: Value = resp.json().await?;
        let mut out = Vec::new();
        let now = dt_to_iso(Utc::now());
        if let Some(diffs) = data["data"]["diff"].as_object() {
            for (_, v) in diffs {
                let symbol = v["f12"].as_str().unwrap_or("").to_string();
                let exchange = v["f13"].as_str().unwrap_or("").to_string();
                let exchange = match exchange.as_str() {
                    "0" | "13" => "SZ",
                    "1" | "51" => "SH",
                    "2" | "71" => "SH",
                    _ => "SH",
                };
                let ts_code = format!("{}.{}", symbol, exchange);
                let name = v["f14"].as_str().unwrap_or("").to_string();
                if symbol.is_empty() || name.is_empty() {
                    continue;
                }
                out.push(StockSymbol {
                    ts_code,
                    symbol,
                    name,
                    exchange: exchange.to_string(),
                    market: None,
                    industry: v["f20"].as_str().map(|s| s.to_string()),
                    list_date: None,
                    status: "active".to_string(),
                    source: "eastmoney".to_string(),
                    updated_at: now.clone(),
                });
            }
        }
        Ok(out)
    }

    async fn list_index_bars(&self, req: StockBarsRequest) -> AppResult<Vec<StockIndexBar>> {
        let em = Self::em_code(&req.code);
        let url = format!(
            "{EASTMONEY_STOCK}?secid={em}&fields=f43,f44,f45,f46,f47,f48,f50,f57,f58,f60,f107,f170"
        );
        let resp = self.http.get(&url).send().await?;
        let data: Value = resp.json().await?;
        let now = dt_to_iso(Utc::now());
        let trade_date = today_trade_date();
        let data_obj = data["data"].as_object();
        let mut out = Vec::new();
        if let Some(d) = data_obj {
            out.push(StockIndexBar {
                index_code: req.code.clone(),
                trade_date: trade_date.clone(),
                open: parse_f64_opt(d.get("f46")),
                high: parse_f64_opt(d.get("f44")),
                low: parse_f64_opt(d.get("f45")),
                close: parse_f64_opt(d.get("f43")),
                pct_chg: parse_f64_opt(d.get("f170")),
                volume: parse_f64_opt(d.get("f47")),
                amount: parse_f64_opt(d.get("f48")),
                source: "eastmoney".to_string(),
                updated_at: now,
            });
        }
        Ok(out)
    }

    async fn list_stock_bars(&self, req: StockBarsRequest) -> AppResult<Vec<StockBar>> {
        let sina = Self::sina_code(&req.code);
        let url = format!(
            "{SINA_DAILY_K}?symbol={sina}&source=sina&new_Format=1&start=20150101&end=20991231"
        );
        let resp = self.http.get(&url).send().await?;
        let text = resp.text().await?;
        let mut out = Vec::new();
        let now = dt_to_iso(Utc::now());
        for line in text.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 6 {
                continue;
            }
            let trade_date = parts[0].replace('-', "");
            let open = parts[1].parse::<f64>().ok();
            let high = parts[2].parse::<f64>().ok();
            let low = parts[3].parse::<f64>().ok();
            let close = parts[4].parse::<f64>().ok();
            let volume = parts[5].parse::<f64>().ok();
            out.push(StockBar {
                ts_code: req.code.clone(),
                trade_date,
                open,
                high,
                low,
                close,
                pre_close: None,
                pct_chg: None,
                volume,
                amount: None,
                turnover_rate: None,
                adj_factor: None,
                adjustment: req.adjustment.clone(),
                source: "sina".to_string(),
                updated_at: now.clone(),
            });
        }
        out.sort_by(|a, b| a.trade_date.cmp(&b.trade_date));
        if req.limit > 0 && (out.len() as i64) > req.limit {
            out = out.into_iter().rev().take(req.limit as usize).collect();
            out.reverse();
        }
        Ok(out)
    }

    async fn list_boards(&self) -> AppResult<Vec<StockBoard>> {
        // 东方财富行业/概念板块：这里先用行业板块接口
        let url =
            format!("{EASTMONEY_CLIST}?pn=1&pz=1000&po=1&np=1&fltt=2&invt=2&fid=f12&fs=m:90+t:2");
        let resp = self.http.get(&url).send().await?;
        let data: Value = resp.json().await?;
        let mut out = Vec::new();
        let now = dt_to_iso(Utc::now());
        if let Some(diffs) = data["data"]["diff"].as_object() {
            for (_, v) in diffs {
                let code = v["f12"].as_str().unwrap_or("").to_string();
                let name = v["f14"].as_str().unwrap_or("").to_string();
                if code.is_empty() || name.is_empty() {
                    continue;
                }
                out.push(StockBoard {
                    board_code: code,
                    board_name: name,
                    board_type: "industry".to_string(),
                    source: "eastmoney".to_string(),
                    updated_at: now.clone(),
                });
            }
        }
        Ok(out)
    }

    async fn list_board_members(&self, board_code: &str) -> AppResult<Vec<StockBoardMember>> {
        // 东财板块成分：通过板块代码过滤个股列表
        let url = format!(
            "{EASTMONEY_CLIST}?pn=1&pz=500&po=1&np=1&fltt=2&invt=2&fid=f12&fs=b:{board_code}"
        );
        let resp = self.http.get(&url).send().await?;
        let data: Value = resp.json().await?;
        let mut out = Vec::new();
        let now = dt_to_iso(Utc::now());
        if let Some(diffs) = data["data"]["diff"].as_object() {
            for (_, v) in diffs {
                let symbol = v["f12"].as_str().unwrap_or("").to_string();
                if symbol.is_empty() {
                    continue;
                }
                let exchange = v["f13"].as_str().unwrap_or("").to_string();
                let exchange = match exchange.as_str() {
                    "0" | "13" => "SZ",
                    "1" | "51" => "SH",
                    _ => "SH",
                };
                let ts_code = format!("{}.{}", symbol, exchange);
                out.push(StockBoardMember {
                    board_code: board_code.to_string(),
                    ts_code,
                    weight: None,
                    source: "eastmoney".to_string(),
                    updated_at: now.clone(),
                });
            }
        }
        Ok(out)
    }

    async fn list_financial_metrics(&self, _ts_code: &str) -> AppResult<Vec<StockFinancialMetric>> {
        // P1：后续接入 AKShare/Baostock 财务数据
        Ok(vec![])
    }

    async fn list_valuation_snapshots(
        &self,
        _ts_code: &str,
    ) -> AppResult<Vec<StockValuationSnapshot>> {
        // P1：后续接入 AKShare/Baostock 估值数据
        Ok(vec![])
    }
}

fn ts_code_exchange(symbol: &str) -> &'static str {
    if symbol.starts_with('6') || symbol.starts_with('5') || symbol.starts_with('9') {
        "SH"
    } else if symbol.starts_with("00")
        || symbol.starts_with("30")
        || symbol.starts_with("12")
        || symbol.starts_with("08")
    {
        "SZ"
    } else if symbol.starts_with('4') || symbol.starts_with('8') {
        "BJ"
    } else {
        "SH"
    }
}

fn parse_f64_opt(v: Option<&Value>) -> Option<f64> {
    v.and_then(|x| x.as_f64())
        .or_else(|| {
            v.and_then(|x| x.as_str())
                .and_then(|s| s.parse::<f64>().ok())
        })
        .or_else(|| v.and_then(|x| x.as_i64()).map(|i| i as f64))
}

fn today_trade_date() -> String {
    let now = Utc::now();
    // 简单处理：交易日使用当前 UTC 日期；实际应结合 A 股交易日历
    now.format("%Y%m%d").to_string()
}
