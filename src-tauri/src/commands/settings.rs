use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::config::LLM_CATALOG;
use crate::crypto::credentials::{encryption_ready, mask_secret};
use crate::engine::sectors;
use crate::models::{
    ApiResponse, AppSettingsView, LlmProviderSetupView, LlmSetupStatus, SaveLlmSetupPayload,
};
use crate::services::{
    hydrate_config_llm, load_llm_providers, restart_runtime_polls, save_llm_provider,
    sync_llm_to_state,
};
use crate::state::AppState;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<AppSettingsView>, String> {
    let schedule_st = state.schedule_status.lock().await;
    let scheduler_running = schedule_st.enabled || schedule_st.cycle_in_progress;
    Ok(ApiResponse::ok(AppSettingsView {
        akshare_enabled: state.config().akshare_enabled,
        akshare_realtime_enabled: state.config().akshare_realtime_enabled,
        realtime_poll_interval: state.config().realtime_poll_interval,
        core_product_count: sectors::core_product_symbols().len(),
        jinshi_enabled: state.config().jinshi_enabled,
        jinshi_poll_interval: state.config().jinshi_poll_interval,
        default_llm_provider: state.config().default_llm_provider.clone(),
        llm_providers: state.llm_snapshot().available_providers(),
        schedule_analysis_trigger: state.config().schedule_analysis_trigger.clone(),
        daily_briefing_enabled: state.config().daily_briefing_enabled,
        daily_briefing_hour: state.config().daily_briefing_hour,
        schedule_interval_hours: state.config().schedule_interval_hours,
        schedule_enabled: state.config().schedule_enabled,
        scheduler_running,
        database_path: state.config().database_path.display().to_string(),
        preferences_path: state.preferences_file_path().display().to_string(),
        news_classify_enabled: state.config().news_classify.enabled,
        news_classify_batch: state.config().news_classify.batch_size,
        market_feed: state.config().market_feed.clone(),
        anomaly_enabled: state.config().anomaly_enabled,
        anomaly_price_pct: state.config().anomaly_price_pct,
        anomaly_window_secs: state.config().anomaly_window_secs,
        anomaly_cooldown_secs: state.config().anomaly_cooldown_secs,
        backfill_days_daily: state.config().backfill_days_daily,
        backfill_days_minute: state.config().backfill_days_minute,
        encryption_configured: encryption_ready(&state.config().encryption_key),
        llm_keys_masked: state
            .config()
            .llm_providers
            .iter()
            .map(|p| (p.name.clone(), mask_secret(&p.api_key)))
            .collect(),
        ollama_configured: state
            .config()
            .llm_providers
            .iter()
            .any(|p| p.name == "ollama"),
        llm_last_errors: state.llm_snapshot().last_errors(),
        ticks_enabled: state.config().ticks_enabled,
        retention_days_klines: state.config().retention_days_klines,
        retention_days_ticks: state.config().retention_days_ticks,
        calendar_reminder_enabled: state.config().calendar_reminder_enabled,
        database_backend: state.config().database_backend.clone(),
        questdb_configured: !state.config().questdb_url.is_empty(),
    }))
}

