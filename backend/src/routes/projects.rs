use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use ulid::Ulid;

use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};

pub async fn list_projects(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let projects =
        sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(projects))
}

pub async fn create_project(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let project = Project {
        id: id.clone(),
        name: payload.name,
        description: payload.description,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    // Use a transaction to ensure project + layers are created atomically
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query(
        "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&project.id)
    .bind(&project.name)
    .bind(&project.description)
    .bind(&project.created_at)
    .bind(&project.updated_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert project: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    for layer_type in &["global", "project", "provider", "model"] {
        let layer_id = Ulid::new().to_string();
        let version_id = Ulid::new().to_string();
        sqlx::query(
            "INSERT INTO prompt_layers (id, project_id, layer_type, target_ref, content, created_at, updated_at) VALUES (?, ?, ?, '', '', ?, ?)",
        )
        .bind(&layer_id)
        .bind(&id)
        .bind(layer_type)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert layer '{}': {}", layer_type, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        sqlx::query(
            "INSERT INTO prompt_versions (id, layer_id, version, content, created_at) VALUES (?, ?, 1, '', ?)",
        )
        .bind(&version_id)
        .bind(&layer_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert initial version for layer '{}': {}", layer_type, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn get_project(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(project))
}

pub async fn update_project(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    let existing = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let name = payload.name.unwrap_or(existing.name);
    let description = payload.description.unwrap_or(existing.description);
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE projects SET name = ?, description = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(&description)
        .bind(&now)
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Project {
        id,
        name,
        description,
        created_at: existing.created_at,
        updated_at: now,
    }))
}

pub async fn delete_project(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}
