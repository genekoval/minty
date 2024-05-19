use super::{AppState, Result, Router};

use axum::{extract::State, routing::post, Form};
use minty::SignUp;

async fn sign_up(
    State(AppState { repo }): State<AppState>,
    Form(sign_up): Form<SignUp>,
) -> Result<String> {
    let user_id = repo.users().add(sign_up).await?;
    let session = repo.user(user_id).create_session().await?;
    Ok(session.to_string())
}

pub fn routes() -> Router {
    Router::new().route("/", post(sign_up))
}
