use super::AppState;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::ACCEPT, request::Parts, StatusCode},
};
use mediatype::{media_type, MediaType, MediaTypeList};

const JSON: MediaType = media_type!(APPLICATION / JSON);

pub enum Accept {
    Html,
    Json,
}

impl Default for Accept {
    fn default() -> Self {
        Self::Html
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Accept {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get(ACCEPT) else {
            return Ok(Self::default());
        };

        let Some(value) = header.to_str().ok() else {
            return Err((
                StatusCode::BAD_REQUEST,
                "Accept header contains invalid characters".into(),
            ));
        };

        let types = MediaTypeList::new(value);

        for ty in types {
            let ty = ty.map_err(|err| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Bad media type in Accept header: {err}"),
                )
            })?;

            if ty == JSON {
                return Ok(Self::Json);
            }
        }

        Ok(Self::default())
    }
}
