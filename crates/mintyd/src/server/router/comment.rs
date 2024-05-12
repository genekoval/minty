use super::{text::Text, AppState, Result, Router};

use crate::server::extract::User;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json,
};
use minty::{http::query::DeleteComment, text, Comment, CommentData, Uuid};

async fn add_reply(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path(id): Path<Uuid>,
    Text(content): Text<text::Comment>,
) -> Result<Json<CommentData>> {
    Ok(Json(repo.add_reply(user, id, content).await?))
}

async fn delete_comment(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Query(DeleteComment { recursive }): Query<DeleteComment>,
) -> Result<StatusCode> {
    let recursive = recursive.unwrap_or(false);
    let deleted = repo.delete_comment(id, recursive).await?;

    let status = if deleted {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    };

    Ok(status)
}

async fn get_comment(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Comment>> {
    Ok(Json(repo.get_comment(id).await?))
}

async fn set_comment_content(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Text(content): Text<text::Comment>,
) -> Result<String> {
    Ok(repo.set_comment_content(id, content).await?)
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
