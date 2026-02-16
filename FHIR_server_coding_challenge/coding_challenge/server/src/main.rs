mod db;
mod handlers;
mod models;

use axum::{
    routing::{get, patch, post, put},
    Router,
};
use db::Database;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/fhir_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let db = Arc::new(Database::new(pool));

    // Build our application with routes
    let app = Router::new()
        .route("/fhir/Patient", post(handlers::create_patient))
        .route("/fhir/Patient", get(handlers::search_patients))
        .route(
            "/fhir/Patient/:id/_history",
            get(handlers::get_patient_history),
        )
        .route(
            "/fhir/Patient/:id",
            get(handlers::get_patient)
                .put(handlers::update_patient)
                .patch(handlers::patch_patient),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("FHIR Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
