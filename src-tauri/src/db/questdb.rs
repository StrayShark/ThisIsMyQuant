//! QuestDB 时序库适配占位（H4）。

use reqwest::Client;

#[derive(Clone)]
pub struct QuestDbAdapter {
    url: String,
    http: Client,
}

impl QuestDbAdapter {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            http: Client::new(),
        }
    }

    pub fn configured(&self) -> bool {
        !self.url.is_empty()
    }

    pub async fn ping(&self) -> bool {
        if !self.configured() {
            return false;
        }
        self.http
            .get(format!("{}/exec?query=SELECT%201", self.url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
