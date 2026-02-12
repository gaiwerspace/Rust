use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::{PatientRepository, SearchParams},
    models::{OperationOutcome, SearchBundle, SearchBundleEntry},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Name parameter - supports "name=value" or "name:contains=value"
    #[serde(alias = "name:contains", alias = "name:exact")]
    pub name: Option<String>,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    #[serde(rename = "_count")]
    pub count: Option<i32>,
    #[serde(rename = "_offset")]
    pub offset: Option<i32>,
}

/// POST /fhir/Patient
/// Create a new patient resource
pub async fn create_patient(
    State(state): State<AppState>,
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

    let repo = PatientRepository::new(state.db_pool.clone());

    let id = repo
        .upsert(resource.clone())
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
        obj.insert("id".to_string(), serde_json::json!(id.to_string()));
        obj.insert(
            "meta".to_string(),
            serde_json::json!({
                "versionId": "1",
                "lastUpdated": chrono::Utc::now().to_rfc3339()
            }),
        );
    }

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /fhir/Patient/{id}
/// Retrieve a patient by ID
pub async fn get_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<OperationOutcome>)> {
    let repo = PatientRepository::new(state.db_pool.clone());

    repo.get_by_id(id)
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
/// Search for patients
pub async fn search_patients(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchBundle>, (StatusCode, Json<OperationOutcome>)> {
    let repo = PatientRepository::new(state.db_pool.clone());

    let mut search_params = SearchParams::new()
        .with_count(params.count.unwrap_or(20).min(100))
        .with_offset(params.offset.unwrap_or(0));

    if let Some(name) = params.name {
        search_params = search_params.with_name(name);
    }

    if let Some(gender) = params.gender {
        search_params = search_params.with_gender(gender);
    }

    if let Some(birthdate) = params.birthdate {
        search_params = search_params.with_birthdate(birthdate);
    }

    let resources = repo
        .search(search_params)
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
        total: resources.len() as i32,
        entry: resources
            .into_iter()
            .map(|resource| SearchBundleEntry { resource })
            .collect(),
    };

    Ok(Json(bundle))
}

/// PUT /fhir/Patient/{id}
/// Update an existing patient resource
pub async fn update_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
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
        if resource_id != id.to_string() {
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
        // Set ID if not present
        if let Some(obj) = resource.as_object_mut() {
            obj.insert("id".to_string(), serde_json::json!(id.to_string()));
        }
    }

    let repo = PatientRepository::new(state.db_pool.clone());

    // Check if patient exists
    let existing = repo
        .get_by_id(id)
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

    // Update the resource via extension
    let updated_id = repo
        .update(id, resource.clone())
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
    let updated_resource = repo
        .get_by_id(updated_id)
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