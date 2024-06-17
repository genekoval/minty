use super::{AppState, Result, Router};

use axum::{extract::State, routing::post, Form};
use minty::SignUp;

async fn sign_up(
    State(AppState { repo }): State<AppState>,
    Form(sign_up): Form<SignUp>,
) -> Result<String> {
    Ok(repo.sign_up(sign_up).await?.to_string())
}

pub fn routes() -> Router {
    Router::new().route("/", post(sign_up))
}
