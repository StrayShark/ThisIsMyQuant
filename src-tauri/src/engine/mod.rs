mod analysis;
pub mod dimensions;
pub mod indicator;
mod kline;
pub mod kline_agg;
pub mod liquidity;
pub mod news_classifier;
pub mod news_llm_classifier;
pub mod followup;
pub mod report_parse;
pub mod sectors;

pub use analysis::{build_context, render_prompt, summarize_context, SYSTEM_PROMPT};
pub use followup::{facts_from_dimension_summary, render_followup_prompt, FOLLOWUP_SYSTEM_PROMPT};
pub use report_parse::{collect_news_ids, parse_llm_report};
pub use kline::KlineAggregator;
