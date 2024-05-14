use super::{AppState, Result, Router};

use axum::{extract::State, routing::post, Form};
use minty::Login;

async fn login(
    State(AppState { repo }): State<AppState>,
    Form(login): Form<Login>,
) -> Result<String> {
    Ok(repo.authenticate(&login).await?.to_string())
}

pub fn routes() -> Router {
    Router::new().route("/", post(login))
}
