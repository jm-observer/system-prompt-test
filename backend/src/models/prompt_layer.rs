use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PromptLayer {
    pub id: String,
    pub project_id: String,
    pub layer_type: String,
    pub target_ref: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLayerRequest {
    pub layer_type: String,
    #[serde(default)]
    pub target_ref: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLayerRequest {
    pub content: Option<String>,
    pub target_ref: Option<String>,
}
