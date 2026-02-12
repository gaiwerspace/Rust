use sqlx::postgres::PgPool;
use std::borrow::Cow;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::extension::FhirExtension;
use crate::models::PatientHistoryRecord;

/// Patient repository using FHIR extension for all operations
pub struct PatientRepository {
    extension: FhirExtension,
}

impl PatientRepository {
    /// Create new repository instance
    pub fn new(pool: Arc<PgPool>) -> Self {
        let extension = FhirExtension::new((*pool).clone());
        Self { extension }
    }

    /// Get patient by ID using fhir_get extension function
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<serde_json::Value>, sqlx::Error> {
        self.extension.fhir_get("Patient", id).await
    }

    /// Insert or update a patient using fhir_put extension function
    /// This is the ONLY way to persist resources - via the extension
    pub async fn upsert(
        &self,
        resource: serde_json::Value,
    ) -> Result<Uuid, sqlx::Error> {
        self.extension.fhir_put("Patient", &resource).await
    }

    /// Update a patient using fhir_update extension function
    pub async fn update(
        &self,
        id: Uuid,
        resource: serde_json::Value,
    ) -> Result<Uuid, sqlx::Error> {
        self.extension.fhir_update("Patient", id, &resource).await
    }

    /// Search patients by single parameter using fhir_search extension function
    pub async fn search_by_param(
        &self,
        param: &str,
        op: &str,
        value: &str,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        self.extension.fhir_search("Patient", param, op, value).await
    }

    /// Search patients with multiple criteria and pagination
    pub async fn search(
        &self,
        params: SearchParams<'_>,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let mut result_ids = Vec::new();

        // Execute searches for each parameter and collect IDs
        if let Some(name) = &params.name {
            let ids = self
                .search_by_param("name", "contains", name.as_ref())
                .await?;
            result_ids.extend(ids);
        }

        if let Some(gender) = &params.gender {
            let ids = self
                .search_by_param("gender", "exact", gender.as_ref())
                .await?;
            if result_ids.is_empty() {
                result_ids = ids;
            } else {
                // Intersect with existing results
                result_ids.retain(|id| ids.contains(id));
            }
        }

        if let Some(birthdate) = &params.birthdate {
            let ids = self
                .search_by_param("birthDate", "eq", birthdate.as_ref())
                .await?;
            if result_ids.is_empty() {
                result_ids = ids;
            } else {
                // Intersect with existing results
                result_ids.retain(|id| ids.contains(id));
            }
        }

        // If no parameters specified, return empty
        if result_ids.is_empty() && params.name.is_none() && params.gender.is_none() && params.birthdate.is_none() {
            return Ok(Vec::new());
        }

        // Fetch full resources using fhir_get for each ID
        let mut resources = Vec::new();
        for id in result_ids
            .into_iter()
            .skip(params.offset.unwrap_or(0) as usize)
            .take(params.count.unwrap_or(20).min(100) as usize)
        {
            if let Ok(Some(resource)) = self.get_by_id(id).await {
                resources.push(resource);
            }
        }

        Ok(resources)
    }

    /// Search patients by name using extension
    pub async fn search_by_name(
        &self,
        name: Cow<'_, str>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        self.search_by_param("name", "contains", name.as_ref())
            .await
    }

    /// Search patients by gender using extension
    pub async fn search_by_gender(
        &self,
        gender: Cow<'_, str>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        self.search_by_param("gender", "exact", gender.as_ref())
            .await
    }

    /// Search patients by birth date using extension
    pub async fn search_by_birthdate(
        &self,
        birthdate: Cow<'_, str>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        self.search_by_param("birthDate", "eq", birthdate.as_ref())
            .await
    }

    /// Get patient history using extension
    pub async fn get_patient_history(
        &self,
        id: Uuid,
    ) -> Result<Vec<PatientHistoryRecord>, sqlx::Error> {
        let results = self.extension.fhir_get_history(id).await?;

        Ok(results
            .into_iter()
            .map(|(version_id, resource, timestamp, method)| PatientHistoryRecord {
                version_id,
                resource,
                timestamp,
                method,
            })
            .collect())
    }

    /// Get specific version of a patient using extension
    pub async fn get_patient_version(
        &self,
        id: Uuid,
        version_id: i32,
    ) -> Result<Option<PatientHistoryRecord>, sqlx::Error> {
        let history = self.get_patient_history(id).await?;

        Ok(history
            .into_iter()
            .find(|record| record.version_id == version_id))
    }
}

/// Search parameters with `Cow` for efficiency
#[derive(Debug, Clone)]
pub struct SearchParams<'a> {
    pub name: Option<Cow<'a, str>>,
    pub gender: Option<Cow<'a, str>>,
    pub birthdate: Option<Cow<'a, str>>,
    pub count: Option<i32>,
    pub offset: Option<i32>,
}

impl<'a> SearchParams<'a> {
    /// Create new search parameters
    pub fn new() -> Self {
        Self {
            name: None,
            gender: None,
            birthdate: None,
            count: None,
            offset: None,
        }
    }

    /// Add name parameter
    pub fn with_name<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add gender parameter
    pub fn with_gender<S: Into<Cow<'a, str>>>(mut self, gender: S) -> Self {
        self.gender = Some(gender.into());
        self
    }

    /// Add birthdate parameter
    pub fn with_birthdate<S: Into<Cow<'a, str>>>(mut self, birthdate: S) -> Self {
        self.birthdate = Some(birthdate.into());
        self
    }

    /// Add pagination count
    pub fn with_count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self
    }

    /// Add pagination offset
    pub fn with_offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl<'a> Default for SearchParams<'a> {
    fn default() -> Self {
        Self::new()
    }
}
