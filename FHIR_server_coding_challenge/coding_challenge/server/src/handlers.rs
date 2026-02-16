use crate::db::Database;
use crate::models::{Bundle, BundleEntry, OperationOutcome, OperationOutcomeIssue, Patient};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use json_patch::Patch;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    name: Option<String>,
    #[serde(rename = "name:contains")]
    name_contains: Option<String>,
    #[serde(rename = "birthdate")]
    birth_date: Option<String>,
    #[serde(rename = "birthdate:ge")]
    birth_date_ge: Option<String>,
    #[serde(rename = "birthdate:le")]
    birth_date_le: Option<String>,
    gender: Option<String>,
    #[serde(rename = "_count")]
    count: Option<u32>,
    #[serde(rename = "_offset")]
    offset: Option<u32>,
}

pub async fn create_patient(
    State(db): State<Arc<Database>>,
    Json(mut patient): Json<Patient>,
) -> Result<(StatusCode, HeaderMap, Json<Patient>), (StatusCode, Json<OperationOutcome>)> {
    // Ensure resource type is correct
    patient.resource_type = "Patient".to_string();

    match db.create_patient(patient).await {
        Ok(created_patient) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/fhir+json".parse().unwrap());
            if let Some(id) = &created_patient.id {
                headers.insert("Location", format!("/fhir/Patient/{}", id).parse().unwrap());
            }

            Ok((StatusCode::CREATED, headers, Json(created_patient)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to create patient: {}", e),
            )),
        )),
    }
}

pub async fn get_patient(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, HeaderMap, Json<Patient>), (StatusCode, Json<OperationOutcome>)> {
    match db.get_patient(&id).await {
        Ok(Some(patient)) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/fhir+json".parse().unwrap());
            Ok((StatusCode::OK, headers, Json(patient)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(OperationOutcome::error_with_location(
                "not-found",
                format!("Patient with id {} not found", id),
                format!("Patient/{}", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to retrieve patient: {}", e),
            )),
        )),
    }
}

pub async fn update_patient(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
    Json(patient): Json<Patient>,
) -> Result<(StatusCode, HeaderMap, Json<Patient>), (StatusCode, Json<OperationOutcome>)> {
    match db.update_patient(&id, patient).await {
        Ok(Some(updated_patient)) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/fhir+json".parse().unwrap());
            Ok((StatusCode::OK, headers, Json(updated_patient)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(OperationOutcome::error_with_location(
                "not-found",
                format!("Patient with id {} not found", id),
                format!("Patient/{}", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to update patient: {}", e),
            )),
        )),
    }
}

pub async fn patch_patient(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
    Json(patch): Json<Patch>,
) -> Result<(StatusCode, HeaderMap, Json<Patient>), (StatusCode, Json<OperationOutcome>)> {
    // 1. Get existing patient
    let existing_patient = match db.get_patient(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(OperationOutcome::error_with_location(
                    "not-found",
                    format!("Patient with id {} not found", id),
                    format!("Patient/{}", id),
                )),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OperationOutcome::error(
                    "processing",
                    format!("Failed to retrieve patient: {}", e),
                )),
            ))
        }
    };

    // 2. Convert to Value for patching
    let mut patient_value = serde_json::to_value(&existing_patient).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to serialize patient: {}", e),
            )),
        )
    })?;

    // 3. Apply patch
    json_patch::patch(&mut patient_value, &patch).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to apply patch: {}", e),
            )),
        )
    })?;

    // 4. Convert back to Patient (this validates structure)
    let mut patched_patient: Patient = serde_json::from_value(patient_value).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(OperationOutcome::error_with_location(
                "invalid",
                format!("Patch results in invalid patient resource: {}", e),
                "Patient",
            )),
        )
    })?;

    // 5. Ensure immutable fields are preserved/restored if necessary
    // ID should match path ID
    if let Some(pid) = &patched_patient.id {
        if pid != &id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OperationOutcome::error(
                    "invariant",
                    "Resource ID cannot be changed",
                )),
            ));
        }
    } else {
        patched_patient.id = Some(id.clone());
    }

    // ResourceType must be Patient
    if patched_patient.resource_type != "Patient" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OperationOutcome::error(
                "invalid",
                "Resource type must be 'Patient'",
            )),
        ));
    }

    // 6. Update in database
    match db.update_patient(&id, patched_patient).await {
        Ok(Some(updated_patient)) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/fhir+json".parse().unwrap());
            Ok((StatusCode::OK, headers, Json(updated_patient)))
        }
        Ok(None) => {
            // Should not happen as we checked existence, but possible if deleted concurrently
            Err((
                StatusCode::NOT_FOUND,
                Json(OperationOutcome::error_with_location(
                    "not-found",
                    format!("Patient with id {} not found", id),
                    format!("Patient/{}", id),
                )),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to update patient: {}", e),
            )),
        )),
    }
}

