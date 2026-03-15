use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PromptVersion {
    pub id: String,
    pub layer_id: String,
    pub version: i64,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub version: i64,
}

#[derive(Debug, Serialize)]
pub struct DiffResult {
    pub v1: i64,
    pub v2: i64,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Serialize)]
pub struct DiffChange {
    pub tag: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MergedPromptResponse {
    pub merged_prompt: String,
    pub layers: Vec<MergedLayerInfo>,
}

#[derive(Debug, Serialize)]
pub struct MergedLayerInfo {
    pub layer_type: String,
    pub has_content: bool,
}

#[derive(Debug, Deserialize)]
pub struct MergeQuery {
    #[serde(default)]
    pub variables: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    pub v1: i64,
    pub v2: i64,
}

pub type Variables = HashMap<String, String>;
