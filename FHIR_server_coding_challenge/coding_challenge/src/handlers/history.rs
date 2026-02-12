use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    db::PatientRepository,
    models::{HistoryBundle, HistoryBundleEntry, HistoryRequest, HistoryResponse, OperationOutcome},
    AppState,
};

/// GET /fhir/Patient/{id}/_history
/// Retrieve the version history of a patient resource
pub async fn get_patient_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<HistoryBundle>, (StatusCode, Json<OperationOutcome>)> {
    let repo = PatientRepository::new(state.db_pool.clone());

    // Check if patient exists
    if repo
        .get_by_id(id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "transient",
                    format!("Database error: {}", e),
                )),
            )
        })?
        .is_none()
    {
        return Err((
            StatusCode::NOT_FOUND,
            Json(OperationOutcome::error_with_location(
                "not-found",
                format!("Patient with ID {} not found", id),
                format!("Patient/{}", id),
            )),
        ));
    }

    // Get history
    let history = repo
        .get_patient_history(id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "exception",
                    format!("Failed to retrieve history: {}", e),
                )),
            )
        })?;

    // Build bundle
    let mut bundle = HistoryBundle::new(id);

    for record in history {
        let mut entry_resource = record.resource.clone();

        // Add meta information with version
        if let Some(obj) = entry_resource.as_object_mut() {
            obj.insert(
                "meta".to_string(),
                serde_json::json!({
                    "versionId": record.version_id.to_string(),
                    "lastUpdated": record.timestamp.to_rfc3339()
                }),
            );
        }

        let entry = HistoryBundleEntry {
            full_url: format!("Patient/{}/Patient/{}", id, record.version_id),
            resource: entry_resource,
            request: Some(HistoryRequest {
                method: record.method.clone(),
                url: format!("Patient/{}", id),
            }),
            response: Some(HistoryResponse {
                status: "200 OK".to_string(),
                last_modified: record.timestamp.to_rfc3339(),
            }),
        };

        bundle = bundle.add_entry(entry);
    }

    Ok(Json(bundle))
}

/// GET /fhir/Patient/{id}/_history/{version_id}
/// Retrieve a specific version of a patient resource
pub async fn get_patient_version(
    State(state): State<AppState>,
    Path((id, version_id)): Path<(Uuid, i32)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<OperationOutcome>)> {
    let repo = PatientRepository::new(state.db_pool.clone());

    let result = repo
        .get_patient_version(id, version_id)
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

    match result {
        Some(record) => {
            let mut resource = record.resource;
            if let Some(obj) = resource.as_object_mut() {
                obj.insert(
                    "meta".to_string(),
                    serde_json::json!({
                        "versionId": record.version_id.to_string(),
                        "lastUpdated": record.timestamp.to_rfc3339()
                    }),
                );
            }
            Ok(Json(resource))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(OperationOutcome::error_with_location(
                "not-found",
                format!("Version {} of Patient {} not found", version_id, id),
                format!("Patient/{}_history/{}", id, version_id),
            )),
        )),
    }
}
