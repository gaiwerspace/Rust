use serde::{Deserialize, Serialize};

/// Search bundle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBundle {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    pub total: i32,
    pub entry: Vec<SearchBundleEntry>,
}

/// Search bundle entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBundleEntry {
    pub resource: serde_json::Value,
}
