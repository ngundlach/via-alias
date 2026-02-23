use std::fmt;

#[derive(Debug)]
pub enum DbServiceError {
    NotFoundError,
    DatabaseError(String),
    PayloadValidationError(String, Vec<String>),
}
impl fmt::Display for DbServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbServiceError::NotFoundError => write!(f, "Resource not found"),
            DbServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DbServiceError::PayloadValidationError(s, items) => {
                let formatted_vec = items
                    .iter()
                    .map(|e| format!("[{}]", e))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Validation Error in {s}: {formatted_vec}")
            }
        }
    }
}

impl std::error::Error for DbServiceError {}

impl From<sqlx::Error> for DbServiceError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbServiceError::NotFoundError,
            _ => DbServiceError::DatabaseError(err.to_string()),
        }
    }
}
