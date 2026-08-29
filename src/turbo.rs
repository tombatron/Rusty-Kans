use askama::Template;
use axum::http::header;
use axum::response::{IntoResponse, Response};

pub struct TurboStream(pub String);

impl IntoResponse for TurboStream {
    fn into_response(self) -> Response {
        ([(header::CONTENT_TYPE, "text/vnd.turbo-stream.html")], self.0,).into_response()
    }
}

#[derive(Debug, Template)]
#[template(path="turbo_form_validation_error.html")]
pub struct TurboFormValidationErrorTemplate {
    target_id: String,
    report: garde::Report,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn turbo_stream_into_response_impl_validation() {
        let turbo_stream_response = TurboStream("This is a fake turbo stream.".to_string());

        let impl_response = turbo_stream_response.into_response();

        assert_eq!("text/vnd.turbo-stream.html", impl_response.headers()[header::CONTENT_TYPE]);
    }
}