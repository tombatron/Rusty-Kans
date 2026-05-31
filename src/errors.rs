use std::fmt::{Display, Formatter};
use serde_json::Error;

pub enum KanbanError {
    BoardNotFound,
    ListNotFound(u64),
    CardNotFound(u64),
    SerializationError(String),
    IoError(String),
}

impl Display for KanbanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KanbanError::BoardNotFound => {
                write!(f, "No board was found.")
            }
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