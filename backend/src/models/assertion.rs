use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Assertion {
    pub id: String,
    pub test_case_id: String,
    pub assertion_type: String,
    pub config: String, // JSON string
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AssertionResult {
    pub id: String,
    pub run_id: String,
    pub assertion_id: String,
    pub passed: bool,
    pub evidence: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Fixture {
    pub id: String,
    pub run_id: String,
    pub request_snapshot: String, // JSON
    pub response_snapshot: String, // JSON
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Baseline {
    pub id: String,
    pub test_case_id: String,
    pub run_id: String,
    pub marked_at: String,
}
