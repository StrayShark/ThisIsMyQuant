//! 本地调试：从 `.env` / `~/global_env/.env` 读取 LLM Key（不写入运行时 Config，仅用于导入 SQLite）。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::project_root;

#[derive(Debug, Clone)]
pub struct EnvLlmCredential {
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

struct EnvLlmField {
    provider: &'static str,
    api_key: &'static str,
    base_url: &'static str,
    model: &'static str,
}

const ENV_LLM_FIELDS: &[EnvLlmField] = &[
    EnvLlmField {
        provider: "doubao",
        api_key: "DOUBAO_API_KEY",
        base_url: "DOUBAO_BASE_URL",
        model: "DOUBAO_MODEL",
    },
    EnvLlmField {
        provider: "deepseek",
        api_key: "DEEPSEEK_API_KEY",
        base_url: "DEEPSEEK_BASE_URL",
        model: "DEEPSEEK_MODEL",
    },
    EnvLlmField {
        provider: "qwen",
        api_key: "QWEN_API_KEY",
        base_url: "QWEN_BASE_URL",
        model: "QWEN_MODEL",
    },
    EnvLlmField {
        provider: "openai",
        api_key: "OPENAI_API_KEY",
        base_url: "OPENAI_BASE_URL",
        model: "OPENAI_MODEL",
    },
    EnvLlmField {
        provider: "minimax",
        api_key: "MINIMAX_API_KEY",
        base_url: "MINIMAX_BASE_URL",
        model: "MINIMAX_MODEL",
    },
];

pub fn global_env_path() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("global_env").join(".env"))
        .unwrap_or_else(|_| PathBuf::from("~/global_env/.env"))
}

fn parse_env_file(path: &Path) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return out;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'');
        if !key.is_empty() {
            out.insert(key.to_string(), val.to_string());
        }
    }
    out
}

fn env_lookup<'a>(
    project: &'a HashMap<String, String>,
    global: &'a HashMap<String, String>,
    key: &str,
) -> Option<&'a str> {
    project
        .get(key)
        .filter(|v| !v.trim().is_empty())
        .map(String::as_str)
        .or_else(|| {
            global
                .get(key)
                .filter(|v| !v.trim().is_empty())
                .map(String::as_str)
        })
}

/// 合并项目 `.env` 与 `~/global_env/.env`（项目优先，global 补全缺失项）。
pub fn collect_llm_credentials_from_env_files() -> Vec<EnvLlmCredential> {
    let root = project_root();
    let project = parse_env_file(&root.join(".env"));
    let global = parse_env_file(&global_env_path());

    let mut out = Vec::new();
    for field in ENV_LLM_FIELDS {
        let Some(api_key) = env_lookup(&project, &global, field.api_key) else {
            continue;
        };
        out.push(EnvLlmCredential {
            provider: field.provider.to_string(),
            api_key: api_key.to_string(),
            base_url: env_lookup(&project, &global, field.base_url).map(str::to_string),
            model: env_lookup(&project, &global, field.model).map(str::to_string),
        });
    }
    out
}

pub fn default_llm_provider_from_env_files() -> Option<String> {
    let root = project_root();
    let project = parse_env_file(&root.join(".env"));
    let global = parse_env_file(&global_env_path());
    env_lookup(&project, &global, "DEFAULT_LLM_PROVIDER").map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_file_skips_comments_and_empty() {
        let dir = std::env::temp_dir().join(format!("env-llm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        std::fs::write(
            &path,
            "# comment\nDOUBAO_API_KEY=abc\nEMPTY=\nDEEPSEEK_API_KEY=\"xyz\"\n",
        )
        .unwrap();
        let map = parse_env_file(&path);
        assert_eq!(map.get("DOUBAO_API_KEY").map(String::as_str), Some("abc"));
        assert_eq!(map.get("DEEPSEEK_API_KEY").map(String::as_str), Some("xyz"));
        assert_eq!(map.get("EMPTY").map(String::as_str), Some(""));
        let _ = std::fs::remove_dir_all(dir);
    }
}
