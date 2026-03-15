use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TestCase {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub user_message: String,
    pub config: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestCaseRequest {
    pub name: String,
    pub user_message: String,
    #[serde(default = "default_config")]
    pub config: String,
}

fn default_config() -> String {
    "{}".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestCaseRequest {
    pub name: Option<String>,
    pub user_message: Option<String>,
    pub config: Option<String>,
}
