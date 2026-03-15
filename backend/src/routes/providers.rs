use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

use crate::crypto;
use crate::models::{CreateProviderRequest, Provider, ProviderResponse, UpdateProviderRequest};

pub async fn list_providers(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ProviderResponse>>, StatusCode> {
    let providers = sqlx::query_as::<_, Provider>("SELECT * FROM providers ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<ProviderResponse> = providers
        .iter()
        .map(|p| {
            let plain_key = crypto::decrypt(&p.encrypted_api_key).unwrap_or_default();
            let masked = crypto::mask_api_key(&plain_key);
            p.to_response(masked)
        })
        .collect();

    Ok(Json(responses))
}

pub async fn create_provider(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProviderResponse>), StatusCode> {
    let id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let encrypted_key = crypto::encrypt(&payload.api_key);

    sqlx::query(
        "INSERT INTO providers (id, name, api_type, base_url, encrypted_api_key, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.api_type)
    .bind(&payload.base_url)
    .bind(&encrypted_key)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let masked = crypto::mask_api_key(&payload.api_key);
    let response = ProviderResponse {
        id,
        name: payload.name,
        api_type: payload.api_type,
        base_url: payload.base_url,
        api_key_masked: masked,
        created_at: now.clone(),
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_provider(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let provider = sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plain_key = crypto::decrypt(&provider.encrypted_api_key).unwrap_or_default();
    let masked = crypto::mask_api_key(&plain_key);
    Ok(Json(provider.to_response(masked)))
}

pub async fn update_provider(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProviderRequest>,
) -> Result<Json<ProviderResponse>, StatusCode> {
    let existing = sqlx::query_as::<_, Provider>("SELECT * FROM providers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let name = payload.name.unwrap_or(existing.name);
    let base_url = payload.base_url.unwrap_or(existing.base_url);
    let now = chrono::Utc::now().to_rfc3339();

    let (encrypted_key, masked) = if let Some(new_key) = &payload.api_key {
        (crypto::encrypt(new_key), crypto::mask_api_key(new_key))
    } else {
        let plain = crypto::decrypt(&existing.encrypted_api_key).unwrap_or_default();
        (existing.encrypted_api_key, crypto::mask_api_key(&plain))
    };

    sqlx::query(
        "UPDATE providers SET name = ?, base_url = ?, encrypted_api_key = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&base_url)
    .bind(&encrypted_key)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ProviderResponse {
        id,
        name,
        api_type: existing.api_type,
        base_url,
        api_key_masked: masked,
        created_at: existing.created_at,
        updated_at: now,
    }))
}

pub async fn delete_provider(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM providers WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}
