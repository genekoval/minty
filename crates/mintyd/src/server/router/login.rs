use super::{
    session::{CookieJarSession, SessionCookie},
    AppState, Result, Router,
};

use axum::{extract::State, routing::post, Form};
use axum_extra::extract::cookie::CookieJar;
use minty::Login;

async fn login(
    State(AppState { repo }): State<AppState>,
    jar: CookieJar,
    Form(login): Form<Login>,
) -> Result<(CookieJar, String)> {
    if let Some(session) = jar.get_session() {
        repo.sessions().delete(session).await?;
    }

    let session = repo.authenticate(&login).await?;

    Ok((jar.add(session.cookie()), session.user_id.to_string()))
}

pub fn routes() -> Router {
    Router::new().route("/", post(login))
}
