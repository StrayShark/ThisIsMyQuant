mod akshare;
mod jinshi;
mod jinshi_calendar;
mod llm;

pub use akshare::AkshareClient;
pub use jinshi::JinshiClient;
pub use jinshi_calendar::{
    fetch_calendar_range, CalendarFetchOptions, default_calendar_range,
    default_calendar_range_from_today, DEFAULT_CALENDAR_DAYS_AHEAD,
};
pub use llm::LlmRouter;
