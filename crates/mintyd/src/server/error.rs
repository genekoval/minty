use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;
use std::result;

pub struct Error(minty_core::Error);

impl From<minty_core::Error> for Error {
    fn from(value: minty_core::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        use minty_core::Error::*;

        let error = self.0;

        match &error {
            NotFound(err) => {
                return (StatusCode::NOT_FOUND, err.clone()).into_response()
            }
            _ => error!("{error}"),
        }

        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            .into_response()
    }
}

pub type Result<T> = result::Result<T, Error>;
