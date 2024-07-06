use crate::server::{error::Error, AppState};

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use cookie::time::Duration;
use minty_core::{Cached, Error::Unauthenticated, SessionId, SessionInfo};
use std::sync::Arc;

pub const COOKIE: &str = "mtyid";

pub trait SessionCookie {
    fn cookie(&self) -> Cookie<'static>;
}

impl SessionCookie for SessionInfo {
    fn cookie(&self) -> Cookie<'static> {
        Cookie::build((COOKIE, self.id.to_string()))
            .path("/")
            .secure(true)
            .http_only(true)
            .max_age(Duration::seconds(self.max_age.num_seconds()))
            .build()
    }
}

pub trait CookieSession {
    fn session(&self) -> Option<SessionId>;
}

impl CookieSession for Cookie<'static> {
    fn session(&self) -> Option<SessionId> {
        self.value_trimmed().parse().ok()
    }
}

pub struct Session(pub Arc<Cached<minty_core::Session>>);

#[async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(err) => match err {},
        };

        let Some(id) = jar.get(COOKIE).and_then(|cookie| cookie.session())
        else {
            return Err(Unauthenticated(None).into());
        };

        let session = state.repo.sessions().get(id).await?;

        Ok(Session(session))
    }
}

pub struct User(pub Arc<Cached<minty_core::User>>);

#[async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?.0;
        let user = session.user();

        Ok(User(user))
    }
}

pub struct OptionalUser(pub Option<Arc<Cached<minty_core::User>>>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(err) => match err {},
        };

        let Some(cookie) = jar.get(COOKIE) else {
            return Ok(Self(None));
        };

        let Ok(id) = cookie.value().parse::<SessionId>() else {
            return Err(Unauthenticated(None).into());
        };

        let session = state.repo.sessions().get(id).await?;
        let user = session.user();

        Ok(Self(Some(user)))
    }
}
