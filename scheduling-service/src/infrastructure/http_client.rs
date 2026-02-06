use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{ApiResponse, DomainResult, StaffStatus};
use uuid::Uuid;

/// Staff response from data service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: StaffStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Resolved group with its members from data service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedGroupResponse {
    pub group_id: Uuid,
    pub group_name: String,
    pub members: Vec<StaffResponse>,
}


/// Client for calling the data service
pub struct DataServiceClient {
    base_url: String,
    client: reqwest::Client,
}

impl DataServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Get all active staff members in a group (including descendants)
    pub async fn get_group_members(&self, group_id: Uuid) -> DomainResult<Vec<StaffResponse>> {
        let url = format!("{}/api/v1/groups/{}/resolved-members", self.base_url, group_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| shared::DomainError::ExternalServiceError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(shared::DomainError::ExternalServiceError(format!(
                "Data service returned error {}: {}",
                status, error_text
            )));
        }

        let api_response = response
            .json::<ApiResponse<Vec<ResolvedGroupResponse>>>()
            .await
            .map_err(|e| shared::DomainError::ExternalServiceError(e.to_string()))?;

        // Flatten all members from all resolved groups
        let staff_list: Vec<StaffResponse> = api_response
            .data
            .into_iter()
            .flat_map(|group| group.members)
            .collect();

        Ok(staff_list)
    }
}
