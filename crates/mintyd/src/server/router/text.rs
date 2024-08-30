use super::AppState;

use crate::server::error::Error;

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::header::CONTENT_TYPE,
};
use mediatype::{media_type, MediaType, MediaTypeBuf};
use minty::text;
use serde::de::DeserializeOwned;
use std::str::FromStr;

const FORM: MediaType = media_type!(APPLICATION / x_::WWW_FORM_URLENCODED);
const TEXT: MediaType = media_type!(TEXT / PLAIN);

pub struct Text<T>(pub T);

#[async_trait]
impl<T> FromRequest<AppState> for Text<T>
where
    T: DeserializeOwned + FromStr<Err = text::Error>,
{
    type Rejection = Error;

    async fn from_request(
        req: Request,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let content_type: MediaTypeBuf = req
            .headers()
            .get(CONTENT_TYPE)
            .ok_or_else(|| {
                minty_core::Error::InvalidInput(
                    "Expected a content-type header".into(),
                )
            })?
            .to_str()
            .map_err(|_| {
                minty_core::Error::InvalidInput(
                    "Content-Type header contains invalid characters".into(),
                )
            })?
            .parse()
            .map_err(|_| {
                minty_core::Error::InvalidInput(
                    "Content-Type header contains invalid media type".into(),
                )
            })?;

        let content_type: MediaType = (&content_type).into();

        let mut data =
            String::from_request(req, state).await.map_err(|err| {
                Error(minty_core::Error::InvalidInput(err.to_string()))
            })?;

        if content_type == FORM {
            data = serde_urlencoded::from_str::<Vec<(String, String)>>(&data)
                .map_err(|err| {
                    minty_core::Error::InvalidInput(format!(
                        "invalid form data: {err}"
                    ))
                })?
                .pop()
                .ok_or_else(|| {
                    minty_core::Error::InvalidInput(
                        "no parameters given".into(),
                    )
                })?
                .1;
        } else if content_type != TEXT {
            return Err(minty_core::Error::InvalidInput(
                "invalid content type".into(),
            )
            .into());
        }

        let text = data.parse().map_err(|err: text::Error| {
            Error(minty_core::Error::InvalidInput(err.to_string()))
        })?;

        Ok(Text(text))
    }
}
