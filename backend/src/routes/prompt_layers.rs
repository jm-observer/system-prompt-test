use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use similar::{ChangeTag, TextDiff};
use sqlx::SqlitePool;
use ulid::Ulid;

use crate::models::*;

pub async fn list_layers(
    State(pool): State<SqlitePool>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<PromptLayer>>, StatusCode> {
    let layers = sqlx::query_as::<_, PromptLayer>(
        "SELECT * FROM prompt_layers WHERE project_id = ? ORDER BY \
         CASE layer_type WHEN 'global' THEN 0 WHEN 'project' THEN 1 \
         WHEN 'provider' THEN 2 WHEN 'model' THEN 3 END",
    )
    .bind(&project_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(layers))
}

pub async fn create_layer(
    State(pool): State<SqlitePool>,
    Path(project_id): Path<String>,
    Json(payload): Json<CreateLayerRequest>,
) -> Result<(StatusCode, Json<PromptLayer>), StatusCode> {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let layer = PromptLayer {
        id: id.clone(),
        project_id,
        layer_type: payload.layer_type,
        target_ref: payload.target_ref,
        content: payload.content.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    sqlx::query(
        "INSERT INTO prompt_layers (id, project_id, layer_type, target_ref, content, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&layer.id)
    .bind(&layer.project_id)
    .bind(&layer.layer_type)
    .bind(&layer.target_ref)
    .bind(&layer.content)
    .bind(&layer.created_at)
    .bind(&layer.updated_at)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create initial version
    let version_id = Ulid::new().to_string();
    sqlx::query(
        "INSERT INTO prompt_versions (id, layer_id, version, content, created_at) VALUES (?, ?, 1, ?, ?)",
    )
    .bind(&version_id)
    .bind(&id)
    .bind(&payload.content)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(layer)))
}

pub async fn get_layer(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<PromptLayer>, StatusCode> {
    let layer = sqlx::query_as::<_, PromptLayer>("SELECT * FROM prompt_layers WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(layer))
}

pub async fn update_layer(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateLayerRequest>,
) -> Result<Json<PromptLayer>, StatusCode> {
    let existing =
        sqlx::query_as::<_, PromptLayer>("SELECT * FROM prompt_layers WHERE id = ?")
            .bind(&id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

    let content_changed = payload.content.is_some()
        && payload.content.as_deref() != Some(&existing.content);
    let content = payload.content.unwrap_or(existing.content.clone());
    let target_ref = payload.target_ref.unwrap_or(existing.target_ref);
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE prompt_layers SET content = ?, target_ref = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&content)
    .bind(&target_ref)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auto-create new version if content changed
    if content_changed {
        let max_version: Option<(i64,)> =
            sqlx::query_as("SELECT MAX(version) FROM prompt_versions WHERE layer_id = ?")
                .bind(&id)
                .fetch_optional(&pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let next_version = max_version.and_then(|r| Some(r.0)).unwrap_or(0) + 1;
        let version_id = Ulid::new().to_string();

        sqlx::query(
            "INSERT INTO prompt_versions (id, layer_id, version, content, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&version_id)
        .bind(&id)
        .bind(next_version)
        .bind(&content)
        .bind(&now)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(PromptLayer {
        id,
        project_id: existing.project_id,
        layer_type: existing.layer_type,
        target_ref,
        content,
        created_at: existing.created_at,
        updated_at: now,
    }))
}

pub async fn delete_layer(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM prompt_layers WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Version management ---

pub async fn list_versions(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Vec<PromptVersion>>, StatusCode> {
    let versions = sqlx::query_as::<_, PromptVersion>(
        "SELECT * FROM prompt_versions WHERE layer_id = ? ORDER BY version DESC",
    )
    .bind(&id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(versions))
}

pub async fn rollback_version(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<RollbackRequest>,
) -> Result<Json<PromptLayer>, StatusCode> {
    // Find target version content
    let target = sqlx::query_as::<_, PromptVersion>(
        "SELECT * FROM prompt_versions WHERE layer_id = ? AND version = ?",
    )
    .bind(&id)
    .bind(payload.version)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let now = chrono::Utc::now().to_rfc3339();

    // Update layer content
    sqlx::query("UPDATE prompt_layers SET content = ?, updated_at = ? WHERE id = ?")
        .bind(&target.content)
        .bind(&now)
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create new version for the rollback
    let max_version: Option<(i64,)> =
        sqlx::query_as("SELECT MAX(version) FROM prompt_versions WHERE layer_id = ?")
            .bind(&id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let next_version = max_version.and_then(|r| Some(r.0)).unwrap_or(0) + 1;
    let version_id = Ulid::new().to_string();

    sqlx::query(
        "INSERT INTO prompt_versions (id, layer_id, version, content, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(&id)
    .bind(next_version)
    .bind(&target.content)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let layer =
        sqlx::query_as::<_, PromptLayer>("SELECT * FROM prompt_layers WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(layer))
}

pub async fn diff_versions(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Query(q): Query<DiffQuery>,
) -> Result<Json<DiffResult>, StatusCode> {
    let v1 = sqlx::query_as::<_, PromptVersion>(
        "SELECT * FROM prompt_versions WHERE layer_id = ? AND version = ?",
    )
    .bind(&id)
    .bind(q.v1)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let v2 = sqlx::query_as::<_, PromptVersion>(
        "SELECT * FROM prompt_versions WHERE layer_id = ? AND version = ?",
    )
    .bind(&id)
    .bind(q.v2)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let diff = TextDiff::from_lines(&v1.content, &v2.content);
    let changes: Vec<DiffChange> = diff
        .iter_all_changes()
        .map(|change| {
            let tag = match change.tag() {
                ChangeTag::Equal => "equal",
                ChangeTag::Insert => "insert",
                ChangeTag::Delete => "delete",
            };
            DiffChange {
                tag: tag.to_string(),
                content: change.value().to_string(),
            }
        })
        .collect();

    Ok(Json(DiffResult {
        v1: q.v1,
        v2: q.v2,
        changes,
    }))
}

// --- Merged prompt ---

pub async fn get_merged_prompt(
    State(pool): State<SqlitePool>,
    Path(project_id): Path<String>,
    Query(q): Query<MergeQuery>,
) -> Result<Json<MergedPromptResponse>, StatusCode> {
    let layers = sqlx::query_as::<_, PromptLayer>(
        "SELECT * FROM prompt_layers WHERE project_id = ? ORDER BY \
         CASE layer_type WHEN 'global' THEN 0 WHEN 'project' THEN 1 \
         WHEN 'provider' THEN 2 WHEN 'model' THEN 3 END",
    )
    .bind(&project_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let layer_infos: Vec<MergedLayerInfo> = layers
        .iter()
        .map(|l| MergedLayerInfo {
            layer_type: l.layer_type.clone(),
            has_content: !l.content.trim().is_empty(),
        })
        .collect();

    let merged: String = layers
        .iter()
        .map(|l| l.content.trim())
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    // Variable injection
    let final_prompt = if let Some(vars_str) = q.variables {
        let vars: Variables =
            serde_json::from_str(&vars_str).map_err(|_| StatusCode::BAD_REQUEST)?;
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.replace_all(&merged, |caps: &regex::Captures| {
            vars.get(&caps[1])
                .cloned()
                .unwrap_or_else(|| caps[0].to_string())
        })
        .to_string()
    } else {
        merged
    };

    Ok(Json(MergedPromptResponse {
        merged_prompt: final_prompt,
        layers: layer_infos,
    }))
}
