use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{RunReport, ReportSummary, AuditLog, ModelPricing};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub prompt_version_id: Option<String>,
}

pub async fn get_run_report(
    State(pool): State<SqlitePool>,
    Path(run_id): Path<String>,
) -> Result<Json<RunReport>, StatusCode> {
    let report = sqlx::query_as::<_, RunReport>(
        "SELECT * FROM run_reports WHERE run_id = ?"
    )
    .bind(run_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(report))
}

pub async fn list_reports_summary(
    State(pool): State<SqlitePool>,
    Query(_query): Query<ReportQuery>,
) -> Result<Json<Vec<ReportSummary>>, StatusCode> {
    // This is a simplified version of trend analysis summary
    let summaries = sqlx::query_as::<_, ReportSummary>(
        r#"
        SELECT 
            r.id as run_id,
            r.status,
            m.model_name,
            COALESCE(rr.total_latency_ms, 0) as latency_ms,
            COALESCE(rr.total_tokens, 0) as total_tokens,
            COALESCE(rr.estimated_cost_usd, 0.0) as cost_usd,
            COALESCE(rr.assertion_passed_count, 0) as assertions_passed,
            COALESCE(rr.assertion_failed_count, 0) as assertions_failed
        FROM runs r
        JOIN models m ON r.model_id = m.id
        LEFT JOIN run_reports rr ON r.id = rr.run_id
        ORDER BY r.created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(summaries))
}

pub async fn list_audit_logs(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<AuditLog>>, StatusCode> {
    let logs = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}

pub async fn get_model_pricing(
    State(pool): State<SqlitePool>,
    Path(model_id): Path<String>,
) -> Result<Json<ModelPricing>, StatusCode> {
    let pricing = sqlx::query_as::<_, ModelPricing>(
        "SELECT * FROM model_pricing WHERE model_id = ?"
    )
    .bind(model_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(pricing))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePricingRequest {
    pub input_1k_tokens_usd: f64,
    pub output_1k_tokens_usd: f64,
}

pub async fn update_model_pricing(
    State(pool): State<SqlitePool>,
    Path(model_id): Path<String>,
    Json(payload): Json<UpdatePricingRequest>,
) -> Result<StatusCode, StatusCode> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = ulid::Ulid::new().to_string();

    sqlx::query(
        r#"
        INSERT INTO model_pricing (id, model_id, input_1k_tokens_usd, output_1k_tokens_usd, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(model_id) DO UPDATE SET
            input_1k_tokens_usd = excluded.input_1k_tokens_usd,
            output_1k_tokens_usd = excluded.output_1k_tokens_usd,
            updated_at = excluded.updated_at
        "#
    )
    .bind(&id)
    .bind(&model_id)
    .bind(payload.input_1k_tokens_usd)
    .bind(payload.output_1k_tokens_usd)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
