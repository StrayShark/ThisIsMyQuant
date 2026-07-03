//! CSV 导出 K 线与报告。

use crate::models::{AnalysisReport, KLine};

pub fn klines_to_csv(klines: &[KLine]) -> String {
    let mut out = String::from("symbol,interval,start_time,open,high,low,close,volume,turnover\n");
    for k in klines {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&k.symbol),
            csv_escape(&k.interval),
            csv_escape(&k.start_time),
            k.open,
            k.high,
            k.low,
            k.close,
            k.volume,
            k.turnover
        ));
    }
    out
}

pub fn reports_to_csv(reports: &[AnalysisReport]) -> String {
    let mut out = String::from(
        "id,symbol,trigger,provider,prompt_version,created_at,context_summary,content\n",
    );
    for r in reports {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            csv_escape(&r.id),
            csv_escape(&r.symbol),
            csv_escape(&r.trigger),
            csv_escape(&r.provider),
            csv_escape(&r.prompt_version),
            csv_escape(&r.created_at),
            csv_escape(&r.context_summary),
            csv_escape(&r.content)
        ));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn parse_klines_csv(
    csv: &str,
    default_symbol: &str,
    interval: &str,
) -> Result<Vec<KLine>, String> {
    let mut lines = csv.lines();
    let header = lines.next().ok_or("empty csv")?;
    if !header.to_lowercase().contains("start_time") {
        return Err("missing start_time column".into());
    }
    let mut out = Vec::new();
    for (i, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = split_csv_line(line);
        if cols.len() < 7 {
            return Err(format!("line {}: expected >=7 columns", i + 2));
        }
        let sym = if cols.len() >= 9 && !cols[0].is_empty() {
            cols[0].clone()
        } else {
            default_symbol.to_string()
        };
        let iv = if cols.len() >= 9 && !cols[1].is_empty() {
            cols[1].clone()
        } else {
            interval.to_string()
        };
        let (start, open, high, low, close, volume, turnover) = if cols.len() >= 9 {
            (
                cols[2].clone(),
                cols[3]
                    .parse()
                    .map_err(|_| format!("line {} bad open", i + 2))?,
                cols[4]
                    .parse()
                    .map_err(|_| format!("line {} bad high", i + 2))?,
                cols[5]
                    .parse()
                    .map_err(|_| format!("line {} bad low", i + 2))?,
                cols[6]
                    .parse()
                    .map_err(|_| format!("line {} bad close", i + 2))?,
                cols[7].parse().unwrap_or(0),
                cols[8].parse().unwrap_or(0.0),
            )
        } else {
            (
                cols[0].clone(),
                cols[1]
                    .parse()
                    .map_err(|_| format!("line {} bad open", i + 2))?,
                cols[2]
                    .parse()
                    .map_err(|_| format!("line {} bad high", i + 2))?,
                cols[3]
                    .parse()
                    .map_err(|_| format!("line {} bad low", i + 2))?,
                cols[4]
                    .parse()
                    .map_err(|_| format!("line {} bad close", i + 2))?,
                cols[5].parse().unwrap_or(0),
                cols[6].parse().unwrap_or(0.0),
            )
        };
        out.push(KLine {
            symbol: sym.to_lowercase(),
            interval: iv,
            start_time: start,
            open,
            high,
            low,
            close,
            volume,
            turnover,
        });
    }
    Ok(out)
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut cols = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' if in_quotes => in_quotes = false,
            '"' => in_quotes = true,
            ',' if !in_quotes => {
                cols.push(cur.clone());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    cols.push(cur);
    cols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_kline_csv() {
        let csv = "start_time,open,high,low,close,volume,turnover\n2024-01-01T00:00:00Z,100,101,99,100.5,1000,0\n";
        let rows = parse_klines_csv(csv, "rb0", "1d").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol, "rb0");
    }
}
