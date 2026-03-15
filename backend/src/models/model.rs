use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiModel {
    pub id: String,
    pub provider_id: String,
    pub model_name: String,
    pub capabilities: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub model_name: String,
    #[serde(default = "default_capabilities")]
    pub capabilities: String,
}

fn default_capabilities() -> String {
    "{}".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelRequest {
    pub model_name: Option<String>,
    pub capabilities: Option<String>,
    pub is_active: Option<bool>,
}
