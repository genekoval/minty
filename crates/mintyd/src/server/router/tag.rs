use super::{text::Text, AppState, Result, Router};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json,
};
use minty::{http::query::SetTagName, text, Source, Tag, TagName, Url, Uuid};

async fn add_source(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    Json(url): Json<Url>,
) -> Result<Json<Source>> {
    Ok(Json(repo.add_tag_source(tag, &url).await?))
}

async fn add_tag(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<text::TagName>,
) -> Result<String> {
    Ok(repo.add_tag(tag).await?.to_string())
}

async fn delete_alias(
    State(AppState { repo }): State<AppState>,
    Path((tag, name)): Path<(Uuid, String)>,
) -> Result<Json<TagName>> {
    Ok(Json(repo.delete_tag_alias(tag, &name).await?))
}

async fn delete_source(
    State(AppState { repo }): State<AppState>,
    Path((tag, source)): Path<(Uuid, i64)>,
) -> Result<StatusCode> {
    repo.delete_tag_source(tag, source).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_sources(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<Uuid>,
    Json(sources): Json<Vec<String>>,
) -> Result<StatusCode> {
    repo.delete_tag_sources(tag, &sources).await?;
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
    Text(description): Text<text::Description>,
) -> Result<String> {
    Ok(repo.set_tag_description(tag, description).await?)
}

async fn set_name(
    State(AppState { repo }): State<AppState>,
    Path((tag, name)): Path<(Uuid, text::TagName)>,
    Query(SetTagName { main }): Query<SetTagName>,
) -> Result<Json<TagName>> {
    let main = main.unwrap_or(false);

    let result = if main {
        repo.set_tag_name(tag, name).await
    } else {
        repo.add_tag_alias(tag, name).await
    }?;

    Ok(Json(result))
}

pub fn routes() -> Router {
    Router::new()
        .route("/:tag", get(get_tag).post(add_tag).delete(delete_tag))
        .route("/:tag/name/:name", put(set_name).delete(delete_alias))
        .route("/:tag/description", put(set_description))
        .route("/:tag/source", post(add_source).delete(delete_sources))
        .route("/:tag/source/:source", delete(delete_source))
}
