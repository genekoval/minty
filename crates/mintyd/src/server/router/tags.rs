use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{SearchResult, TagPreview, TagQuery, Uuid};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Params {
    from: Option<u32>,
    size: Option<u32>,
    name: String,
    exclude: Option<Vec<Uuid>>,
}

impl From<Params> for TagQuery {
    fn from(
        Params {
            from,
            size,
            name,
            exclude,
        }: Params,
    ) -> Self {
        Self {
            from: from.unwrap_or(0),
            size: size.unwrap_or(10),
            name,
            exclude: exclude.unwrap_or_default(),
        }
    }
}

async fn get_tags(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<Params>,
) -> Result<Json<SearchResult<TagPreview>>> {
    Ok(Json(repo.get_tags(&query.into()).await?))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_tags))
}
