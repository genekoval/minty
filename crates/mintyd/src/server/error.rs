use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;
use std::result;

pub struct Error(pub minty_core::Error);

impl From<minty_core::Error> for Error {
    fn from(value: minty_core::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        use minty_core::Error::*;

        let error = self.0;

        match error {
            InvalidInput(err) => {
                return (StatusCode::BAD_REQUEST, err).into_response()
            }
            NotFound { .. } => {
                return (StatusCode::NOT_FOUND, error.to_string())
                    .into_response()
            }
            AlreadyExists { .. } => {
                return (StatusCode::CONFLICT, error.to_string())
                    .into_response()
            }
            Unauthenticated => return StatusCode::UNAUTHORIZED.into_response(),
            _ => error!("{error}"),
        }

        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            .into_response()
    }
}

pub type Result<T> = result::Result<T, Error>;
