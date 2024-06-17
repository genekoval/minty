use super::{text::Text, AppState, Result, Router};

use crate::server::extract::{OptionalUser, User};

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
    Ok(Json(
        repo.with_user(user)
            .post(post)
            .await?
            .add_comment(content)
            .await?,
    ))
}

async fn get_comments(
    State(AppState { repo }): State<AppState>,
    Path(post): Path<Uuid>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<Vec<CommentData>>> {
    Ok(Json(
        repo.optional_user(user)?
            .post(post)
            .await?
            .get_comments()
            .await?,
    ))
}

pub fn routes() -> Router {
    Router::new().route("/:post", get(get_comments).post(add_comment))
}
