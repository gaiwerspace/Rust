use serde::{Deserialize, Serialize};

/// FHIR OperationOutcome for error responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcome {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub issue: Vec<OperationOutcomeIssue>,
}

/// Individual issue in OperationOutcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutcomeIssue {
    pub severity: String,
    pub code: String,
    pub details: Option<CodeableConcept>,
    pub diagnostics: Option<String>,
    pub location: Option<Vec<String>>,
    pub expression: Option<Vec<String>>,
}

/// CodeableConcept for issue details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    pub coding: Option<Vec<Coding>>,
    pub text: Option<String>,
}

/// Coding for issue classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
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
