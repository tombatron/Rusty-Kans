use std::fmt::{Display, Formatter};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Error;

#[derive(Debug, PartialEq)]
pub enum KanbanError {
    ListNotFound(u64),
    CardNotFound(u64),
    SerializationError(String),
    IoError(String),
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
                write!(f, "There was an error during serialization: {}", serialization_error)
            }
            KanbanError::IoError(io_error) => {
                write!(f, "There was an IO error: {}", io_error)
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
            KanbanError::ListNotFound(list_id) => {
                (StatusCode::NOT_FOUND, format!("List with ID ({}) was not found.", list_id)).into_response()
            }
            KanbanError::CardNotFound(card_id) => {
                (StatusCode::NOT_FOUND, format!("Card with ID ({}) was not found.", card_id)).into_response()
            }
            KanbanError::SerializationError(serialization_error) => {
                (StatusCode::BAD_REQUEST, format!("There was an error during serialization: {}", serialization_error)).into_response()
            }
            KanbanError::IoError(io_error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("There was an IO error: {}", io_error)).into_response()
            }
        }
    }
}
