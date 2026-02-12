use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Wrapper for PostgreSQL extension functions
pub struct FhirExtension {
    pool: PgPool,
}

impl FhirExtension {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Call fhir_put extension function
    /// Persists a resource and returns its UUID
    pub async fn fhir_put(
        &self,
        resource_type: &str,
        resource_data: &serde_json::Value,
    ) -> Result<Uuid, sqlx::Error> {
        let id: Uuid = sqlx::query_scalar(
            "SELECT fhir_put($1, $2::jsonb)"
        )
        .bind(resource_type)
        .bind(resource_data)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            "✓ Resource persisted via extension: {} ({})",
            resource_type,
            id
        );
        Ok(id)
    }

    /// Call fhir_get extension function
    /// Retrieves a resource by type and ID
    pub async fn fhir_get(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let result: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT fhir_get($1, $2)"
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_optional(&self.pool)
        .await?;

        if result.is_some() {
            tracing::debug!("✓ Resource retrieved via extension: {} ({})", resource_type, resource_id);
        }
        Ok(result)
    }

    /// Call fhir_search extension function
    /// Searches resources by parameter
    pub async fn fhir_search(
        &self,
        resource_type: &str,
        param: &str,
        op: &str,
        value: &str,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT fhir_search($1, $2, $3, $4)"
        )
        .bind(resource_type)
        .bind(param)
        .bind(op)
        .bind(value)
        .fetch_all(&self.pool)
        .await?;

        tracing::debug!(
            "✓ Search via extension: {} ({}={}) returned {} results",
            resource_type,
            param,
            value,
            ids.len()
        );
        Ok(ids)
    }

    /// Call fhir_get_history extension function
    /// Retrieves all versions of a resource
    pub async fn fhir_get_history(
        &self,
        resource_id: Uuid,
    ) -> Result<Vec<(i32, serde_json::Value, DateTime<Utc>, String)>, sqlx::Error> {
        let results: Vec<(i32, serde_json::Value, DateTime<Utc>, String)> = sqlx::query_as(
            "SELECT version_id, resource, ts, method FROM fhir_get_history($1)"
        )
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await?;

        tracing::debug!("✓ History retrieved via extension: {} versions", results.len());
        Ok(results)
    }

    /// Verify extension is installed and accessible
    pub async fn verify_extension(&self) -> Result<bool, sqlx::Error> {
        let result: Option<String> = sqlx::query_scalar(
            "SELECT extname FROM pg_extension WHERE extname = 'fhir_extension'"
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(_) => {
                tracing::info!("✓ FHIR Extension verified");
                Ok(true)
            }
            None => {
                tracing::warn!("⚠ FHIR Extension not found - using SQL fallback functions");
                Ok(false)
            }
        }
    }

    /// Check if extension functions exist
    pub async fn verify_functions(&self) -> Result<bool, sqlx::Error> {
        let result: Option<i32> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pg_proc WHERE proname IN ('fhir_put', 'fhir_get', 'fhir_search', 'fhir_get_history')"
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(count) if count >= 4 => {
                tracing::info!("✓ All 4 FHIR extension functions verified");
                Ok(true)
            }
            Some(count) => {
                tracing::warn!("⚠ Only {}/4 FHIR functions found", count);
                Ok(false)
            }
            None => {
                tracing::error!("✗ Failed to verify FHIR functions");
                Ok(false)
            }
        }
    }

    /// Call fhir_update extension function
    /// Updates an existing resource and returns its UUID
    pub async fn fhir_update(
        &self,
        resource_type: &str,
        resource_id: Uuid,
        resource_data: &serde_json::Value,
    ) -> Result<Uuid, sqlx::Error> {
        let id: Uuid = sqlx::query_scalar(
            "SELECT fhir_update($1, $2, $3::jsonb)"
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(resource_data)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            "✓ Resource updated via extension: {} ({})",
            resource_type,
            id
        );
        Ok(id)
    }
}