#[tauri::command]
pub async fn get_llm_setup(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<LlmSetupStatus>, String> {
    let cfg = state.config();
    let configured: std::collections::HashMap<_, _> = cfg
        .llm_providers
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();
    let providers = LLM_CATALOG
        .iter()
        .map(|t| {
            let existing = configured.get(t.name);
            LlmProviderSetupView {
                name: t.name.to_string(),
                label: t.label.to_string(),
                default_base_url: t.default_base_url.to_string(),
                default_model: t.default_model.to_string(),
                key_required: t.key_required,
                configured: existing.is_some(),
                api_key_masked: existing
                    .map(|p| mask_secret(&p.api_key))
                    .unwrap_or_else(|| "（未配置）".into()),
                base_url: existing
                    .map(|p| p.base_url.clone())
                    .unwrap_or_else(|| t.default_base_url.to_string()),
                model: existing
                    .map(|p| p.model.clone())
                    .unwrap_or_else(|| t.default_model.to_string()),
            }
        })
        .collect();
    Ok(ApiResponse::ok(LlmSetupStatus {
        setup_required: cfg.llm_providers.is_empty(),
        default_provider: cfg.default_llm_provider.clone(),
        encryption_ready: encryption_ready(&cfg.encryption_key),
        providers,
    }))
}

#[tauri::command]
pub async fn save_llm_setup(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    payload: SaveLlmSetupPayload,
) -> Result<ApiResponse<LlmSetupStatus>, String> {
    let enc = state.config().encryption_key.clone();
    for cred in &payload.credentials {
        let key = cred.api_key.trim();
        let is_ollama = cred.provider == "ollama";
        if key.is_empty() && !is_ollama {
            continue;
        }
        save_llm_provider(
            &state.db,
            &enc,
            &cred.provider,
            key,
            cred.base_url.as_deref(),
            cred.model.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    }

    let mut cfg = state.config().clone();
    cfg.llm_providers = load_llm_providers(&state.db, &enc).map_err(|e| e.to_string())?;
    if cfg.llm_providers.is_empty() {
        return Ok(ApiResponse::err("请至少配置一个 LLM 提供商"));
    }
    if !payload.default_provider.is_empty() {
        cfg.default_llm_provider = payload.default_provider.clone();
    } else if !cfg
        .llm_providers
        .iter()
        .any(|p| p.name == cfg.default_llm_provider)
    {
        cfg.default_llm_provider = cfg.llm_providers[0].name.clone();
    }

    {
        let mut w = state.config_store.write().map_err(|e| e.to_string())?;
        *w = cfg.clone();
    }
    sync_llm_to_state(&state);
    restart_runtime_polls(&app, &state).await;

    let mut prefs = state.user_prefs();
    prefs.default_llm_provider = cfg.default_llm_provider.clone();
    let prefs = prefs.normalize();
    let path = state.preferences_file_path();
    let _ = crate::config::save_user_preferences(&path, &prefs);
    state.set_user_prefs(prefs);

    get_llm_setup(state).await
}

#[tauri::command]
pub async fn get_user_preferences(
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<crate::config::UserPreferences>, String> {
    Ok(ApiResponse::ok(state.user_prefs().normalize()))
}

#[tauri::command]
pub async fn save_user_preferences(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    prefs: crate::config::UserPreferences,
) -> Result<ApiResponse<crate::config::UserPreferences>, String> {
    let prefs = prefs.normalize();
    let path = state.preferences_file_path();
    crate::config::save_user_preferences(&path, &prefs).map_err(|e| e.to_string())?;
    state.set_user_prefs(prefs.clone());
    crate::services::apply_preferences(&app, &state, prefs.clone()).await;
    Ok(ApiResponse::ok(prefs))
}

#[tauri::command]
pub async fn reload_config(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<ApiResponse<AppSettingsView>, String> {
    let stored = crate::config::load_user_preferences(&state.db, &state.config().database_path)
        .map_err(|e| e.to_string())?;
    state.set_user_prefs(stored.clone());
    let mut new_cfg = crate::config::Config::load_with_preferences(stored);
    hydrate_config_llm(&state.db, &mut new_cfg).map_err(|e| e.to_string())?;
    crate::services::apply_runtime_config(&state, new_cfg).await;
    sync_llm_to_state(&state);
    crate::services::restart_runtime_polls(&app, &state).await;
    get_settings(state).await
}

#[tauri::command]
pub async fn probe_ollama(state: State<'_, Arc<AppState>>) -> Result<ApiResponse<bool>, String> {
    let health = state.llm_snapshot().health().await;
    Ok(ApiResponse::ok(*health.get("ollama").unwrap_or(&false)))
}
