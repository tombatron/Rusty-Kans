use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::fmt::Debug;
use serde::Deserialize;

#[derive(Debug)]
pub enum FormError<T> {
    Deserialize(axum::extract::rejection::FormRejection),
    Validation { input: T, errors: FormErrors }
}

impl<T> IntoResponse for FormError<T> {
    fn into_response(self) -> Response {
        match self {
            FormError::Deserialize(rejection) => rejection.into_response(),
            FormError::Validation { errors, .. } => {
                let body = errors
                    .0
                    .iter()
                    .map(|(field, msgs)| format!("{field}: {}", msgs.join(",")))
                    .collect::<Vec<_>>()
                    .join("\n");

                (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
            }
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct FormErrors(HashMap<String, Vec<String>>);

impl FormErrors {
    pub fn from_report(report: &garde::Report) -> Self {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for (path, error) in report.iter() {
            map.entry(path.to_string()).or_default().push(error.to_string());
        }

        Self(map)
    }

    pub fn get(&self, field: &str) -> Option<&Vec<String>> {
        self.0.get(field)
    }

    pub fn has(&self, field: &str) -> bool {
        self.0.contains_key(field)
    }
}