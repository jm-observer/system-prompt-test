use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RunReport {
    pub id: String,
    pub run_id: String,
    pub total_latency_ms: i64,
    pub total_tokens: i64,
    pub estimated_cost_usd: f64,
    pub assertion_passed_count: i64,
    pub assertion_failed_count: i64,
    pub failure_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModelPricing {
    pub id: String,
    pub model_id: String,
    pub input_1k_tokens_usd: f64,
    pub output_1k_tokens_usd: f64,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReportSummary {
    pub run_id: String,
    pub status: String,
    pub model_name: String,
    pub latency_ms: i64,
    pub total_tokens: i64,
    pub cost_usd: f64,
    pub assertions_passed: i64,
    pub assertions_failed: i64,
}
