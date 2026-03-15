use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::models::{AiModel, CreateModelRequest, UpdateModelRequest};

pub async fn list_models(
    State(pool): State<SqlitePool>,
    Path(provider_id): Path<String>,
) -> Result<Json<Vec<AiModel>>, StatusCode> {
    let models = sqlx::query_as::<_, AiModel>(
        "SELECT * FROM models WHERE provider_id = ? ORDER BY created_at DESC",
    )
    .bind(&provider_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(models))
}

pub async fn create_model(
    State(pool): State<SqlitePool>,
    Path(provider_id): Path<String>,
    Json(payload): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<AiModel>), StatusCode> {
    let id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO models (id, provider_id, model_name, capabilities, is_active, created_at)
         VALUES (?, ?, ?, ?, 1, ?)",
    )
    .bind(&id)
    .bind(&provider_id)
    .bind(&payload.model_name)
    .bind(&payload.capabilities)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let model = AiModel {
        id,
        provider_id,
        model_name: payload.model_name,
        capabilities: payload.capabilities,
        is_active: true,
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(model)))
}

pub async fn update_model(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateModelRequest>,
) -> Result<Json<AiModel>, StatusCode> {
    let existing = sqlx::query_as::<_, AiModel>("SELECT * FROM models WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let model_name = payload.model_name.unwrap_or(existing.model_name);
    let capabilities = payload.capabilities.unwrap_or(existing.capabilities);
    let is_active = payload.is_active.unwrap_or(existing.is_active);

    sqlx::query("UPDATE models SET model_name = ?, capabilities = ?, is_active = ? WHERE id = ?")
        .bind(&model_name)
        .bind(&capabilities)
        .bind(is_active)
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AiModel {
        id,
        provider_id: existing.provider_id,
        model_name,
        capabilities,
        is_active,
        created_at: existing.created_at,
    }))
}

pub async fn delete_model(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM models WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// List all active models across all providers (for run panel model selection)
pub async fn list_all_models(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<AiModel>>, StatusCode> {
    let models = sqlx::query_as::<_, AiModel>(
        "SELECT * FROM models WHERE is_active = 1 ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(models))
}
