use std::fmt;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DbServiceError {
    NotFoundError,
    DatabaseError(String),
    PayloadValidationError(String, Vec<String>),
    AuthError(String),
    PermissionError(String),
    TokenInvalid,
    ResourceConflict,
    InvalidCredentials,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ValidationErrorResponse {
    pub(crate) on_item: String,
    pub(crate) errors: Vec<String>,
}

impl fmt::Display for DbServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbServiceError::NotFoundError => write!(f, "Resource not found"),
            DbServiceError::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            DbServiceError::PayloadValidationError(s, items) => {
                let formatted_vec = items
                    .iter()
                    .map(|e| format!("[{e}]"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Validation Error in {s}: {formatted_vec}")
            }
            DbServiceError::AuthError(msg) => write!(f, "Auth error: {msg}"),
            DbServiceError::TokenInvalid => write!(f, "Token is invalid"),
            DbServiceError::PermissionError(msg) => write!(f, "Permission error: {msg}"),
            DbServiceError::ResourceConflict => write!(f, "Resource already exists"),
            DbServiceError::InvalidCredentials => write!(f, "Wrong username or password"),
        }
    }
}
impl IntoResponse for DbServiceError {
    fn into_response(self) -> Response {
        match self {
            DbServiceError::NotFoundError => StatusCode::NOT_FOUND.into_response(),
            DbServiceError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            DbServiceError::PayloadValidationError(s, e) => {
                let errors = ValidationErrorResponse {
                    on_item: s,
                    errors: e,
                };
                (StatusCode::BAD_REQUEST, Json(errors)).into_response()
            }
            DbServiceError::AuthError(_) => StatusCode::UNAUTHORIZED.into_response(),
            DbServiceError::PermissionError(_) | DbServiceError::TokenInvalid => {
                StatusCode::FORBIDDEN.into_response()
            }
            DbServiceError::ResourceConflict => StatusCode::CONFLICT.into_response(),
            DbServiceError::InvalidCredentials => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}
impl std::error::Error for DbServiceError {}

impl From<sqlx::Error> for DbServiceError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbServiceError::NotFoundError,
            sqlx::Error::Database(e) => {
                if e.is_unique_violation() {
                    DbServiceError::ResourceConflict
                } else {
                    DbServiceError::DatabaseError(e.to_string())
                }
            }
            _ => DbServiceError::DatabaseError(err.to_string()),
        }
    }
}
