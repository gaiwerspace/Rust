use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<HumanName>>,
    #[serde(rename = "birthDate", skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,

    #[serde(flatten, skip_serializing_if = "Map::is_empty", default)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "versionId", skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    #[serde(rename = "lastUpdated", skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(flatten, skip_serializing_if = "Map::is_empty", default)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Vec<BundleLink>>,
    pub entry: Vec<BundleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleLink {
    pub relation: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    pub resource: Patient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcome {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub issue: Vec<OperationOutcomeIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcomeIssue {
    pub severity: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coding: Option<Vec<Coding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

impl OperationOutcome {
    /// Create a new error outcome
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: "error".to_string(),
                code: code.into(),
                details: None,
                diagnostics: Some(message.into()),
                location: None,
                expression: None,
            }],
        }
    }

    /// Create outcome with location information
    pub fn error_with_location(
        code: impl Into<String>,
        message: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: "error".to_string(),
                code: code.into(),
                details: None,
                diagnostics: Some(message.into()),
                location: Some(vec![location.into()]),
                expression: None,
            }],
        }
    }

    /// Create validation error with field location
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        let field_str = field.into();
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: "error".to_string(),
                code: "invalid".to_string(),
                details: Some(CodeableConcept {
                    coding: Some(vec![Coding {
                        system: Some("http://hl7.org/fhir/issue-type".to_string()),
                        code: Some("invalid".to_string()),
                        display: Some("Invalid".to_string()),
                    }]),
                    text: None,
                }),
                diagnostics: Some(message.into()),
                location: Some(vec![field_str]),
                expression: None,
            }],
        }
    }
}

impl Patient {
    pub fn new() -> Self {
        Self {
            id: None,
            resource_type: "Patient".to_string(),
            meta: None,
            name: None,
            birth_date: None,
            gender: None,
            extra: Map::new(),
        }
    }
}

impl Default for Patient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;
    use serde_json::Value;

    #[test]
    fn test_patient_new() {
        let patient = Patient::new();
        assert_eq!(patient.resource_type, "Patient");
        assert!(patient.id.is_none());
        assert!(patient.meta.is_none());
        assert!(patient.name.is_none());
        assert!(patient.birth_date.is_none());
        assert!(patient.gender.is_none());
    }

    #[test]
    fn test_patient_default() {
        let patient = Patient::default();
        assert_eq!(patient.resource_type, "Patient");
    }

    #[test]
    fn test_patient_serialization() {
        let patient = Patient {
            id: Some("123".to_string()),
            resource_type: "Patient".to_string(),
            meta: Some(Meta {
                version_id: Some("1".to_string()),
                last_updated: None,
            }),
            name: Some(vec![HumanName {
                family: Some("Gauß".to_string()),
                given: Some(vec!["Carl".to_string()]),
                text: Some("Carl Gauß".to_string()),
                extra: Map::<String, Value>::new(),
            }]),
            birth_date: Some("1990-01-01".to_string()),
            gender: Some("male".to_string()),
            extra: Map::<String, Value>::new(),
        };

        let json = serde_json::to_string(&patient).unwrap();
        assert!(json.contains("\"resourceType\":\"Patient\""));
        assert!(json.contains("\"id\":\"123\""));
        assert!(json.contains("\"birthDate\":\"1990-01-01\""));
    }

    #[test]
    fn test_patient_deserialization() {
        let json = r#"{
            "resourceType": "Patient",
            "name": [{"family": "Smith", "given": ["Jane"]}],
            "gender": "female",
            "birthDate": "1985-05-15"
        }"#;

        let patient: Patient = serde_json::from_str(json).unwrap();
        assert_eq!(patient.resource_type, "Patient");
        assert_eq!(patient.gender, Some("female".to_string()));
        assert_eq!(patient.birth_date, Some("1985-05-15".to_string()));
        assert!(patient.name.is_some());
    }

    #[test]
    fn test_human_name_serialization() {
        let name = HumanName {
            family: Some("Johnson".to_string()),
            given: Some(vec!["Emily".to_string(), "Rose".to_string()]),
            text: Some("Emily Rose Johnson".to_string()),
            extra: Map::<String, Value>::new(),
        };

        let json = serde_json::to_value(&name).unwrap();
        assert_eq!(json["family"], "Johnson");
        assert_eq!(json["given"][0], "Emily");
        assert_eq!(json["given"][1], "Rose");
    }

    #[test]
    fn test_bundle_creation() {
        let patient = Patient::new();
        let bundle = Bundle {
            resource_type: "Bundle".to_string(),
            bundle_type: "searchset".to_string(),
            total: 1,
            link: None,
            entry: vec![BundleEntry { resource: patient }],
        };

        assert_eq!(bundle.resource_type, "Bundle");
        assert_eq!(bundle.bundle_type, "searchset");
        assert_eq!(bundle.total, 1);
        assert_eq!(bundle.entry.len(), 1);
    }

    #[test]
    fn test_operation_outcome() {
        let outcome = OperationOutcome {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: "error".to_string(),
                code: "not-found".to_string(),
                details: Some(CodeableConcept {
                    coding: None,
                    text: Some("Resource not found".to_string()),
                }),
                diagnostics: Some("Patient with ID xyz not found".to_string()),
                location: None,
                expression: None,
            }],
        };

        assert_eq!(outcome.resource_type, "OperationOutcome");
        assert_eq!(outcome.issue.len(), 1);
        assert_eq!(outcome.issue[0].severity, "error");
    }

    #[test]
    fn test_skip_serializing_none_fields() {
        let patient = Patient {
            id: None,
            resource_type: "Patient".to_string(),
            meta: None,
            name: None,
            birth_date: None,
            gender: None,
            extra: Map::<String, Value>::new(),
        };

        let json = serde_json::to_string(&patient).unwrap();
        assert!(!json.contains("\"id\""));
        assert!(!json.contains("\"meta\""));
        assert!(!json.contains("\"name\""));
        assert!(!json.contains("\"birthDate\""));
        assert!(!json.contains("\"gender\""));
    }

    #[test]
    fn test_meta_with_timestamp() {
        let now = Utc::now();
        let meta = Meta {
            version_id: Some("2".to_string()),
            last_updated: Some(now),
        };

        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["versionId"], "2");
        assert!(json["lastUpdated"].is_string());
    }
}
