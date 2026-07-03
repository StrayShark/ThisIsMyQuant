pub mod env_llm;
pub mod llm_catalog;
pub mod prefs_store;
pub mod user_prefs;

pub use env_llm::{collect_llm_credentials_from_env_files, default_llm_provider_from_env_files};
pub use llm_catalog::{build_provider_config, template, LLM_CATALOG};
pub use prefs_store::{load_user_preferences, preferences_path, save_user_preferences};
pub use user_prefs::UserPreferences;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_path: PathBuf,
    pub akshare_enabled: bool,
    pub akshare_realtime_enabled: bool,
    pub realtime_poll_interval: f64,
    pub jinshi_enabled: bool,
    pub jinshi_api_base: String,
    pub jinshi_rili_api_base: String,
    pub jinshi_rili_app_id: String,
    pub jin10_mcp_token: String,
    pub jin10_mcp_server_url: String,
    pub jin10_mcp_protocol_version: String,
    pub jinshi_cache_ttl: f64,
    pub jinshi_poll_interval: f64,
    pub default_llm_provider: String,
    pub llm_providers: Vec<LlmProviderConfig>,
    /// 定时任务周期内 LLM 分析使用的 trigger（scheduled / tomorrow / short_term 等）
    pub schedule_analysis_trigger: String,
    /// 每日固定时刻对全部 core 品种跑 tomorrow 分析
    pub daily_briefing_enabled: bool,
    pub daily_briefing_hour: u8,
    pub schedule_interval_hours: u64,
    pub schedule_enabled: bool,
    pub liquidity: LiquidityConfig,
    pub news_classify: NewsClassifyConfig,
    pub market_feed: String,
    pub anomaly_enabled: bool,
    pub anomaly_price_pct: f64,
    pub anomaly_window_secs: i64,
    pub anomaly_cooldown_secs: u64,
    pub backfill_days_daily: i64,
    pub backfill_days_minute: i64,
    pub encryption_key: String,
    pub retention_days_klines: i64,
    pub retention_days_ticks: i64,
    pub ticks_enabled: bool,
    pub calendar_reminder_enabled: bool,
    pub calendar_reminder_mins: u64,
    pub questdb_url: String,
    pub database_backend: String,
}

#[derive(Debug, Clone)]
pub struct NewsClassifyConfig {
    pub enabled: bool,
    pub provider: String,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct LiquidityConfig {
    pub min_volume_20d: f64,
    pub min_turnover_20d: f64,
}

#[derive(Debug, Clone)]
pub struct LlmProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

pub fn project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("src-tauri").is_dir() {
        return cwd;
    }
    if cwd.file_name().is_some_and(|n| n == "src-tauri") {
        if let Some(p) = cwd.parent() {
            return p.to_path_buf();
        }
    }
    cwd
}

fn load_env_file(path: &Path) {
    if path.exists() {
        let _ = dotenvy::from_path(path);
        log::info!("loaded env from {}", path.display());
    }
}

fn env_str(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 行情源固定为 AKShare 轮询；若 .env 配置了已废弃的 ctp/simnow 会告警并忽略。
pub fn resolve_market_feed(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if matches!(lower.as_str(), "ctp" | "simnow") {
        log::warn!("MARKET_FEED={raw}: CTP/SimNow 已移除，已自动改用 akshare_poll");
    } else if !lower.is_empty() && lower != "akshare_poll" {
        log::warn!("MARKET_FEED={raw}: 未知行情源，已改用 akshare_poll");
    }
    "akshare_poll".into()
}

impl Config {
    /// 从 .env 加载密钥/基础设施，运营项使用 `UserPreferences` 默认值。
    pub fn load() -> Self {
        Self::load_with_preferences(UserPreferences::default())
    }

    pub fn load_with_preferences(prefs: UserPreferences) -> Self {
        let mut cfg = Self::load_secrets();
        prefs.normalize().apply_to(&mut cfg);
        cfg
    }

    fn load_secrets() -> Self {
        let root = project_root();
        load_env_file(&root.join(".env"));

        let db_url = env_str("DATABASE_URL", "sqlite:///data/quant.db");
        let path_str = db_url.replace("sqlite:///", "");
        let database_path = if Path::new(&path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            root.join(path_str)
        };

        let llm_providers: Vec<LlmProviderConfig> = Vec::new();

        Self {
            database_path,
            akshare_enabled: true,
            akshare_realtime_enabled: true,
            realtime_poll_interval: 5.0,
            jinshi_enabled: true,
            jinshi_api_base: env_str("JINSHI_API_BASE", "https://mp-api.jin10.com"),
            jinshi_rili_api_base: env_str(
                "JINSHI_RILI_API_BASE",
                "https://e0430d16720e4211b5e072c26205c890.z3c.jin10.com",
            ),
            jinshi_rili_app_id: env_str("JINSHI_RILI_APP_ID", "sKKYe29sFuJaeOCJ"),
            jin10_mcp_token: {
                let primary = env_str("JIN10_MCP_TOKEN", "");
                if primary.trim().is_empty() {
                    env_str("JIN10_BEARER_TOKEN", "")
                } else {
                    primary
                }
            },
            jin10_mcp_server_url: env_str("JIN10_MCP_SERVER_URL", "https://mcp.jin10.com/mcp"),
            jin10_mcp_protocol_version: env_str("JIN10_MCP_PROTOCOL_VERSION", "2025-11-25"),
            jinshi_cache_ttl: 300.0,
            jinshi_poll_interval: 300.0,
            default_llm_provider: "doubao".into(),
            llm_providers,
            schedule_analysis_trigger: "scheduled".into(),
            daily_briefing_enabled: true,
            daily_briefing_hour: 17,
            schedule_interval_hours: 6,
            schedule_enabled: true,
            liquidity: LiquidityConfig {
                min_volume_20d: 5000.0,
                min_turnover_20d: 500_000_000.0,
            },
            news_classify: NewsClassifyConfig {
                enabled: true,
                provider: String::new(),
                batch_size: 10,
            },
            market_feed: "akshare_poll".into(),
            anomaly_enabled: true,
            anomaly_price_pct: 1.5,
            anomaly_window_secs: 300,
            anomaly_cooldown_secs: 900,
            backfill_days_daily: 120,
            backfill_days_minute: 5,
            encryption_key: env_str("ENCRYPTION_KEY", ""),
            retention_days_klines: 365,
            retention_days_ticks: 14,
            ticks_enabled: true,
            calendar_reminder_enabled: true,
            calendar_reminder_mins: 30,
            questdb_url: String::new(),
            database_backend: "sqlite".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn akshare_is_default_feed() {
        assert_eq!(resolve_market_feed(""), "akshare_poll");
        assert_eq!(resolve_market_feed("akshare_poll"), "akshare_poll");
    }

    #[test]
    fn deprecated_ctp_feed_falls_back() {
        assert_eq!(resolve_market_feed("ctp"), "akshare_poll");
        assert_eq!(resolve_market_feed("simnow"), "akshare_poll");
    }
}
