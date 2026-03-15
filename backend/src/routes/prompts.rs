use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use ulid::Ulid;

use crate::models::{CreatePromptRequest, Prompt, UpdatePromptRequest};

pub async fn list_prompts(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Prompt>>, StatusCode> {
    let prompts = sqlx::query_as::<_, Prompt>("SELECT * FROM prompts ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(prompts))
}

pub async fn create_prompt(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreatePromptRequest>,
) -> Result<(StatusCode, Json<Prompt>), StatusCode> {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let prompt = Prompt {
        id: id.clone(),
        name: payload.name,
        content: payload.content,
        created_at: now.clone(),
        updated_at: now,
    };

    sqlx::query(
        "INSERT INTO prompts (id, name, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&prompt.id)
    .bind(&prompt.name)
    .bind(&prompt.content)
    .bind(&prompt.created_at)
    .bind(&prompt.updated_at)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(prompt)))
}

pub async fn get_prompt(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Prompt>, StatusCode> {
    let prompt = sqlx::query_as::<_, Prompt>("SELECT * FROM prompts WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(prompt))
}

pub async fn update_prompt(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePromptRequest>,
) -> Result<Json<Prompt>, StatusCode> {
    let existing = sqlx::query_as::<_, Prompt>("SELECT * FROM prompts WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let name = payload.name.unwrap_or(existing.name);
    let content = payload.content.unwrap_or(existing.content);
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE prompts SET name = ?, content = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(&content)
        .bind(&now)
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated = Prompt {
        id,
        name,
        content,
        created_at: existing.created_at,
        updated_at: now,
    };
    Ok(Json(updated))
}

pub async fn delete_prompt(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM prompts WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}
