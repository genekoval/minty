use super::AppState;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::ACCEPT, request::Parts, StatusCode},
};
use mediatype::{media_type, MediaType, MediaTypeList};

/// A request header that indicates the request is via an element
/// using hx-boost.
const HX_BOOSTED: &str = "HX_Boosted";

/// A request header that accompanies every request made by HTMX.
/// The value is always "true".
const HX_REQUEST: &str = "HX-Request";

const JSON: MediaType = media_type!(APPLICATION / JSON);

#[derive(Clone, Copy, Debug)]
pub enum Accept {
    Html,
    Boosted,
    Fragment,
    Json,
}

impl Accept {
    pub fn is_api(self) -> bool {
        matches!(self, Self::Json)
    }
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
        if parts.headers.get(HX_BOOSTED).is_some() {
            return Ok(Self::Boosted);
        }

        if let Some(header) = parts.headers.get(HX_REQUEST) {
            if let Ok(value) = header.to_str() {
                if value == "true" {
                    return Ok(Self::Fragment);
                }
            }
        }

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
