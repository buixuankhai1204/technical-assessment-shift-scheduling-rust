// HTTP client for calling Data Service
use async_trait::async_trait;
use reqwest::Client;

/// Trait for calling Data Service
#[async_trait]
pub trait DataServiceClient: Send + Sync {
    async fn get_resolved_members(
        &self,
        group_id: &str,
    ) -> Result<Vec<serde_json::Value>, reqwest::Error>;
}

/// Implementation of DataServiceClient using reqwest
pub struct ReqwestDataServiceClient {
    client: Client,
    base_url: String,
}

impl ReqwestDataServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl DataServiceClient for ReqwestDataServiceClient {
    async fn get_resolved_members(
        &self,
        group_id: &str,
    ) -> Result<Vec<serde_json::Value>, reqwest::Error> {
        let url = format!(
            "{}/api/v1/groups/{}/resolved-members",
            self.base_url, group_id
        );
        self.client.get(&url).send().await?.json().await
    }
}
