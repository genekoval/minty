use crate::server::{error::Error, AppState};

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use cookie::time::Duration;
use minty_core::{Cached, Error::Unauthenticated, SessionId, SessionInfo};
use std::sync::Arc;

const COOKIE: &str = "mtyid";

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

pub trait CookieJarSession {
    fn get_session(&self) -> Option<SessionId>;

    fn remove_session_cookie(self) -> Self;
}

impl CookieJarSession for CookieJar {
    fn get_session(&self) -> Option<SessionId> {
        self.get(COOKIE).and_then(|cookie| cookie.session())
    }

    fn remove_session_cookie(self) -> Self {
        let cookie = Cookie::build(COOKIE).path("/");
        self.remove(cookie)
    }
}

pub enum InvalidSession {
    BadCookie(CookieJar),
    Other(Error),
}

impl From<minty_core::Error> for InvalidSession {
    fn from(value: minty_core::Error) -> Self {
        Self::Other(Error(value))
    }
}

impl IntoResponse for InvalidSession {
    fn into_response(self) -> Response {
        match self {
            Self::BadCookie(jar) => {
                (StatusCode::UNAUTHORIZED, jar.remove_session_cookie())
                    .into_response()
            }
            Self::Other(err) => err.into_response(),
        }
    }
}

pub struct Session(pub Arc<Cached<minty_core::Session>>);

#[async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = InvalidSession;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(err) => match err {},
        };

        let Some(cookie) = jar.get(COOKIE) else {
            return Err(InvalidSession::Other(Unauthenticated(None).into()));
        };

        let Some(id) = cookie.session() else {
            return Err(InvalidSession::BadCookie(jar));
        };

        let Some(session) = state.repo.sessions().get(id).await? else {
            return Err(InvalidSession::BadCookie(jar));
        };

        Ok(Session(session))
    }
}

pub struct User(pub Arc<Cached<minty_core::User>>);

#[async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = InvalidSession;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?.0;
        Ok(User(session.user()))
    }
}

pub struct OptionalUser(pub Option<Arc<Cached<minty_core::User>>>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = InvalidSession;

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

        let Some(id) = cookie.session() else {
            return Err(InvalidSession::BadCookie(jar));
        };

        let Some(session) = state.repo.sessions().get(id).await? else {
            return Err(InvalidSession::BadCookie(jar));
        };

        Ok(Self(Some(session.user())))
    }
}
