use axum::response::{IntoResponse, Response};
use chrono::format::SecondsFormat;
use minty::DateTime;

pub struct Timestamp(pub DateTime);

impl From<DateTime> for Timestamp {
    fn from(value: DateTime) -> Self {
        Self(value)
    }
}

impl IntoResponse for Timestamp {
    fn into_response(self) -> Response {
        self.0
            .to_rfc3339_opts(SecondsFormat::Micros, false)
            .into_response()
    }
}
