pub mod projects;
pub mod prompt_layers;
pub mod providers;
pub mod models_routes;
pub mod test_cases;
pub mod runs;
pub mod assertions;
pub mod reports;

use axum::{http::StatusCode, routing::{get, post, put}, Router};
use sqlx::SqlitePool;

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/health", get(health))
        // Projects
        .route(
            "/api/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/api/projects/{id}",
            get(projects::get_project)
                .put(projects::update_project)
                .delete(projects::delete_project),
        )
        // Layers (nested under project)
        .route(
            "/api/projects/{project_id}/layers",
            get(prompt_layers::list_layers).post(prompt_layers::create_layer),
        )
        // Layers (standalone)
        .route(
            "/api/prompt-layers/{id}",
            get(prompt_layers::get_layer)
                .put(prompt_layers::update_layer)
                .delete(prompt_layers::delete_layer),
        )
        // Versions
        .route(
            "/api/prompt-layers/{id}/versions",
            get(prompt_layers::list_versions),
        )
        .route(
            "/api/prompt-layers/{id}/rollback",
            post(prompt_layers::rollback_version),
        )
        .route(
            "/api/prompt-layers/{id}/versions/diff",
            get(prompt_layers::diff_versions),
        )
        // Merged prompt
        .route(
            "/api/projects/{id}/merged-prompt",
            get(prompt_layers::get_merged_prompt),
        )
        // Providers
        .route(
            "/api/providers",
            get(providers::list_providers).post(providers::create_provider),
        )
        .route(
            "/api/providers/{id}",
            get(providers::get_provider)
                .put(providers::update_provider)
                .delete(providers::delete_provider),
        )
        // Models (nested under provider)
        .route(
            "/api/providers/{provider_id}/models",
            get(models_routes::list_models).post(models_routes::create_model),
        )
        // Models (standalone)
        .route(
            "/api/models/{id}",
            put(models_routes::update_model)
                .delete(models_routes::delete_model),
        )
        .route("/api/models", get(models_routes::list_all_models))
        // Test Cases
        .route(
            "/api/projects/{project_id}/test-cases",
            get(test_cases::list_test_cases).post(test_cases::create_test_case),
        )
        .route(
            "/api/test-cases/{id}",
            get(test_cases::get_test_case)
                .put(test_cases::update_test_case)
                .delete(test_cases::delete_test_case),
        )
        // Runs
        .route(
            "/api/test-cases/{id}/run",
            post(runs::create_run),
        )
        .route(
            "/api/test-cases/{id}/runs",
            get(runs::list_runs),
        )
        .route("/api/runs/{id}", get(runs::get_run))
        .route("/api/runs/{id}/stream", get(runs::stream_run))
        // Assertions
        .route(
            "/api/test-cases/{test_case_id}/assertions",
            get(assertions::list_assertions).post(assertions::create_assertion),
        )
        .route(
            "/api/assertions/{id}",
            axum::routing::delete(assertions::delete_assertion),
        )
        .route(
            "/api/runs/{run_id}/assertion-results",
            get(assertions::list_assertion_results),
        )
        // Reports
        .route("/api/runs/{run_id}/report", get(reports::get_run_report))
        .route("/api/reports/summary", get(reports::list_reports_summary))
        .route("/api/audit-logs", get(reports::list_audit_logs))
        .route(
            "/api/models/{model_id}/pricing",
            get(reports::get_model_pricing).put(reports::update_model_pricing),
        )
        .with_state(pool)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
