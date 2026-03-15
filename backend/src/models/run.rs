use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Run {
    pub id: String,
    pub test_case_id: String,
    pub model_id: String,
    pub status: String,
    pub system_prompt: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RunResult {
    pub id: String,
    pub run_id: String,
    pub response_text: String,
    pub token_usage: String,
    pub latency_ms: Option<i64>,
    pub raw_response: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct RunWithResult {
    #[serde(flatten)]
    pub run: Run,
    pub result: Option<RunResult>,
}

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub model_ids: Vec<String>,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}
