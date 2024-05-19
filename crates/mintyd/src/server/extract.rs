use crate::server::{error::Error, AppState};

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderValue},
};
use headers::{
    authorization::Credentials, Authorization, Cookie, HeaderMapExt,
};
use minty::Uuid;
use minty_core::{Base64DecodeError, Error::Unauthenticated, SessionId};
use std::str;

const COOKIE: &str = "mtyid";

pub struct Session(pub SessionId);

#[async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match try_get_session(parts) {
            Ok(Some(session)) => Ok(Self(session)),
            Ok(None) => Err(Unauthenticated(None).into()),
            Err(err) => Err(err.into()),
        }
    }
}

pub struct User(pub Uuid);

#[async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match try_get_user(parts, state).await {
            Ok(Some(id)) => Ok(Self(id)),
            Ok(None) => Err(Unauthenticated(None).into()),
            Err(err) => Err(err.into()),
        }
    }
}

pub struct OptionalUser(pub Option<Uuid>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(try_get_user(parts, state).await?))
    }
}

struct Key(HeaderValue);

impl Key {
    fn value(&self) -> &str {
        // Key is only created from HeaderValues that have validated
        // they are also UTF-8 strings.
        let text = unsafe { str::from_utf8_unchecked(self.0.as_bytes()) };
        text["Key ".len()..].trim_start()
    }
}

impl Credentials for Key {
    const SCHEME: &'static str = "Key";

    fn decode(value: &HeaderValue) -> Option<Self> {
        if value.to_str().is_ok() {
            Some(Self(value.clone()))
        } else {
            None
        }
    }

    fn encode(&self) -> HeaderValue {
        self.0.clone()
    }
}

fn try_get_authorization(
    parts: &Parts,
) -> Option<Result<SessionId, Base64DecodeError>> {
    Some(
        parts
            .headers
            .typed_get::<Authorization<Key>>()?
            .0
            .value()
            .parse(),
    )
}

fn try_get_cookie(
    parts: &Parts,
) -> Option<Result<SessionId, Base64DecodeError>> {
    Some(parts.headers.typed_get::<Cookie>()?.get(COOKIE)?.parse())
}

fn try_get_session(
    parts: &Parts,
) -> Result<Option<SessionId>, minty_core::Error> {
    try_get_authorization(parts)
        .or_else(|| try_get_cookie(parts))
        .transpose()
        .map_err(|_| Unauthenticated(Some("invalid key")))
}

async fn try_get_user(
    parts: &Parts,
    AppState { repo }: &AppState,
) -> Result<Option<Uuid>, minty_core::Error> {
    let Some(session) = try_get_session(parts)? else {
        return Ok(None);
    };

    let id = repo.get_user_session(session).await?;

    Ok(Some(id))
}
