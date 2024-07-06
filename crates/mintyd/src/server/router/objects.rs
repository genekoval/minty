use super::{session::User, AppState, Result, Router};

use axum::{extract::State, routing::get, Json};
use minty::ObjectError;

async fn get_preview_errors(
    State(AppState { repo }): State<AppState>,
    User(admin): User,
) -> Result<Json<Vec<ObjectError>>> {
    Ok(Json(
        repo.admin(admin)?.objects().get_preview_errors().await?,
    ))
}

pub fn routes() -> Router {
    Router::new().route("/errors", get(get_preview_errors))
}
