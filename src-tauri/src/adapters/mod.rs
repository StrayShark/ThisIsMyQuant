mod akshare;
mod feed;
mod jinshi;
mod jinshi_calendar;
mod llm;
mod overseas_futures;

pub use akshare::AkshareClient;
pub use feed::{feed_from_config, AksharePollFeed, MarketFeed};
pub use jinshi::JinshiClient;
pub use jinshi_calendar::{
    default_calendar_range, default_calendar_range_from_today, fetch_calendar_range,
    CalendarFetchOptions, DEFAULT_CALENDAR_DAYS_AHEAD,
};
pub use llm::LlmRouter;
pub use overseas_futures::{fetch_overseas_quote, list_overseas_symbols};
