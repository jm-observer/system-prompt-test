use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub name: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub name: Option<String>,
    pub content: Option<String>,
}
