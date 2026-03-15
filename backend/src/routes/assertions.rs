use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{Assertion, AssertionResult};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateAssertionRequest {
    pub assertion_type: String,
    pub config: String,
}

pub async fn list_assertions(
    State(pool): State<SqlitePool>,
    Path(test_case_id): Path<String>,
) -> Result<Json<Vec<Assertion>>, StatusCode> {
    let assertions = sqlx::query_as::<_, Assertion>(
        "SELECT * FROM assertions WHERE test_case_id = ? ORDER BY created_at DESC",
    )
    .bind(test_case_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(assertions))
}

pub async fn create_assertion(
    State(pool): State<SqlitePool>,
    Path(test_case_id): Path<String>,
    Json(payload): Json<CreateAssertionRequest>,
) -> Result<(StatusCode, Json<Assertion>), StatusCode> {
    let id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO assertions (id, test_case_id, assertion_type, config, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&test_case_id)
    .bind(&payload.assertion_type)
    .bind(&payload.config)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assertion = Assertion {
        id,
        test_case_id,
        assertion_type: payload.assertion_type,
        config: payload.config,
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(assertion)))
}

pub async fn delete_assertion(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM assertions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_assertion_results(
    State(pool): State<SqlitePool>,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<AssertionResult>>, StatusCode> {
    let results = sqlx::query_as::<_, AssertionResult>(
        "SELECT * FROM assertion_results WHERE run_id = ? ORDER BY created_at ASC",
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}
