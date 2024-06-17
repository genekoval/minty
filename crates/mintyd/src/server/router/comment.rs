use super::{text::Text, AppState, Result, Router};

use crate::server::extract::{OptionalUser, User};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json,
};
use minty::{http::query::DeleteComment, text, Comment, CommentData, Uuid};

async fn add_reply(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    User(user): User,
    Text(content): Text<text::Comment>,
) -> Result<Json<CommentData>> {
    Ok(Json(repo.with_user(user).comment(id).reply(content).await?))
}

async fn delete_comment(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Query(DeleteComment { recursive }): Query<DeleteComment>,
    User(user): User,
) -> Result<StatusCode> {
    let recursive = recursive.unwrap_or(false);
    repo.with_user(user).comment(id).delete(recursive).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_comment(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<Comment>> {
    Ok(Json(repo.optional_user(user)?.comment(id).get().await?))
}

async fn set_comment_content(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    User(user): User,
    Text(content): Text<text::Comment>,
) -> Result<String> {
    Ok(repo
        .with_user(user)
        .comment(id)
        .set_content(content)
        .await?)
}

pub fn routes() -> Router {
    Router::new().route(
        "/:id",
        get(get_comment)
            .post(add_reply)
            .put(set_comment_content)
            .delete(delete_comment),
    )
}
