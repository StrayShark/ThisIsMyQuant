//! 行情数据源抽象（AKShare 轮询）。

use async_trait::async_trait;

use crate::adapters::AkshareClient;
use crate::error::AppResult;
use crate::models::Tick;

#[async_trait]
pub trait MarketFeed: Send + Sync {
    fn source_name(&self) -> &'static str;
    async fn fetch_latest_tick(&self, symbol: &str) -> AppResult<Option<Tick>>;
}

pub struct AksharePollFeed {
    client: AkshareClient,
}

impl AksharePollFeed {
    pub fn new(client: AkshareClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MarketFeed for AksharePollFeed {
    fn source_name(&self) -> &'static str {
        "akshare_poll"
    }

    async fn fetch_latest_tick(&self, symbol: &str) -> AppResult<Option<Tick>> {
        self.client.fetch_latest_tick(symbol).await
    }
}

pub fn feed_from_config(_feed_kind: &str, akshare: AkshareClient) -> Box<dyn MarketFeed> {
    Box::new(AksharePollFeed::new(akshare))
}
