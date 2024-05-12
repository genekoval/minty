use super::{text::Text, AppState, Result, Router};

use crate::server::extract::User;

use axum::{
    extract::{Path, State},
    routing::get,
    Json,
};
use minty::{text, CommentData, Uuid};

async fn add_comment(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path(post): Path<Uuid>,
    Text(content): Text<text::Comment>,
) -> Result<Json<CommentData>> {
    Ok(Json(repo.add_comment(user, post, content).await?))
}

async fn get_comments(
    State(AppState { repo }): State<AppState>,
    Path(post): Path<Uuid>,
) -> Result<Json<Vec<CommentData>>> {
    Ok(Json(repo.get_comments(post).await?))
}

pub fn routes() -> Router {
    Router::new().route("/:post", get(get_comments).post(add_comment))
}
