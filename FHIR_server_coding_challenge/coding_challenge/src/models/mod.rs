pub mod error;
pub mod history;
pub mod patient;

pub use error::*;
pub use history::*;
pub use patient::*;

/// Patient history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientHistoryEntry {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    #[serde(rename = "versionId")]
    pub version_id: i32,
    pub meta: HistoryMeta,
    #[serde(flatten)]
    pub resource: serde_json::Value,
}

/// Metadata for history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMeta {
    #[serde(rename = "versionId")]
    pub version_id: i32,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

/// FHIR Bundle for history results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryBundle {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    pub total: i64,
    pub entry: Vec<HistoryBundleEntry>,
}

/// Bundle entry for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryBundleEntry {
    #[serde(rename = "fullUrl")]
    pub full_url: String,
    pub resource: serde_json::Value,
}

impl HistoryBundle {
    /// Create new history bundle
    pub fn new(id: uuid::Uuid) -> Self {
        Self {
            resource_type: "Bundle".to_string(),
            bundle_type: "history".to_string(),
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