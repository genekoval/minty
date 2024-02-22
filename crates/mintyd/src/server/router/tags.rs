use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{http::query::TagQuery, SearchResult, TagPreview};

async fn get_tags(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<TagQuery>,
) -> Result<Json<SearchResult<TagPreview>>> {
    Ok(Json(repo.get_tags(&query.into()).await?))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_tags))
}
