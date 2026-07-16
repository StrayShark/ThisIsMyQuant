pub mod ai_context;
mod analysis;
pub mod anomaly;
pub mod calendar_filter;
pub mod dimensions;
pub mod followup;
pub mod fundamentals;
pub mod indicator;
mod kline;
pub mod kline_agg;
pub mod liquidity;
pub mod news_classifier;
pub mod news_llm_classifier;
pub mod report_parse;
pub mod sectors;
pub mod sim_account;
pub mod sim_contract;
pub mod sim_matching;
pub mod sim_risk;
pub mod stock_factors;
pub mod stock_paper;

pub use analysis::{
    build_context, render_prompt, summarize_context, PROMPT_VERSION, SYSTEM_PROMPT,
};
pub use followup::{facts_from_dimension_summary, render_followup_prompt, FOLLOWUP_SYSTEM_PROMPT};
pub use kline::KlineAggregator;
pub use report_parse::{collect_news_ids, parse_llm_report};
