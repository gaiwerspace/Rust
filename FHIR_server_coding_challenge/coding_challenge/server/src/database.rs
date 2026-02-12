use crate::models::Patient;
use anyhow::Result;
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create or update a Patient resource
    pub async fn create_patient(&self, patient: Patient) -> Result<Patient> {
        let patient_json = serde_json::to_value(&patient)?;
        let patient_id = Uuid::new_v4();

        // Insert directly into fhir_resources table
        let result = sqlx::query(
            "INSERT INTO fhir_resources (id, resource_type, resource_data, version_id, last_updated)
             VALUES ($1, 'Patient', $2, 1, NOW())
             RETURNING id, version_id, last_updated"
        )
        .bind(patient_id)
        .bind(&patient_json)
        .fetch_one(&self.pool)
        .await?;

        let created_id: Uuid = result.get("id");
        let version_id: i32 = result.get("version_id");
        let last_updated: chrono::DateTime<chrono::Utc> = result.get("last_updated");

        // Build the complete patient with metadata
        let mut created_patient: Patient = serde_json::from_value(patient_json.clone())?;
        created_patient.id = Some(created_id.to_string());
        created_patient.meta = Some(crate::models::Meta {
            version_id: Some(version_id.to_string()),
            last_updated: Some(last_updated),
        });

        // Also record initial version in fhir.patient_history so that
        // the FHIR history endpoint (_history) can return a proper Bundle.
        //
        // We intentionally ignore errors here to avoid failing the entire
        // request if the history schema is missing or misconfigured.
        let _ = sqlx::query(
            "INSERT INTO fhir.patient_history (id, version_id, resource, txid, ts, status)
             VALUES ($1, $2, $3, txid_current(), NOW(), 'created')"
        )
        .bind(created_id)
        .bind(version_id)
        .bind(patient_json)
        .execute(&self.pool)
        .await;

        Ok(created_patient)
    }

    /// Get a Patient by UUID
    pub async fn get_patient(&self, id: &str) -> Result<Option<Patient>> {
        // Parse the ID as UUID
        let patient_uuid = Uuid::parse_str(id)?;

        // Query the patient with metadata from the database
        let query = sqlx::query(
            "SELECT resource_data, version_id, last_updated FROM fhir_resources WHERE id = $1 AND resource_type = 'Patient'"
        )
        .bind(patient_uuid)
        .fetch_optional(&self.pool)
        .await?;

        match query {
            Some(row) => {
                let resource_data: Value = row.get("resource_data");
                let version_id: i32 = row.get("version_id");
                let last_updated: chrono::DateTime<chrono::Utc> = row.get("last_updated");

                let mut patient: Patient = serde_json::from_value(resource_data)?;
                patient.id = Some(id.to_string());
                patient.meta = Some(crate::models::Meta {
                    version_id: Some(version_id.to_string()),
                    last_updated: Some(last_updated),
                });

                Ok(Some(patient))
            }
            None => Ok(None),
        }
    }

    /// Update a Patient resource (PUT semantics - merge with version management)
    pub async fn update_patient(&self, id: &str, patient: Patient) -> Result<Option<Patient>> {
        // Parse UUID
        let patient_uuid = Uuid::parse_str(id)?;

        // Get existing patient first to verify it exists and merge data
        let existing = match self.get_patient(id).await? {
            Some(p) => p,
            None => return Ok(None),
        };

        // Helper to merge the `name` field without losing existing entries.
        // Strategy:
        // - If no new names are provided, keep existing as-is
        // - If new names are provided:
        //   - If a name has a `use` value (flattened into `extra["use"]`),
        //     it replaces the existing name with the same `use`
        //   - Names with a new `use` (or without `use`) are appended
        fn merge_names(
            existing: Option<Vec<crate::models::HumanName>>,
            incoming: Option<Vec<crate::models::HumanName>>,
        ) -> Option<Vec<crate::models::HumanName>> {
            use std::collections::HashMap;

            // If nothing new is provided, keep what we had
            let Some(incoming_list) = incoming else {
                return existing;
            };

            // If only new names exist, just take them
            let Some(mut existing_list) = existing else {
                return Some(incoming_list);
            };

            // Index existing names by `use` (if present)
            fn extract_use(name: &crate::models::HumanName) -> Option<String> {
                name.extra
                    .get("use")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }

            let mut index_by_use: HashMap<String, usize> = HashMap::new();
            for (idx, n) in existing_list.iter().enumerate() {
                if let Some(u) = extract_use(n) {
                    index_by_use.entry(u).or_insert(idx);
                }
            }

            // Merge: replace by `use` where possible, otherwise append
            for new_name in incoming_list {
                if let Some(u) = extract_use(&new_name) {
                    if let Some(&idx) = index_by_use.get(&u) {
                        existing_list[idx] = new_name;
                        continue;
                    }
                }
                existing_list.push(new_name);
            }

            Some(existing_list)
        }

        // Merge the update with existing data (update takes precedence for non-None fields)
        let merged_patient = Patient {
            id: Some(id.to_string()),
            resource_type: "Patient".to_string(),
            meta: None, // Will be set from database
            name: merge_names(existing.name, patient.name),
            gender: if patient.gender.is_some() { patient.gender } else { existing.gender },
            birth_date: if patient.birth_date.is_some() { patient.birth_date } else { existing.birth_date },
            extra: {
                // Merge extra fields - existing fields preserved, new fields added
                let mut merged_extra = existing.extra;
                for (key, value) in patient.extra {
                    merged_extra.insert(key, value);
                }
                merged_extra
            },
        };

        // Update in database and increment version
        let patient_json = serde_json::to_value(&merged_patient)?;

        let result = sqlx::query(
            "UPDATE fhir_resources
             SET resource_data = $1, version_id = version_id + 1, last_updated = NOW()
             WHERE id = $2 AND resource_type = 'Patient'
             RETURNING version_id, last_updated"
        )
        .bind(&patient_json)
        .bind(patient_uuid)
        .fetch_one(&self.pool)
        .await?;

        let version_id: i32 = result.get("version_id");
        let last_updated: chrono::DateTime<chrono::Utc> = result.get("last_updated");

        // Build the complete updated patient with metadata
        let mut updated_patient = merged_patient;
        updated_patient.meta = Some(crate::models::Meta {
            version_id: Some(version_id.to_string()),
            last_updated: Some(last_updated),
        });

        // Record this version in fhir.patient_history so the _history
        // endpoint can expose a full version list for the patient.
        let _ = sqlx::query(
            "INSERT INTO fhir.patient_history (id, version_id, resource, txid, ts, status)
             VALUES ($1, $2, $3, txid_current(), NOW(), 'updated')"
        )
        .bind(patient_uuid)
        .bind(version_id)
        .bind(&patient_json)
        .execute(&self.pool)
        .await;

        Ok(Some(updated_patient))
    }

    /// Search patients by parameters using FHIR search semantics
    pub async fn search_patients(
        &self,
        name: Option<&str>,
        birth_date: Option<&str>,
        birth_date_ge: Option<&str>,
        birth_date_le: Option<&str>,
        gender: Option<&str>,
        count: u32,
        offset: u32,
    ) -> Result<Vec<Patient>> {
        // Start with a query to get all patients
        let mut query_str = "SELECT id, resource_data, version_id, last_updated FROM fhir_resources WHERE resource_type = 'Patient'".to_string();

        // Add name filter if provided
        if let Some(name_val) = name {
            query_str.push_str(&format!(
                " AND (resource_data #>> '{{name,0,family}}' ILIKE '%{}%' OR resource_data #>> '{{name,0,given,0}}' ILIKE '%{}%')",
                name_val.replace("'", "''"),
                name_val.replace("'", "''")
            ));
        }

        // Add gender filter if provided
        if let Some(gender_val) = gender {
            query_str.push_str(&format!(
                " AND resource_data->>'gender' = '{}'",
                gender_val.replace("'", "''")
            ));
        }

        // Add birth date filter if provided
        if let Some(birth_date_val) = birth_date {
            query_str.push_str(&format!(
                " AND resource_data->>'birthDate' = '{}'",
                birth_date_val.replace("'", "''")
            ));
        }

        // Add birth date greater than or equal filter
        if let Some(birth_date_ge_val) = birth_date_ge {
            query_str.push_str(&format!(
                " AND resource_data->>'birthDate' >= '{}'",
                birth_date_ge_val.replace("'", "''")
            ));
        }

        // Add birth date less than or equal filter
        if let Some(birth_date_le_val) = birth_date_le {
            query_str.push_str(&format!(
                " AND resource_data->>'birthDate' <= '{}'",
                birth_date_le_val.replace("'", "''")
            ));
        }

        // Add pagination
        query_str.push_str(&format!(" ORDER BY id LIMIT {} OFFSET {}", count, offset));

        let rows = sqlx::query(&query_str)
            .fetch_all(&self.pool)
            .await?;

        let mut patients = Vec::new();
        for row in rows {
            let patient_id: Uuid = row.get("id");
            let resource_data: Value = row.get("resource_data");
            let version_id: i32 = row.get("version_id");
            let last_updated: chrono::DateTime<chrono::Utc> = row.get("last_updated");

            let mut patient: Patient = serde_json::from_value(resource_data)?;
            patient.id = Some(patient_id.to_string());
            patient.meta = Some(crate::models::Meta {
                version_id: Some(version_id.to_string()),
                last_updated: Some(last_updated),
            });

            patients.push(patient);
        }

        Ok(patients)
    }

    /// Count total active patients
    pub async fn count_patients(&self) -> Result<i64> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM fhir.patient WHERE status = 'created'")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.get("count"))
    }

    /// Delete a Patient (soft delete)
    pub async fn delete_patient(&self, id: &str) -> Result<bool> {
        let patient_uuid = Uuid::parse_str(id)?;

        // Soft delete by marking status as 'deleted'
        let result = sqlx::query(
            "UPDATE fhir.patient SET status = 'deleted' WHERE id = $1 RETURNING id"
        )
        .bind(patient_uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    /// Get version history of a Patient
    /// Returns (version_id, timestamp, resource, status)
    pub async fn get_patient_history(
        &self,
        id: &str,
    ) -> Result<Vec<(i32, String, Value, Option<String>)>> {
        let patient_uuid = Uuid::parse_str(id)?;

        let rows = sqlx::query(
            "SELECT version_id,
                    to_char(ts, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as ts,
                    resource,
                    status
             FROM fhir.patient_history
             WHERE id = $1
             ORDER BY version_id DESC"
        )
        .bind(patient_uuid)
        .fetch_all(&self.pool)
        .await?;

        let history = rows
            .into_iter()
            .map(|row| {
                let version_id: i32 = row.get("version_id");
                let ts: String = row.get("ts");
                let resource: Value = row.get("resource");
                let status: Option<String> = row.try_get("status").ok();
                (version_id, ts, resource, status)
            })
            .collect();

        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HumanName, Patient};
    use serde_json::Map;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> Database {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/fhir_db".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Database::new(pool)
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

    fn create_comprehensive_test_patient() -> Patient {
        let mut extra = Map::<String, Value>::new();

        // Add identifier
        extra.insert("identifier".to_string(), serde_json::json!([
            {
                "use": "usual",
                "system": "urn:oid:1.2.36.146.595.217.0.1",
                "value": "TEST-123456"
            },
            {
                "use": "official",
                "system": "http://hl7.org/fhir/sid/us-ssn",
                "value": "123-45-6789"
            }
        ]));

        // Add active status
        extra.insert("active".to_string(), serde_json::json!(true));

        // Add telecom
        extra.insert("telecom".to_string(), serde_json::json!([
            {
                "system": "phone",
                "value": "+49-391-555-1234",
                "use": "home"
            },
            {
                "system": "email",
                "value": "anna.schmidt@example.de",
                "use": "home"
            }
        ]));

        // Add address
        extra.insert("address".to_string(), serde_json::json!([
            {
                "use": "home",
                "type": "both",
                "line": ["Hauptstraße 123"],
                "city": "Wanzleben-Börde",
                "state": "Saxony-Anhalt",
                "postalCode": "39164",
                "country": "DE"
            }
        ]));

        // Add maritalStatus
        extra.insert("maritalStatus".to_string(), serde_json::json!({
            "coding": [{
                "system": "http://terminology.hl7.org/CodeSystem/v3-MaritalStatus",
                "code": "M",
                "display": "Married"
            }]
        }));

        // Add contact
        extra.insert("contact".to_string(), serde_json::json!([
            {
                "relationship": [{
                    "coding": [{
                        "code": "C",
                        "display": "Emergency Contact"
                    }]
                }],
                "name": {
                    "family": "Schmidt",
                    "given": ["Thomas"]
                }
            }
        ]));

        // Add communication
        extra.insert("communication".to_string(), serde_json::json!([
            {
                "language": {
                    "coding": [{
                        "code": "de",
                        "display": "German"
                    }]
                },
                "preferred": true
            }
        ]));

        Patient {
            id: None,
            resource_type: "Patient".to_string(),
            meta: None,
            name: Some(vec![
                HumanName {
                    family: Some("Schmidt".to_string()),
                    given: Some(vec!["Anna".to_string(), "Maria".to_string()]),
                    text: Some("Anna Maria Schmidt".to_string()),
                    extra: Map::<String, Value>::new(),
                },
            ]),
            gender: Some("female".to_string()),
            birth_date: Some("1985-03-15".to_string()),
            extra,
        }
    }

    #[tokio::test]
    async fn test_create_patient() {
        let db = setup_test_db().await;
        let patient = create_test_patient("Smith", "Carl", "male", "1990-01-01");
        let result = db.create_patient(patient).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.resource_type, "Patient");
    }

    #[tokio::test]
    async fn test_get_patient() {
        let db = setup_test_db().await;
        let patient = create_test_patient("Gauß", "Jane", "female", "1985-05-15");
        let created = db.create_patient(patient).await.unwrap();
        let patient_id = created.id.clone().unwrap();

        let result = db.get_patient(&patient_id).await;
        assert!(result.is_ok());

        let retrieved = result.unwrap();
        assert!(retrieved.is_some());
        let patient = retrieved.unwrap();
        assert_eq!(patient.id, Some(patient_id));
        assert_eq!(patient.gender, Some("female".to_string()));
        assert_eq!(patient.birth_date, Some("1985-05-15".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_patient() {
        let db = setup_test_db().await;
        let fake_id = Uuid::new_v4().to_string();
        let result = db.get_patient(&fake_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_search_patients_by_gender() {
        let db = setup_test_db().await;
        db.create_patient(create_test_patient("Brown", "Alice", "female", "1992-03-20")).await.unwrap();

        let result = db.search_patients(None, None, None, None, Some("female"), 10, 0).await;
        assert!(result.is_ok());
        let patients = result.unwrap();
        assert!(!patients.is_empty());
    }

    #[tokio::test]
    async fn test_search_patients_by_birth_date() {
        let db = setup_test_db().await;
        let birth_date = "1995-12-25";
        db.create_patient(create_test_patient("White", "Charlie", "male", birth_date)).await.unwrap();

        let result = db.search_patients(None, Some(birth_date), None, None, None, 10, 0).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_patients_by_name() {
        let db = setup_test_db().await;
        db.create_patient(create_test_patient("Johnson", "Emily", "female", "1991-08-30")).await.unwrap();

        let result = db.search_patients(Some("Johnson"), None, None, None, None, 10, 0).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_patients_pagination() {
        let db = setup_test_db().await;
        for i in 0..5 {
            db.create_patient(create_test_patient(
                &format!("Family{}", i),
                &format!("Given{}", i),
                "other",
                "2000-01-01"
            )).await.unwrap();
        }

        let page1 = db.search_patients(None, None, None, None, None, 2, 0).await.unwrap();
        let page2 = db.search_patients(None, None, None, None, None, 2, 2).await.unwrap();

        assert!(page1.len() <= 2);
        assert!(page2.len() <= 2);
    }

    #[tokio::test]
    async fn test_search_all_patients() {
        let db = setup_test_db().await;
        let result = db.search_patients(None, None, None, None, None, 100, 0).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_patient_with_multiple_given_names() {
        let db = setup_test_db().await;
        let mut patient = create_test_patient("Wilson", "Mary", "female", "1987-04-12");
        if let Some(ref mut names) = patient.name {
            if let Some(ref mut given) = names[0].given {
                given.push("Jane".to_string());
            }
        }

        let created = db.create_patient(patient).await.unwrap();
        let patient_id = created.id.clone().unwrap();

        let retrieved = db.get_patient(&patient_id).await.unwrap().unwrap();
        assert_eq!(retrieved.name.as_ref().unwrap()[0].given.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_update_patient_merges_data() {
        let db = setup_test_db().await;
        let original = create_test_patient("Gauß", "Carl", "male", "1990-01-01");
        let created = db.create_patient(original).await.unwrap();
        let patient_id = created.id.clone().unwrap();

        let update = Patient {
            id: None,
            resource_type: "Patient".to_string(),
            meta: None,
            name: None,
            gender: Some("female".to_string()),
            birth_date: None,
            extra: Map::<String, Value>::new(),
        };

        let result = db.update_patient(&patient_id, update).await;
        assert!(result.is_ok());

        let updated = result.unwrap().unwrap();
        assert_eq!(updated.gender, Some("female".to_string()));
        assert_eq!(updated.birth_date, Some("1990-01-01".to_string()));
    }

    #[tokio::test]
    async fn test_comprehensive_patient_creation() {
        let db = setup_test_db().await;
        let patient = create_comprehensive_test_patient();
        let result = db.create_patient(patient).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.gender.as_ref().unwrap(), "female");
        assert_eq!(created.birth_date.as_ref().unwrap(), "1985-03-15");

        assert!(created.extra.contains_key("identifier"));
        assert!(created.extra.contains_key("telecom"));
        assert!(created.extra.contains_key("address"));
        assert!(created.extra.contains_key("communication"));
    }

    #[tokio::test]
    async fn test_comprehensive_patient_retrieval() {
        let db = setup_test_db().await;
        let original = create_comprehensive_test_patient();
        let created = db.create_patient(original).await.unwrap();
        let patient_id = created.id.clone().unwrap();

        let retrieved = db.get_patient(&patient_id).await.unwrap().unwrap();

        assert_eq!(retrieved.gender.as_ref().unwrap(), "female");
        assert_eq!(retrieved.birth_date.as_ref().unwrap(), "1985-03-15");
        assert!(retrieved.extra.contains_key("identifier"));
        assert!(retrieved.extra.contains_key("address"));
    }

    #[tokio::test]
    async fn test_comprehensive_patient_update() {
        let db = setup_test_db().await;
        let original = create_comprehensive_test_patient();
        let created = db.create_patient(original).await.unwrap();
        let patient_id = created.id.clone().unwrap();

        let update = Patient {
            id: None,
            resource_type: "Patient".to_string(),
            meta: None,
            name: None,
            gender: Some("other".to_string()),
            birth_date: None,
            extra: Map::<String, Value>::new(),
        };

        let updated = db.update_patient(&patient_id, update).await.unwrap().unwrap();
        assert_eq!(updated.gender.as_ref().unwrap(), "other");
        assert!(updated.extra.contains_key("identifier"));
    }
}
