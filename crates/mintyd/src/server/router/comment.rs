use super::{AppState, Result, Router};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json,
};
use axum_extra::extract::OptionalQuery;
use minty::{Comment, CommentData, Uuid};

async fn add_reply(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    content: String,
) -> Result<Json<CommentData>> {
    Ok(Json(repo.add_reply(id, &content).await?))
}

async fn delete_comment(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    OptionalQuery(recursive): OptionalQuery<bool>,
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
    content: String,
) -> Result<String> {
    Ok(repo.set_comment_content(id, &content).await?)
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
