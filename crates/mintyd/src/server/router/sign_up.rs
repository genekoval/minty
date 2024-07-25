use super::{
    session::{CookieJarSession, SessionCookie},
    AppState, Result, Router,
};

use axum::{
    extract::{Query, State},
    routing::post,
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use minty::{http::query, SignUp};

async fn sign_up(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<query::SignUp>,
    jar: CookieJar,
    Form(sign_up): Form<SignUp>,
) -> Result<(CookieJar, String)> {
    if let Some(session) = jar.get_session() {
        repo.sessions().delete(session).await?;
    }

    let invitation = query.invitation.as_deref();
    let session = repo.sign_up(sign_up, invitation).await?;

    Ok((jar.add(session.cookie()), session.user_id.to_string()))
}

pub fn routes() -> Router {
    Router::new().route("/", post(sign_up))
}
