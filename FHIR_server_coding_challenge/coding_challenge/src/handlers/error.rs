use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::models::OperationOutcome;

/// Custom error response with FHIR OperationOutcome
pub struct FhirError {
    pub status: StatusCode,
    pub outcome: OperationOutcome,
}

impl FhirError {
    /// Create validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            outcome: OperationOutcome::validation_error(field, message),
        }
    }

    /// Create not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            outcome: OperationOutcome::error("not-found", message),
        }
    }

    /// Create server error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            outcome: OperationOutcome::error("exception", message),
        }
    }

    /// Create conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            outcome: OperationOutcome::error("conflict", message),
        }
    }
}

impl IntoResponse for FhirError {
    fn into_response(self) -> Response {
        (self.status, Json(self.outcome)).into_response()
    }
}
