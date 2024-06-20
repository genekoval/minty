use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::post,
    Form,
};
use minty::{http::query, SignUp};

async fn sign_up(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<query::SignUp>,
    Form(sign_up): Form<SignUp>,
) -> Result<String> {
    let invitation = query.invitation.as_deref();
    Ok(repo.sign_up(sign_up, invitation).await?.to_string())
}

pub fn routes() -> Router {
    Router::new().route("/", post(sign_up))
}
