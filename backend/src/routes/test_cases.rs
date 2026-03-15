use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::models::{CreateTestCaseRequest, TestCase, UpdateTestCaseRequest};

pub async fn list_test_cases(
    State(pool): State<SqlitePool>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<TestCase>>, StatusCode> {
    let cases = sqlx::query_as::<_, TestCase>(
        "SELECT * FROM test_cases WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(&project_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(cases))
}

pub async fn create_test_case(
    State(pool): State<SqlitePool>,
    Path(project_id): Path<String>,
    Json(payload): Json<CreateTestCaseRequest>,
) -> Result<(StatusCode, Json<TestCase>), StatusCode> {
    let id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO test_cases (id, project_id, name, user_message, config, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&project_id)
    .bind(&payload.name)
    .bind(&payload.user_message)
    .bind(&payload.config)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let case = TestCase {
        id,
        project_id,
        name: payload.name,
        user_message: payload.user_message,
        config: payload.config,
        created_at: now.clone(),
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(case)))
}

pub async fn get_test_case(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<TestCase>, StatusCode> {
    let case = sqlx::query_as::<_, TestCase>("SELECT * FROM test_cases WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(case))
}

pub async fn update_test_case(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTestCaseRequest>,
) -> Result<Json<TestCase>, StatusCode> {
    let existing = sqlx::query_as::<_, TestCase>("SELECT * FROM test_cases WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let name = payload.name.unwrap_or(existing.name);
    let user_message = payload.user_message.unwrap_or(existing.user_message);
    let config = payload.config.unwrap_or(existing.config);
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE test_cases SET name = ?, user_message = ?, config = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&user_message)
    .bind(&config)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TestCase {
        id,
        project_id: existing.project_id,
        name,
        user_message,
        config,
        created_at: existing.created_at,
        updated_at: now,
    }))
}

pub async fn delete_test_case(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM test_cases WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}
