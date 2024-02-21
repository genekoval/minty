use super::{AppState, Result, Router};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json,
};
use axum_extra::extract::OptionalQuery;
use minty::{Source, Tag, TagName, Url, Uuid};

async fn add_source(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    Json(url): Json<Url>,
) -> Result<Json<Source>> {
    Ok(Json(repo.add_tag_source(tag, &url).await?))
}

async fn add_tag(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<String>,
) -> Result<String> {
    Ok(repo.add_tag(&tag).await?.to_string())
}

async fn delete_alias(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    Path(name): Path<String>,
) -> Result<Json<TagName>> {
    Ok(Json(repo.delete_tag_alias(tag, &name).await?))
}

async fn delete_source(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    Path(source): Path<i64>,
) -> Result<StatusCode> {
    repo.delete_tag_source(tag, source).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_tag(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
) -> Result<StatusCode> {
    repo.delete_tag(tag).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn get_tag(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
) -> Result<Json<Tag>> {
    Ok(Json(repo.get_tag(tag).await?))
}

async fn set_description(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    description: String,
) -> Result<String> {
    Ok(repo.set_tag_description(tag, &description).await?)
}

async fn set_name(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    Path(name): Path<String>,
    OptionalQuery(main): OptionalQuery<bool>,
) -> Result<Json<TagName>> {
    let main = main.unwrap_or(false);

    let result = if main {
        repo.set_tag_name(tag, &name).await
    } else {
        repo.add_tag_alias(tag, &name).await
    }?;

    Ok(Json(result))
}

pub fn routes() -> Router {
    Router::new()
        .route("/:tag", get(get_tag).post(add_tag).delete(delete_tag))
        .route("/:tag/name/:name", put(set_name).delete(delete_alias))
        .route("/:tag/description", put(set_description))
        .route("/:tag/source", post(add_source))
        .route("/:tag/source/:source", delete(delete_source))
}
