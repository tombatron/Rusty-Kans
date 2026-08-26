use std::collections::HashMap;
use axum::extract::{FromRequest, Request};
use axum::Form;
use axum::response::{IntoResponse, Response};
use garde::Validate;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

pub struct ValidForm<T>(pub T);

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

impl<S, T> FromRequest<S> for ValidForm<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate<Context = ()> + Send,
{
    type Rejection = FormError<T>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state)
            .await
            .map_err(FormError::Deserialize)?;

        match value.validate() {
            Ok(()) => Ok(ValidForm(value)),
            Err(report) => Err(FormError::Validation {
                errors: FormErrors::from_report(&report),
                input: value
            })
        }
    }
}

#[derive(Default)]
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