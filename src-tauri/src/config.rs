use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_path: PathBuf,
    pub akshare_enabled: bool,
    pub akshare_realtime_enabled: bool,
    pub realtime_poll_interval: f64,
    pub watchlist: Vec<String>,
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
    pub daily_analysis_cron: String,
    pub realtime_analysis_interval: u64,
    pub liquidity: LiquidityConfig,
    pub news_classify: NewsClassifyConfig,
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

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_str(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn load() -> Self {
        let root = project_root();
        load_env_file(&root.join(".env"));

        let db_url = env_str("DATABASE_URL", "sqlite:///data/quant.db");
        let path_str = db_url.replace("sqlite:///", "");
        let database_path = if Path::new(&path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            root.join(path_str)
        };

        let watchlist_raw = env_str("WATCHLIST", "rb2510,au2512,IF2512");
        let watchlist: Vec<String> = watchlist_raw
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let mut llm_providers = Vec::new();
        let pairs: [(&str, &str, &str, &str); 5] = [
            ("DOUBAO", "DOUBAO_API_KEY", "DOUBAO_BASE_URL", "DOUBAO_MODEL"),
            ("MINIMAX", "MINIMAX_API_KEY", "MINIMAX_BASE_URL", "MINIMAX_MODEL"),
            ("OPENAI", "OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_MODEL"),
            ("DEEPSEEK", "DEEPSEEK_API_KEY", "DEEPSEEK_BASE_URL", "DEEPSEEK_MODEL"),
            ("QWEN", "QWEN_API_KEY", "QWEN_BASE_URL", "QWEN_MODEL"),
        ];
        let model_defaults: [(&str, &str); 5] = [
            ("DOUBAO", "doubao-seed-2.0-pro"),
            ("MINIMAX", "MiniMax-M3"),
            ("OPENAI", "gpt-4o-mini"),
            ("DEEPSEEK", "deepseek-chat"),
            ("QWEN", "qwen-plus"),
        ];
        let url_defaults: [(&str, &str); 5] = [
            ("DOUBAO", "https://ark.cn-beijing.volces.com/api/v3"),
            ("MINIMAX", "https://api.minimaxi.com/v1"),
            ("OPENAI", "https://api.openai.com/v1"),
            ("DEEPSEEK", "https://api.deepseek.com"),
            ("QWEN", "https://dashscope.aliyuncs.com/compatible-mode/v1"),
        ];
        for (i, (name, key_var, url_var, model_var)) in pairs.iter().enumerate() {
            let key = env_str(key_var, "");
            if key.is_empty() {
                continue;
            }
            let (_, default_model) = model_defaults[i];
            let (_, default_url) = url_defaults[i];
            llm_providers.push(LlmProviderConfig {
                name: name.to_lowercase(),
                api_key: key,
                base_url: env_str(url_var, default_url),
                model: env_str(model_var, default_model),
            });
        }

        Self {
            database_path,
            akshare_enabled: env_bool("AKSHARE_ENABLED", true),
            akshare_realtime_enabled: env_bool("AKSHARE_REALTIME_ENABLED", true),
            realtime_poll_interval: env_f64("REALTIME_POLL_INTERVAL", 5.0),
            watchlist,
            jinshi_enabled: env_bool("JINSHI_ENABLED", true),
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
            jinshi_cache_ttl: env_f64("JINSHI_CACHE_TTL", 300.0),
            jinshi_poll_interval: env_f64("JINSHI_POLL_INTERVAL", 300.0),
            default_llm_provider: env_str("DEFAULT_LLM_PROVIDER", "doubao"),
            llm_providers,
            daily_analysis_cron: env_str("DAILY_ANALYSIS_CRON", "0 17"),
            realtime_analysis_interval: env_f64("REALTIME_ANALYSIS_INTERVAL", 300.0) as u64,
            liquidity: LiquidityConfig {
                min_volume_20d: env_f64("LIQUIDITY_MIN_VOLUME_20D", 5000.0),
                min_turnover_20d: env_f64("LIQUIDITY_MIN_TURNOVER_20D", 500_000_000.0),
            },
            news_classify: NewsClassifyConfig {
                enabled: env_bool("NEWS_CLASSIFY_LLM_ENABLED", true),
                provider: env_str("NEWS_CLASSIFY_LLM", ""),
                batch_size: env_f64("NEWS_CLASSIFY_BATCH", 10.0) as usize,
            },
        }
    }
}
