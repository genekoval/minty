use super::{AppState, Result, Router};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json,
};
use minty::{Modification, Post, Uuid};

async fn add_objects(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Path(destination): Path<Uuid>,
    Json(objects): Json<Vec<Uuid>>,
) -> Result<String> {
    Ok(repo
        .add_post_objects(id, &objects, Some(destination))
        .await?
        .to_string())
}

async fn append_objects(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(objects): Json<Vec<Uuid>>,
) -> Result<String> {
    Ok(repo.add_post_objects(id, &objects, None).await?.to_string())
}

async fn add_related_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Path(related): Path<Uuid>,
) -> Result<StatusCode> {
    repo.add_related_post(id, related).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_tag(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Path(tag): Path<Uuid>,
) -> Result<StatusCode> {
    repo.add_post_tag(id, tag).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_draft(
    State(AppState { repo }): State<AppState>,
) -> Result<String> {
    Ok(repo.create_post_draft().await?.to_string())
}

async fn create_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    repo.create_post(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_objects(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Json(objects): Json<Vec<Uuid>>,
) -> Result<String> {
    Ok(repo.delete_post_objects(id, &objects).await?.to_string())
}

async fn delete_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    repo.delete_post(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_related_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Path(related): Path<Uuid>,
) -> Result<StatusCode> {
    repo.delete_related_post(id, related).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_tag(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Path(tag): Path<Uuid>,
) -> Result<StatusCode> {
    repo.delete_post_tag(id, tag).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Post>> {
    Ok(Json(repo.get_post(id).await?))
}

async fn set_description(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    description: String,
) -> Result<Json<Modification<String>>> {
    Ok(Json(repo.set_post_description(id, &description).await?))
}

async fn set_title(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    title: String,
) -> Result<Json<Modification<String>>> {
    Ok(Json(repo.set_post_title(id, &title).await?))
}

pub fn routes() -> Router {
    Router::new()
        .route("/", post(create_draft))
        .route("/:id", get(get_post).put(create_post).delete(delete_post))
        .route("/:id/description", put(set_description))
        .route("/:id/objects", post(append_objects).delete(delete_objects))
        .route("/:id/objects/:destination", post(add_objects))
        .route(
            "/:id/related/:related",
            put(add_related_post).delete(delete_related_post),
        )
        .route("/:id/tag/:tag", put(add_tag).delete(delete_tag))
        .route("/:id/title", put(set_title))
}
