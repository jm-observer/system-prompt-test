pub mod prompts;

use axum::{
    http::StatusCode,
    routing::get,
    Router,
};
use sqlx::SqlitePool;

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/prompts",
            get(prompts::list_prompts).post(prompts::create_prompt),
        )
        .route(
            "/api/prompts/{id}",
            get(prompts::get_prompt)
                .put(prompts::update_prompt)
                .delete(prompts::delete_prompt),
        )
        .with_state(pool)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
