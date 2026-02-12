use axum::{
    http::StatusCode,
    routing::{get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod db;
mod handlers;
mod models;

pub use db::{PatientRepository, FhirExtension, SearchParams};
pub use handlers::*;
pub use models::*;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<sqlx::PgPool>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Database setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/fhir_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Verify database connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .expect("Database connection failed");

    tracing::info!("✓ Database connected successfully");

    // Verify FHIR extension
    let extension = FhirExtension::new(pool.clone());
    match extension.verify_functions().await {
        Ok(true) => {
            tracing::info!("✓ FHIR extension functions verified");
        }
        Ok(false) => {
            tracing::warn!("⚠ Some FHIR extension functions missing - check migrations");
        }
        Err(e) => {
            tracing::error!("✗ Failed to verify extension: {}", e);
        }
    }

    let state = AppState {
        db_pool: Arc::new(pool),
    };

    // Build router
    let app = Router::new()
        // Patient endpoints
        .route("/fhir/Patient", post(handlers::create_patient))
        .route("/fhir/Patient", get(handlers::search_patients))
        .route("/fhir/Patient/:id", get(handlers::get_patient))
        .route("/fhir/Patient/:id", put(handlers::update_patient))
        .route("/fhir/Patient/:id/_history", get(handlers::get_patient_history))
        .route(
            "/fhir/Patient/:id/_history/:version_id",
            get(handlers::get_patient_version),
        )
        // Health check
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("🚀 Server running on http://0.0.0.0:3000");
    tracing::info!("📖 FHIR API available at http://0.0.0.0:3000/fhir");
    tracing::info!("💾 All operations use FHIR extension functions (fhir_put, fhir_get, fhir_search)");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// Health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}