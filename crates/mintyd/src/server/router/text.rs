use std::str::FromStr;

use crate::server::error::Error;

use axum::{
    async_trait,
    extract::{FromRequest, Request},
};
use minty::text;

pub struct Text<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Text<T>
where
    S: Send + Sync,
    T: FromStr<Err = text::Error>,
{
    type Rejection = Error;

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let text = String::from_request(req, state)
            .await
            .map_err(|err| {
                Error(minty_core::Error::InvalidInput(err.to_string()))
            })?
            .parse()
            .map_err(|err: text::Error| {
                Error(minty_core::Error::InvalidInput(err.to_string()))
            })?;

        Ok(Text(text))
    }
}
