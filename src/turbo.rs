use axum::http::header;
use axum::response::{IntoResponse, Response};

pub struct TurboStream(pub String);

impl IntoResponse for TurboStream {
    fn into_response(self) -> Response {
        (
            [(header::CONTENT_TYPE, "text/vnd.turbo-stream.html")],
            self.0,
        )
            .into_response()
    }
}