pub async fn search_patients(
    State(db): State<Arc<Database>>,
    Query(params): Query<SearchParams>,
) -> Result<(StatusCode, HeaderMap, Json<Bundle>), (StatusCode, Json<OperationOutcome>)> {
    let count = params.count.unwrap_or(20).min(100); // Default 20, max 100
    let offset = params.offset.unwrap_or(0);

    // Prioritize :contains modifier over exact match
    let name_param = params.name_contains.as_deref().or(params.name.as_deref());

    match db
        .search_patients(
            name_param,
            params.birth_date.as_deref(),
            params.birth_date_ge.as_deref(),
            params.birth_date_le.as_deref(),
            params.gender.as_deref(),
            count,
            offset,
        )
        .await
    {
        Ok(patients) => {
            let entries: Vec<BundleEntry> = patients
                .into_iter()
                .map(|patient| BundleEntry { resource: patient })
                .collect();

            let bundle = Bundle {
                resource_type: "Bundle".to_string(),
                bundle_type: "searchset".to_string(),
                total: entries.len() as u32,
                link: None,
                entry: entries,
            };

            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/fhir+json".parse().unwrap());

            Ok((StatusCode::OK, headers, Json(bundle)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to search patients: {}", e),
            )),
        )),
    }
}

pub async fn get_patient_history(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, HeaderMap, Json<Value>), (StatusCode, Json<OperationOutcome>)> {
    match db.get_patient_history(&id).await {
        Ok(history) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", "application/fhir+json".parse().unwrap());
            // Base URL for absolute fullUrl/link values (FHIR R4 requirement)
            let base_url = std::env::var("FHIR_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string());

            // Create FHIR Bundle with type "history"
            // Ensure each entry's resource is a valid FHIR R4 Patient with
            // id and meta.versionId/meta.lastUpdated populated.
            // Also ensure each entry has a `request` element per history spec.

            // Use the newest record timestamp (first row, desc by version_id)
            let bundle_last_updated = history.first().map(|(_, ts, _, _)| ts.clone());

            let entries: Vec<Value> = history
                .into_iter()
                .map(|(version_id, ts, mut resource, status)| {
                    // Make sure resource is an object we can enrich
                    if let Value::Object(ref mut map) = resource {
                        // Ensure resourceType is set to Patient
                        map.entry("resourceType")
                            .or_insert_with(|| Value::String("Patient".to_string()));

                        // Ensure id matches the instance id from the URL
                        map.insert("id".to_string(), Value::String(id.clone()));

                        // Ensure meta.versionId and meta.lastUpdated are present
                        let meta_entry = map
                            .entry("meta")
                            .or_insert_with(|| Value::Object(serde_json::Map::new()));

                        if let Value::Object(ref mut meta_map) = meta_entry {
                            meta_map.insert(
                                "versionId".to_string(),
                                Value::String(version_id.to_string()),
                            );
                            meta_map.insert("lastUpdated".to_string(), Value::String(ts.clone()));
                        }
                    }

                    // Determine HTTP method for history.request based on status
                    let method = match status.as_deref() {
                        Some("created") => "POST",
                        // For updated/snapshot/other we treat as PUT
                        _ => "PUT",
                    };

                    // Build FHIR-compliant history entry
                    json!({
                        // Absolute logical URL without /_history per bdl-8
                        "fullUrl": format!("{}/fhir/Patient/{}", base_url, id),
                        "resource": resource,
                        "request": {
                            "method": method,
                            "url": format!("Patient/{}", id),
                        },
                        "response": {
                            "status": "200 OK",
                            "lastModified": ts
                        }
                    })
                })
                .collect();

            let mut bundle = json!({
                "resourceType": "Bundle",
                "type": "history",
                "total": entries.len(),
                "link": [{
                    "relation": "self",
                    "url": format!("{}/fhir/Patient/{}/_history", base_url, id),
                }],
                "entry": entries
            });

            // Add Bundle.meta.lastUpdated if we have at least one history record
            if let (Some(ts), Value::Object(ref mut map)) = (bundle_last_updated, &mut bundle) {
                map.insert("meta".to_string(), json!({ "lastUpdated": ts }));
            }

            Ok((StatusCode::OK, headers, Json(bundle)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OperationOutcome::error(
                "processing",
                format!("Failed to retrieve patient history: {}", e),
            )),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HumanName;
    use serde_json::Map;
    use serde_json::Value;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> Arc<Database> {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:password@localhost:5432/fhir_db".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Arc::new(Database::new(pool))
    }

    fn create_test_patient(family: &str, given: &str, gender: &str, birth_date: &str) -> Patient {
        Patient {
            id: None,
            resource_type: "Patient".to_string(),
            meta: None,
            name: Some(vec![HumanName {
                family: Some(family.to_string()),
                given: Some(vec![given.to_string()]),
                text: Some(format!("{} {}", given, family)),
                extra: Map::<String, Value>::new(),
            }]),
            gender: Some(gender.to_string()),
            birth_date: Some(birth_date.to_string()),
            extra: Map::<String, Value>::new(),
        }
    }

    #[tokio::test]
    async fn test_create_patient_handler() {
        let db = setup_test_db().await;
        let patient = create_test_patient("TestFamily", "TestGiven", "male", "1990-01-01");

        let result = create_patient(State(db), Json(patient)).await;

        assert!(result.is_ok());
        let (status, headers, json) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert!(headers.contains_key("Content-Type"));
        assert!(headers.contains_key("Location"));
        assert!(json.id.is_some());
    }

    #[tokio::test]
    async fn test_get_patient_handler() {
        let db = setup_test_db().await;

        // Create a patient first
        let patient = create_test_patient("GetTest", "Patient", "female", "1985-05-15");
        let (_, _, created) = create_patient(State(db.clone()), Json(patient))
            .await
            .unwrap();
        let patient_id = created.id.clone().unwrap();

        // Get the patient
        let result = get_patient(State(db), Path(patient_id.clone())).await;

        assert!(result.is_ok());
        let (status, _, json) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json.id, Some(patient_id));
    }

    #[tokio::test]
    async fn test_get_nonexistent_patient_handler() {
        let db = setup_test_db().await;
        let fake_id = uuid::Uuid::new_v4().to_string();

        let result = get_patient(State(db), Path(fake_id)).await;

        assert!(result.is_err());
        let (status, outcome) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(outcome.resource_type, "OperationOutcome");
    }

    #[tokio::test]
    async fn test_search_patients_handler() {
        let db = setup_test_db().await;

        // Create test patients
        let patient1 = create_test_patient("SearchTest1", "Alice", "female", "1990-01-01");
        let patient2 = create_test_patient("SearchTest2", "Bob", "male", "1985-05-15");
        let _ = create_patient(State(db.clone()), Json(patient1))
            .await
            .unwrap();
        let _ = create_patient(State(db.clone()), Json(patient2))
            .await
            .unwrap();

        // Search all patients
        let params = SearchParams {
            name: None,
            name_contains: None,
            birth_date: None,
            birth_date_ge: None,
            birth_date_le: None,
            gender: None,
            count: Some(10),
            offset: Some(0),
        };

        let result = search_patients(State(db), Query(params)).await;

        assert!(result.is_ok());
        let (status, _, bundle) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(bundle.resource_type, "Bundle");
        assert_eq!(bundle.bundle_type, "searchset");
        assert!(!bundle.entry.is_empty());
    }

    #[tokio::test]
    async fn test_search_patients_by_gender_handler() {
        let db = setup_test_db().await;

        let patient = create_test_patient("GenderTest", "Charlie", "other", "1995-12-25");
        let _ = create_patient(State(db.clone()), Json(patient))
            .await
            .unwrap();

        let params = SearchParams {
            name: None,
            name_contains: None,
            birth_date: None,
            birth_date_ge: None,
            birth_date_le: None,
            gender: Some("other".to_string()),
            count: Some(10),
            offset: Some(0),
        };

        let result = search_patients(State(db), Query(params)).await;

        assert!(result.is_ok());
        let (_, _, bundle) = result.unwrap();
        assert!(!bundle.entry.is_empty());
        assert!(bundle
            .entry
            .iter()
            .all(|e| e.resource.gender == Some("other".to_string())));
    }

    #[tokio::test]
    async fn test_search_patients_pagination_handler() {
        let db = setup_test_db().await;

        // Create multiple patients
        for i in 0..5 {
            let patient = create_test_patient(
                &format!("PaginationTest{}", i),
                &format!("Patient{}", i),
                "unknown",
                "2000-01-01",
            );
            let _ = create_patient(State(db.clone()), Json(patient))
                .await
                .unwrap();
        }

        // Get first page
        let params1 = SearchParams {
            name: None,
            name_contains: None,
            birth_date: None,
            birth_date_ge: None,
            birth_date_le: None,
            gender: None,
            count: Some(2),
            offset: Some(0),
        };

        let result1 = search_patients(State(db.clone()), Query(params1)).await;
        assert!(result1.is_ok());
        let (_, _, bundle1) = result1.unwrap();
        assert!(bundle1.entry.len() <= 2);

        // Get second page
        let params2 = SearchParams {
            name: None,
            name_contains: None,
            birth_date: None,
            birth_date_ge: None,
            birth_date_le: None,
            gender: None,
            count: Some(2),
            offset: Some(2),
        };

        let result2 = search_patients(State(db), Query(params2)).await;
        assert!(result2.is_ok());
        let (_, _, bundle2) = result2.unwrap();
        assert!(bundle2.entry.len() <= 2);
    }

    #[tokio::test]
    async fn test_search_params_defaults() {
        let db = setup_test_db().await;

        let params = SearchParams {
            name: None,
            name_contains: None,
            birth_date: None,
            birth_date_ge: None,
            birth_date_le: None,
            gender: None,
            count: None,  // Should default to 20
            offset: None, // Should default to 0
        };

        let result = search_patients(State(db), Query(params)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_patient_sets_resource_type() {
        let db = setup_test_db().await;
        let mut patient = create_test_patient("TypeTest", "Patient", "male", "1990-01-01");
        patient.resource_type = "WrongType".to_string();

        let result = create_patient(State(db), Json(patient)).await;

        assert!(result.is_ok());
        let (_, _, created) = result.unwrap();
        assert_eq!(created.resource_type, "Patient");
    }
}
