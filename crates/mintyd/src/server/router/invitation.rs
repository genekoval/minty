use super::{session::User, AppState, Result, Router};

use axum::{
    extract::{Path, State},
    routing::get,
    Json,
};

async fn generate_invitation(
    State(AppState { repo }): State<AppState>,
    User(user): User,
) -> Result<String> {
    Ok(repo.with_user(user).invite()?)
}

async fn get_inviter(
    State(AppState { repo }): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<minty::User>> {
    Ok(Json(repo.get_inviter(&token).await?))
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(generate_invitation))
        .route("/:token", get(get_inviter))
}
