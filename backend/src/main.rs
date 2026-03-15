mod db;
mod models;
mod routes;

use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data.db".to_string());

    let pool = db::create_pool(&database_url).await;
    db::run_migrations(&pool).await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::create_router(pool).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    println!("Backend running on http://localhost:3001");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
