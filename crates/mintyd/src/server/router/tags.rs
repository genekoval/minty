use super::{session::OptionalUser, AppState, Result, Router};

use crate::server::error::Error;

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json,
};
use minty::{http::query::ProfileQuery, SearchResult, TagPreview, Uuid};

async fn get_tags(
    State(AppState { repo }): State<AppState>,
    Path(tags): Path<String>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<Vec<TagPreview>>> {
    let ids = tags
        .split(',')
        .map(|value| match Uuid::try_parse(value).ok() {
            Some(id) => Ok(id),
            None => Err(Error(minty_core::Error::InvalidInput(format!(
                "Bad UUID: {value}"
            )))),
        })
        .collect::<Result<Vec<Uuid>>>()?;

    Ok(Json(repo.optional_user(user)?.tags().get(&ids).await?))
}

async fn search(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<ProfileQuery>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<SearchResult<TagPreview>>> {
    Ok(Json(
        repo.optional_user(user)?.tags().find(&query.into()).await?,
    ))
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(search))
        .route("/:tags", get(get_tags))
}
