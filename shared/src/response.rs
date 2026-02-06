use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub message: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            message: message.into(),
            data,
            total: None,
        }
    }

    pub fn with_total(message: impl Into<String>, data: T, total: u64) -> Self {
        Self {
            message: message.into(),
            data,
            total: Some(total),
        }
    }
}
