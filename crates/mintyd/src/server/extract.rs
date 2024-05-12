use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use headers::{Cookie, HeaderMapExt};
use minty::Uuid;
use std::convert::Infallible;

pub struct User(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(get_user(parts).ok_or(StatusCode::UNAUTHORIZED)?))
    }
}

pub struct OptionalUser(pub Option<Uuid>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(get_user(parts)))
    }
}

fn get_user(parts: &Parts) -> Option<Uuid> {
    let cookie = parts.headers.typed_get::<Cookie>()?;
    let value = cookie.get("user")?;
    let id = Uuid::try_parse(value).ok()?;

    Some(id)
}
