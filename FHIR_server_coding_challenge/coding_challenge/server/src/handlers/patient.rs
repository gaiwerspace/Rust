use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::Database;
use crate::models::{OperationOutcome, SearchBundle, SearchBundleEntry};

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientResource {
    pub id: String,
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Name substring search (family, given, or text)
    pub name: Option<String>,
    /// Gender exact match (male, female, other, unknown)
    pub gender: Option<String>,
    /// Birth date exact match (YYYY-MM-DD)
    pub birthdate: Option<String>,
    /// Birth date greater than or equal to (YYYY-MM-DD)
    #[serde(rename = "birthdate-ge")]
    pub birthdate_ge: Option<String>,
    /// Birth date less than or equal to (YYYY-MM-DD)
    #[serde(rename = "birthdate-le")]
    pub birthdate_le: Option<String>,
    /// Number of results per page (default: 20, max: 100)
    #[serde(rename = "_count")]
    pub count: Option<i32>,
    /// Pagination offset (default: 0)
    #[serde(rename = "_offset")]
    pub offset: Option<i32>,
}

/// POST /fhir/Patient
/// Create a new patient resource
pub async fn create_patient(
    State(db): State<Arc<Database>>,
    Json(resource): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<OperationOutcome>)> {
    // Validate resource type
    if resource.get("resourceType").and_then(|v| v.as_str()) != Some("Patient") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OperationOutcome::validation_error(
                "resourceType",
                "Resource type must be 'Patient'",
            )),
        ));
    }

    let id = db
        .create_patient(&resource)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Failed to create patient: {}", e),
                )),
            )
        })?;

    let mut response = resource;
    if let Some(obj) = response.as_object_mut() {
        obj.insert("id".to_string(), serde_json::json!(id));
        obj.insert(
            "meta".to_string(),
            serde_json::json!({
                "versionId": "1",
                "lastUpdated": chrono::Utc::now().to_rfc3339()
            }),
        );
    }

    tracing::info!("✓ Patient created: {}", id);
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /fhir/Patient/:id
/// Retrieve a patient by ID
pub async fn get_patient(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<OperationOutcome>)> {
    db.get_patient(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Database error: {}", e),
                )),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(OperationOutcome::error_with_location(
                    "not-found",
                    format!("Patient with ID {} not found", id),
                    format!("Patient/{}", id),
                )),
            )
        })
        .map(Json)
}

/// GET /fhir/Patient
/// Search for patients with advanced query support
/// Query parameters:
///   - name: substring search in family, given, or text fields
///   - gender: exact match (male, female, other, unknown)
///   - birthdate: exact match (YYYY-MM-DD)
///   - birthdate-ge: birth date >= (YYYY-MM-DD)
///   - birthdate-le: birth date <= (YYYY-MM-DD)
///   - _count: results per page (default: 20, max: 100)
///   - _offset: pagination offset (default: 0)
pub async fn search_patients(
    State(db): State<Arc<Database>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchBundle>, (StatusCode, Json<OperationOutcome>)> {
    // Validate pagination parameters
    let count = params.count.unwrap_or(20).min(100).max(1) as i64;
    let offset = params.offset.unwrap_or(0).max(0) as i64;

    // Validate date ranges if provided
    if let (Some(ref ge), Some(ref le)) = (&params.birthdate_ge, &params.birthdate_le) {
        if ge > le {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OperationOutcome::validation_error(
                    "birthdate-ge",
                    "birthdate-ge must be less than or equal to birthdate-le",
                )),
            ));
        }
    }

    // Execute search
    let (resources, total) = db
        .search_patients(
            params.name.as_deref(),
            params.gender.as_deref(),
            params.birthdate.as_deref(),
            params.birthdate_ge.as_deref(),
            params.birthdate_le.as_deref(),
            count,
            offset,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Search failed: {}", e),
                )),
            )
        })?;

    let bundle = SearchBundle {
        resource_type: "Bundle".to_string(),
        bundle_type: "searchset".to_string(),
        total,
        entry: resources
            .into_iter()
            .map(|resource| SearchBundleEntry { resource })
            .collect(),
    };

    tracing::debug!(
        "✓ Search completed: {} results (count: {}, offset: {})",
        total,
        count,
        offset
    );
    Ok(Json(bundle))
}

/// PUT /fhir/Patient/:id
/// Update an existing patient resource
pub async fn update_patient(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
    Json(mut resource): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<OperationOutcome>)> {
    // Validate resource type
    if resource.get("resourceType").and_then(|v| v.as_str()) != Some("Patient") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OperationOutcome::validation_error(
                "resourceType",
                "Resource type must be 'Patient'",
            )),
        ));
    }

    // Verify ID matches or set it
    if let Some(resource_id) = resource.get("id").and_then(|v| v.as_str()) {
        if resource_id != id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OperationOutcome::error_with_location(
                    "invariant",
                    "Resource ID in URL does not match resource ID in body",
                    "id",
                )),
            ));
        }
    } else {
        if let Some(obj) = resource.as_object_mut() {
            obj.insert("id".to_string(), serde_json::json!(id.clone()));
        }
    }

    // Check if patient exists
    let existing = db
        .get_patient(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Database error: {}", e),
                )),
            )
        })?;

    if existing.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(OperationOutcome::error_with_location(
                "not-found",
                format!("Patient with ID {} not found", id),
                format!("Patient/{}", id),
            )),
        ));
    }

    // Update the resource
    db.update_patient(&id, &resource)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Failed to update patient: {}", e),
                )),
            )
        })?;

    // Fetch updated resource
    let updated_resource = db
        .get_patient(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Failed to fetch updated patient: {}", e),
                )),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    "Updated patient not found",
                )),
            )
        })?;

    tracing::info!("✓ Patient updated: {}", id);
    Ok((StatusCode::OK, Json(updated_resource)))
}
