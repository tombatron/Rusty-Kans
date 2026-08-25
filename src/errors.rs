use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Error;
use std::fmt::{Display, Formatter};
use sqlx::migrate::MigrateError;

#[derive(Debug, PartialEq)]
pub enum KanbanError {
    BoardNotFound(u64),
    ListNotFound(u64),
    CardNotFound(u64),
    SerializationError(String),
    IoError(String),
    DatabaseError(String),
    TemplateError(String),
    RequestError(String),
}

impl Display for KanbanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KanbanError::BoardNotFound(board_id) => {
                write!(f, "Board with ID ({}) was not found.", board_id)
            }
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
            KanbanError::RequestError(request_error) => {
                write!(f, "There was a request error: {}", request_error)
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

impl From<tower_sessions::session::Error> for KanbanError {
    fn from(value: tower_sessions::session::Error) -> Self {
        KanbanError::RequestError(value.to_string())
    }
}

impl From<MigrateError> for KanbanError {
    fn from(value: MigrateError) -> Self {
        KanbanError::DatabaseError(value.to_string())
    }
}

impl From<KanbanError> for (StatusCode, String) {
    fn from(value: KanbanError) -> Self {
        let error_message = value.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, error_message)
    }
}

impl IntoResponse for KanbanError {
    fn into_response(self) -> Response {
        match self {
            KanbanError::BoardNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
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
            KanbanError::RequestError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
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
    use crate::errors::KanbanError;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use sqlx::SqlitePool;
    use std::fs;
    use test_case::test_case;

    #[test_case(KanbanError::BoardNotFound(67), "Board with ID (67) was not found.".to_string())]
    #[test_case(KanbanError::ListNotFound(67), "List with ID (67) was not found.".to_string())]
    #[test_case(KanbanError::CardNotFound(67), "Card with ID (67) was not found.".to_string())]
    #[test_case(KanbanError::SerializationError("Something totally went wrong.".to_string()), "There was an error during serialization: Something totally went wrong.".to_string())]
    #[test_case(KanbanError::IoError("I can't even read this thing.".to_string()), "There was an IO error: I can't even read this thing.".to_string())]
    #[test_case(KanbanError::DatabaseError("Tables be locked yo.".to_string()), "There was a database error: Tables be locked yo.".to_string())]
    #[test_case(KanbanError::TemplateError("Template is totally messed up man.".to_string()), "There was a templating error: Template is totally messed up man.".to_string())]
    #[test_case(KanbanError::RequestError("We don't even speak that language.".to_string()), "There was a request error: We don't even speak that language.".to_string())]
    fn kanban_display_impl_verification(error: KanbanError, expected: String) {
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn can_create_kanban_error_from_serde_error() {
        let error = serde_json::from_str::<serde_json::Value>("whatever").unwrap_err();

        let result = KanbanError::from(error);

        assert!(matches!(result, KanbanError::SerializationError(_)));
    }

    #[test]
    fn can_create_kanban_error_from_io_error() {
        let error = fs::read("/this/doesnt/exists/probably").unwrap_err();

        let result = KanbanError::from(error);

        assert!(matches!(result, KanbanError::IoError(_)))
    }

    #[test_case(KanbanError::BoardNotFound(67), StatusCode::NOT_FOUND)]
    #[test_case(KanbanError::ListNotFound(67), StatusCode::NOT_FOUND)]
    #[test_case(KanbanError::CardNotFound(67), StatusCode::NOT_FOUND)]
    #[test_case(KanbanError::SerializationError("Couldn't serialize.".to_string()), StatusCode::BAD_REQUEST)]
    #[test_case(KanbanError::IoError("Couldn't read the file.".to_string()), StatusCode::INTERNAL_SERVER_ERROR)]
    #[test_case(KanbanError::DatabaseError("Tables be locked yo.".to_string()), StatusCode::INTERNAL_SERVER_ERROR)]
    #[test_case(KanbanError::TemplateError("I'm not even supposed to be here today.".to_string()), StatusCode::INTERNAL_SERVER_ERROR)]
    #[test_case(KanbanError::RequestError("I'm not ready to receive you.".to_string()), StatusCode::BAD_REQUEST)]
    fn kanban_into_response_impl_verification(error: KanbanError, status_code: StatusCode) {
        let result = error.into_response();

        assert_eq!(status_code, result.status());
    }

    #[sqlx::test]
    async fn can_create_kanban_error_from_sql_error(db: SqlitePool) -> sqlx::Result<()> {
        let result = sqlx::query("SELECT FACE FROM YOUR")
            .execute(&db)
            .await
            .unwrap_err();

        let kb_error = KanbanError::from(result);

        assert!(matches!(kb_error, KanbanError::DatabaseError(_)));

        Ok(())
    }

    #[sqlx::test]
    fn can_create_kanban_error_from_askama_error() {
        let error = askama::Error::from(std::fmt::Error);

        let result = KanbanError::from(error);

        assert!(matches!(result, KanbanError::TemplateError(_)));
    }
}
