use super::{AppState, Result, Router};

use axum::{
    extract::{Path, State},
    routing::get,
    Json,
};
use minty::{CommentData, Uuid};

async fn get_comments(
    State(AppState { repo }): State<AppState>,
    Path(post): Path<Uuid>,
) -> Result<Json<Vec<CommentData>>> {
    Ok(Json(repo.get_comments(post).await?))
}

pub fn routes() -> Router {
    Router::new().route("/:post", get(get_comments))
}
