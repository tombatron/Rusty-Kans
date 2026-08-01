use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum KanbanError {
    ListNotFound(u64),
    CardNotFound(u64),
    SerializationError(String),
    IoError(String),
    DatabaseError(String),
    TemplateError(String),
}

impl Display for KanbanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KanbanError::ListNotFound(list_id) => {
                write!(f, "List with ID ({}) was not found.", list_id)
            }
            KanbanError::CardNotFound(card_id) => {
                write!(f, "Card with ID ({}) was not found.", card_id)
            }
            KanbanError::SerializationError(serialization_error) => {
                write!(
                    f,
                    "There was an error during serialization: {}",
                    serialization_error
                )
            }
            KanbanError::IoError(io_error) => {
                write!(f, "There was an IO error: {}", io_error)
            }
            KanbanError::DatabaseError(database_error) => {
                write!(f, "There was a database error: {}", database_error)
            }
            KanbanError::TemplateError(template_error) => {
                write!(f, "There was a templating error: {}", template_error)
            }
        }
    }
}

impl From<Error> for KanbanError {
    fn from(value: Error) -> Self {
        KanbanError::SerializationError(value.to_string())
    }
}

impl From<std::io::Error> for KanbanError {
    fn from(value: std::io::Error) -> Self {
        KanbanError::IoError(value.to_string())
    }
}

impl IntoResponse for KanbanError {
    fn into_response(self) -> Response {
        match self {
            KanbanError::ListNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            KanbanError::CardNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            KanbanError::SerializationError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            KanbanError::IoError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            KanbanError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            KanbanError::TemplateError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }
    }
}

impl From<sqlx::Error> for KanbanError {
    fn from(value: sqlx::Error) -> Self {
        KanbanError::DatabaseError(value.to_string())
    }
}

impl From<askama::Error> for KanbanError {
    fn from(value: askama::Error) -> Self {
        KanbanError::TemplateError(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_not_found_message() {
        let err = KanbanError::ListNotFound(42);
        assert_eq!(err.to_string(), "List with ID (42) was not found.");
    }
}