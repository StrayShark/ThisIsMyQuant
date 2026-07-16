use crate::config::LlmProviderConfig;

pub struct LlmProviderTemplate {
    pub name: &'static str,
    pub label: &'static str,
    pub default_base_url: &'static str,
    pub default_model: &'static str,
    /// false = 本地 Ollama，无需 API Key
    pub key_required: bool,
}

pub const LLM_CATALOG: &[LlmProviderTemplate] = &[
    LlmProviderTemplate {
        name: "doubao",
        label: "豆包 Doubao",
        default_base_url: "https://ark.cn-beijing.volces.com/api/v3",
        default_model: "doubao-seed-2.0-pro",
        key_required: true,
    },
    LlmProviderTemplate {
        name: "deepseek",
        label: "DeepSeek",
        default_base_url: "https://api.deepseek.com",
        default_model: "deepseek-chat",
        key_required: true,
    },
    LlmProviderTemplate {
        name: "qwen",
        label: "通义千问 Qwen",
        default_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
        default_model: "qwen-plus",
        key_required: true,
    },
    LlmProviderTemplate {
        name: "kimi",
        label: "Kimi / Moonshot",
        default_base_url: "https://api.moonshot.cn/v1",
        default_model: "kimi-k2-0711-preview",
        key_required: true,
    },
    LlmProviderTemplate {
        name: "openai",
        label: "OpenAI",
        default_base_url: "https://api.openai.com/v1",
        default_model: "gpt-4o-mini",
        key_required: true,
    },
    LlmProviderTemplate {
        name: "minimax",
        label: "MiniMax",
        default_base_url: "https://api.minimaxi.com/v1",
        default_model: "MiniMax-M3",
        key_required: true,
    },
    LlmProviderTemplate {
        name: "ollama",
        label: "Ollama（本地）",
        default_base_url: "http://127.0.0.1:11434/v1",
        default_model: "llama3.2",
        key_required: false,
    },
];

pub fn template(name: &str) -> Option<&'static LlmProviderTemplate> {
    LLM_CATALOG.iter().find(|t| t.name == name)
}

pub fn build_provider_config(
    name: &str,
    api_key: &str,
    base_url: Option<&str>,
    model: Option<&str>,
) -> Option<LlmProviderConfig> {
    let t = template(name)?;
    let key = api_key.trim();
    if t.key_required && key.is_empty() {
        return None;
    }
    Some(LlmProviderConfig {
        name: t.name.to_string(),
        api_key: if t.key_required {
            key.to_string()
        } else {
            "ollama".into()
        },
        base_url: base_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(t.default_base_url)
            .trim()
            .trim_end_matches('/')
            .to_string(),
        model: model
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(t.default_model)
            .to_string(),
    })
}
