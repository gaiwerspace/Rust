use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::borrow::Cow;
use uuid::Uuid;

/// FHIR Patient resource wrapper
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatientResource {
    pub id: Uuid,
    pub resource_type: String,
    pub resource: serde_json::Value,
    pub txid: i64,
    pub ts: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

/// FHIR Patient history entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatientHistory {
    pub id: Uuid,
    pub version_id: i32,
    pub resource: serde_json::Value,
    pub txid: i64,
    pub ts: chrono::DateTime<chrono::Utc>,
    pub status: Option<String>,
}

/// Search parameters with `Cow` for efficiency
#[derive(Debug, Clone)]
pub struct SearchParams<'a> {
    pub name: Option<Cow<'a, str>>,
    pub gender: Option<Cow<'a, str>>,
    pub birthdate: Option<Cow<'a, str>>,
    pub identifier: Option<Cow<'a, str>>,
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
            identifier: None,
            count: None,
            offset: None,
        }
    }

    /// Add name parameter (accepts both owned and borrowed strings)
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

    /// Add identifier parameter
    pub fn with_identifier<S: Into<Cow<'a, str>>>(mut self, identifier: S) -> Self {
        self.identifier = Some(identifier.into());
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
