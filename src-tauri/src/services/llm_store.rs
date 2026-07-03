//! LLM 凭据持久化（SQLite + AES，不依赖 .env）。

use crate::config::{build_provider_config, LlmProviderConfig};
use crate::crypto::credentials::{decrypt_value, encrypt_value, encryption_ready};
use crate::db::Database;
use crate::error::{AppError, AppResult};

const ENC_KEY_SECRET: &str = "encryption_key";

pub fn resolve_encryption_key(db: &Database, env_key: &str) -> AppResult<String> {
    if encryption_ready(env_key) {
        return Ok(env_key.trim().to_string());
    }
    db.get_or_create_app_secret(ENC_KEY_SECRET)
}

pub fn load_llm_providers(
    db: &Database,
    encryption_key: &str,
) -> AppResult<Vec<LlmProviderConfig>> {
    let rows = db.list_llm_credentials()?;
    let mut out = Vec::new();
    for (provider, enc_key, base_url, model) in rows {
        let api_key = if enc_key.is_empty() {
            String::new()
        } else {
            decrypt_value(&enc_key, encryption_key)
                .map_err(|e| AppError::Msg(format!("decrypt llm key for {provider}: {e}")))?
        };
        if let Some(cfg) = build_provider_config(&provider, &api_key, Some(&base_url), Some(&model))
        {
            out.push(cfg);
        }
    }
    Ok(out)
}

pub fn save_llm_provider(
    db: &Database,
    encryption_key: &str,
    provider: &str,
    api_key: &str,
    base_url: Option<&str>,
    model: Option<&str>,
) -> AppResult<LlmProviderConfig> {
    let cfg = build_provider_config(provider, api_key, base_url, model)
        .ok_or_else(|| AppError::Msg(format!("invalid or incomplete credential for {provider}")))?;
    let enc = if cfg.api_key.is_empty() {
        String::new()
    } else {
        encrypt_value(&cfg.api_key, encryption_key)?
    };
    db.upsert_llm_credential(&cfg.name, &enc, &cfg.base_url, &cfg.model)?;
    Ok(cfg)
}

pub fn hydrate_config_llm(db: &Database, cfg: &mut crate::config::Config) -> AppResult<()> {
    let enc = resolve_encryption_key(db, &cfg.encryption_key)?;
    cfg.encryption_key = enc.clone();
    cfg.llm_providers = load_llm_providers(db, &enc)?;
    Ok(())
}

pub fn sync_llm_to_state(state: &std::sync::Arc<crate::state::AppState>) {
    let cfg = state.config();
    state.replace_llm(cfg.llm_providers.clone(), cfg.default_llm_provider.clone());
}

/// 本地 debug 构建：DB 无 LLM 凭据时，从 `.env` / `~/global_env/.env` 自动导入。
#[cfg(debug_assertions)]
pub fn maybe_import_llm_from_env_dev(
    db: &Database,
    cfg: &mut crate::config::Config,
) -> AppResult<bool> {
    if !db.list_llm_credentials()?.is_empty() {
        return Ok(false);
    }

    let creds = crate::config::collect_llm_credentials_from_env_files();
    if creds.is_empty() {
        log::info!("dev: no LLM keys in .env or ~/global_env/.env, skip auto-import");
        return Ok(false);
    }

    let enc = resolve_encryption_key(db, &cfg.encryption_key)?;
    cfg.encryption_key = enc.clone();

    for cred in &creds {
        save_llm_provider(
            db,
            &enc,
            &cred.provider,
            &cred.api_key,
            cred.base_url.as_deref(),
            cred.model.as_deref(),
        )?;
        log::info!("dev: imported LLM credential for {}", cred.provider);
    }

    cfg.llm_providers = load_llm_providers(db, &enc)?;
    if let Some(default) = crate::config::default_llm_provider_from_env_files() {
        if cfg.llm_providers.iter().any(|p| p.name == default) {
            cfg.default_llm_provider = default;
        }
    } else if !cfg.llm_providers.is_empty()
        && !cfg
            .llm_providers
            .iter()
            .any(|p| p.name == cfg.default_llm_provider)
    {
        cfg.default_llm_provider = cfg.llm_providers[0].name.clone();
    }

    log::info!(
        "dev: auto-imported {} LLM provider(s) from env files",
        cfg.llm_providers.len()
    );
    Ok(true)
}

#[cfg(not(debug_assertions))]
pub fn maybe_import_llm_from_env_dev(
    _db: &Database,
    _cfg: &mut crate::config::Config,
) -> AppResult<bool> {
    Ok(false)
}
