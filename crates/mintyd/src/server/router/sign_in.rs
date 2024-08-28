use super::{
    session::{CookieJarSession, OptionalUser, SessionCookie},
    Accept, AppState, Content, Result, Router,
};

use crate::server::content::SignIn;

use axum::{extract::State, response::Redirect, routing::get, Form};
use axum_extra::extract::CookieJar;
use minty::Login;

async fn sign_in_form(
    State(AppState { repo: _ }): State<AppState>,
    OptionalUser(user): OptionalUser,
    accept: Accept,
) -> Content<SignIn> {
    Content {
        accept,
        user,
        data: SignIn,
    }
}

async fn create_session(
    State(AppState { repo }): State<AppState>,
    jar: CookieJar,
    Form(login): Form<Login>,
) -> Result<(CookieJar, Redirect)> {
    if let Some(session) = jar.get_session() {
        repo.sessions().delete(session).await?;
    }

    let session = repo.authenticate(&login).await?;

    Ok((jar.add(session.cookie()), Redirect::to("/")))
}

pub fn routes() -> Router {
    Router::new().route("/", get(sign_in_form).post(create_session))
}
