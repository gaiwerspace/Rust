use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Bundle entry for history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryBundleEntry {
    #[serde(rename = "fullUrl")]
    pub full_url: String,
    pub resource: serde_json::Value,
    pub request: Option<HistoryRequest>,
    pub response: Option<HistoryResponse>,
}

/// Request information for history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub method: String,
    pub url: String,
}

/// Response information for history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub status: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
}

/// FHIR Bundle for history results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryBundle {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    pub id: String,
    pub meta: HistoryBundleMeta,
    pub total: i64,
    pub entry: Vec<HistoryBundleEntry>,
}

/// Metadata for history bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryBundleMeta {
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

impl HistoryBundle {
    /// Create new history bundle
    pub fn new(id: uuid::Uuid) -> Self {
        Self {
            resource_type: "Bundle".to_string(),
            bundle_type: "history".to_string(),
            id: id.to_string(),
            meta: HistoryBundleMeta {
                last_updated: Utc::now().to_rfc3339(),
            },
            total: 0,
            entry: Vec::new(),
        }
    }

    /// Add history entry to bundle
    pub fn add_entry(mut self, entry: HistoryBundleEntry) -> Self {
        self.total += 1;
        self.entry.push(entry);
        self
    }
}

/// Patient history record from database
#[derive(Debug, Clone)]
pub struct PatientHistoryRecord {
    pub version_id: i32,
    pub resource: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub method: String,
}
