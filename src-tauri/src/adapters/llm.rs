use std::collections::HashMap;

use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};

use crate::config::LlmProviderConfig;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct LlmRouter {
    clients: HashMap<String, LlmProviderConfig>,
    default: String,
    http: Client,
}

impl LlmRouter {
    pub fn new(providers: Vec<LlmProviderConfig>, default: String) -> Self {
        let clients: HashMap<String, LlmProviderConfig> = providers
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        let default = if clients.contains_key(&default) {
            default
        } else {
            clients.keys().next().cloned().unwrap_or_default()
        };
        Self {
            clients,
            default,
            http: Client::new(),
        }
    }

    pub fn available_providers(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    pub fn default_provider(&self) -> &str {
        &self.default
    }

    pub async fn health(&self) -> HashMap<String, bool> {
        let mut out = HashMap::new();
        for (name, cfg) in &self.clients {
            out.insert(name.clone(), self.probe(cfg).await);
        }
        out
    }

    async fn probe(&self, cfg: &LlmProviderConfig) -> bool {
        let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
        let body = json!({
            "model": cfg.model,
            "messages": [{"role": "user", "content": "ping"}],
            "max_tokens": 1,
        });
        self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", cfg.api_key))
            .json(&body)
            .send()
            .await
            .map(|r| r.status().is_success() || r.status().as_u16() < 500)
            .unwrap_or(false)
    }

    fn ordered<'a>(&'a self, preferred: Option<&str>) -> Vec<&'a LlmProviderConfig> {
        let first = preferred.unwrap_or(&self.default);
        let mut out = Vec::new();
        if let Some(c) = self.clients.get(first) {
            out.push(c);
        }
        for (name, c) in &self.clients {
            if name != first {
                out.push(c);
            }
        }
        out
    }

    pub async fn complete(
        &self,
        prompt: &str,
        system: &str,
        provider: Option<&str>,
    ) -> AppResult<String> {
        self.complete_with_temperature(prompt, system, provider, 0.3)
            .await
    }

    /// 低温 JSON 输出（资讯分类等结构化任务）。
    pub async fn complete_json(
        &self,
        prompt: &str,
        system: &str,
        provider: Option<&str>,
    ) -> AppResult<String> {
        self.complete_with_temperature(prompt, system, provider, 0.0)
            .await
    }

    pub async fn complete_with_temperature(
        &self,
        prompt: &str,
        system: &str,
        provider: Option<&str>,
        temperature: f32,
    ) -> AppResult<String> {
        let mut last_err = String::new();
        for cfg in self.ordered(provider) {
            match self
                .complete_one(cfg, prompt, system, temperature)
                .await
            {
                Ok(s) => return Ok(s),
                Err(e) => {
                    log::warn!("{} complete failed: {e}", cfg.name);
                    last_err = e.to_string();
                }
            }
        }
        Err(AppError::Msg(format!("all LLM providers failed: {last_err}")))
    }

    async fn complete_one(
        &self,
        cfg: &LlmProviderConfig,
        prompt: &str,
        system: &str,
        temperature: f32,
    ) -> AppResult<String> {
        let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
        let mut messages = Vec::new();
        if !system.is_empty() {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": prompt}));
        let body = json!({
            "model": cfg.model,
            "messages": messages,
            "temperature": temperature,
            "stream": false,
        });
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", cfg.api_key))
            .json(&body)
            .send()
            .await?;
        let data: Value = resp.json().await?;
        data["choices"][0]["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| AppError::Msg("empty LLM response".into()))
    }

    pub async fn stream<F>(
        &self,
        prompt: &str,
        system: &str,
        provider: Option<&str>,
        mut on_token: F,
    ) -> AppResult<()>
    where
        F: FnMut(String) + Send,
    {
        let mut last_err = String::new();
        for cfg in self.ordered(provider) {
            match self.stream_one(cfg, prompt, system, &mut on_token).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    log::warn!("{} stream failed: {e}", cfg.name);
                    last_err = e.to_string();
                }
            }
        }
        Err(AppError::Msg(format!("all LLM providers failed: {last_err}")))
    }

    async fn stream_one<F>(
        &self,
        cfg: &LlmProviderConfig,
        prompt: &str,
        system: &str,
        on_token: &mut F,
    ) -> AppResult<()>
    where
        F: FnMut(String) + Send,
    {
        let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
        let mut messages = Vec::new();
        if !system.is_empty() {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": prompt}));
        let body = json!({
            "model": cfg.model,
            "messages": messages,
            "temperature": 0.3,
            "stream": true,
        });
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", cfg.api_key))
            .json(&body)
            .send()
            .await?;
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();
                if let Some(token) = parse_sse_line(&line) {
                    on_token(token);
                }
            }
        }
        Ok(())
    }
}

fn parse_sse_line(line: &str) -> Option<String> {
    if !line.starts_with("data:") {
        return None;
    }
    let data = line[5..].trim();
    if data == "[DONE]" {
        return None;
    }
    let obj: Value = serde_json::from_str(data).ok()?;
    obj["choices"][0]["delta"]["content"]
        .as_str()
        .map(String::from)
}
