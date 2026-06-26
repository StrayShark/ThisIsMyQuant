use serde::{Deserialize, Serialize};

use super::Config;

/// 应用内可配置项（默认值内置，持久化于 SQLite，不依赖 .env）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UserPreferences {
    pub watchlist: Vec<String>,
    pub schedule_enabled: bool,
    pub schedule_interval_hours: u64,
    pub schedule_analysis_trigger: String,
    pub daily_briefing_enabled: bool,
    pub daily_briefing_hour: u8,
    pub akshare_enabled: bool,
    pub akshare_realtime_enabled: bool,
    pub realtime_poll_interval: f64,
    pub jinshi_enabled: bool,
    pub jinshi_poll_interval: f64,
    pub default_llm_provider: String,
    pub news_classify_enabled: bool,
    pub news_classify_batch: usize,
    pub anomaly_enabled: bool,
    pub anomaly_price_pct: f64,
    pub anomaly_window_secs: i64,
    pub anomaly_cooldown_secs: u64,
    pub backfill_days_daily: i64,
    pub backfill_days_minute: i64,
    pub ticks_enabled: bool,
    pub retention_days_klines: i64,
    pub retention_days_ticks: i64,
    pub calendar_reminder_enabled: bool,
    pub calendar_reminder_mins: u64,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            watchlist: vec![
                "rb2510".into(),
                "au2512".into(),
                "if2512".into(),
            ],
            schedule_enabled: true,
            schedule_interval_hours: 6,
            schedule_analysis_trigger: "scheduled".into(),
            daily_briefing_enabled: true,
            daily_briefing_hour: 17,
            akshare_enabled: true,
            akshare_realtime_enabled: true,
            realtime_poll_interval: 5.0,
            jinshi_enabled: true,
            jinshi_poll_interval: 300.0,
            default_llm_provider: "doubao".into(),
            news_classify_enabled: true,
            news_classify_batch: 10,
            anomaly_enabled: true,
            anomaly_price_pct: 1.5,
            anomaly_window_secs: 300,
            anomaly_cooldown_secs: 900,
            backfill_days_daily: 120,
            backfill_days_minute: 5,
            ticks_enabled: true,
            retention_days_klines: 365,
            retention_days_ticks: 14,
            calendar_reminder_enabled: true,
            calendar_reminder_mins: 30,
        }
    }
}

impl UserPreferences {
    pub fn normalize(mut self) -> Self {
        self.watchlist = self
            .watchlist
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        self.schedule_interval_hours = self.schedule_interval_hours.clamp(1, 168);
        self.schedule_analysis_trigger = normalize_analysis_trigger(&self.schedule_analysis_trigger);
        self.daily_briefing_hour = self.daily_briefing_hour.min(23);
        self.realtime_poll_interval = self.realtime_poll_interval.clamp(1.0, 3600.0);
        self.jinshi_poll_interval = self.jinshi_poll_interval.clamp(30.0, 86400.0);
        self.news_classify_batch = self.news_classify_batch.clamp(1, 50);
        self.anomaly_price_pct = self.anomaly_price_pct.clamp(0.1, 50.0);
        self.anomaly_window_secs = self.anomaly_window_secs.clamp(60, 86400);
        self.anomaly_cooldown_secs = self.anomaly_cooldown_secs.clamp(60, 86400);
        self
    }

    pub fn apply_to(&self, cfg: &mut Config) {
        cfg.watchlist = self.watchlist.clone();
        cfg.schedule_enabled = self.schedule_enabled;
        cfg.schedule_interval_hours = self.schedule_interval_hours;
        cfg.schedule_analysis_trigger = self.schedule_analysis_trigger.clone();
        cfg.daily_briefing_enabled = self.daily_briefing_enabled;
        cfg.daily_briefing_hour = self.daily_briefing_hour;
        cfg.akshare_enabled = self.akshare_enabled;
        cfg.akshare_realtime_enabled = self.akshare_realtime_enabled;
        cfg.realtime_poll_interval = self.realtime_poll_interval;
        cfg.jinshi_enabled = self.jinshi_enabled;
        cfg.jinshi_poll_interval = self.jinshi_poll_interval;
        cfg.default_llm_provider = self.default_llm_provider.clone();
        cfg.news_classify.enabled = self.news_classify_enabled;
        cfg.news_classify.batch_size = self.news_classify_batch;
        cfg.anomaly_enabled = self.anomaly_enabled;
        cfg.anomaly_price_pct = self.anomaly_price_pct;
        cfg.anomaly_window_secs = self.anomaly_window_secs;
        cfg.anomaly_cooldown_secs = self.anomaly_cooldown_secs;
        cfg.backfill_days_daily = self.backfill_days_daily;
        cfg.backfill_days_minute = self.backfill_days_minute;
        cfg.ticks_enabled = self.ticks_enabled;
        cfg.retention_days_klines = self.retention_days_klines;
        cfg.retention_days_ticks = self.retention_days_ticks;
        cfg.calendar_reminder_enabled = self.calendar_reminder_enabled;
        cfg.calendar_reminder_mins = self.calendar_reminder_mins;
    }

    pub fn from_config(cfg: &Config) -> Self {
        Self {
            watchlist: cfg.watchlist.clone(),
            schedule_enabled: cfg.schedule_enabled,
            schedule_interval_hours: cfg.schedule_interval_hours,
            schedule_analysis_trigger: cfg.schedule_analysis_trigger.clone(),
            daily_briefing_enabled: cfg.daily_briefing_enabled,
            daily_briefing_hour: cfg.daily_briefing_hour,
            akshare_enabled: cfg.akshare_enabled,
            akshare_realtime_enabled: cfg.akshare_realtime_enabled,
            realtime_poll_interval: cfg.realtime_poll_interval,
            jinshi_enabled: cfg.jinshi_enabled,
            jinshi_poll_interval: cfg.jinshi_poll_interval,
            default_llm_provider: cfg.default_llm_provider.clone(),
            news_classify_enabled: cfg.news_classify.enabled,
            news_classify_batch: cfg.news_classify.batch_size,
            anomaly_enabled: cfg.anomaly_enabled,
            anomaly_price_pct: cfg.anomaly_price_pct,
            anomaly_window_secs: cfg.anomaly_window_secs,
            anomaly_cooldown_secs: cfg.anomaly_cooldown_secs,
            backfill_days_daily: cfg.backfill_days_daily,
            backfill_days_minute: cfg.backfill_days_minute,
            ticks_enabled: cfg.ticks_enabled,
            retention_days_klines: cfg.retention_days_klines,
            retention_days_ticks: cfg.retention_days_ticks,
            calendar_reminder_enabled: cfg.calendar_reminder_enabled,
            calendar_reminder_mins: cfg.calendar_reminder_mins,
        }
    }
}

fn normalize_analysis_trigger(raw: &str) -> String {
    match raw.trim() {
        "scheduled" | "tomorrow" | "short_term" | "manual" | "daily" | "realtime" | "anomaly" => {
            raw.trim().to_string()
        }
        _ => "scheduled".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_watchlist_not_empty() {
        assert_eq!(UserPreferences::default().watchlist.len(), 3);
    }
}
