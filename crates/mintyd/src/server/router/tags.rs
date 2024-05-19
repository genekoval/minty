use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{http::query::ProfileQuery, SearchResult, TagPreview};

async fn get_tags(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<ProfileQuery>,
) -> Result<Json<SearchResult<TagPreview>>> {
    Ok(Json(repo.tags().find(&query.into()).await?))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_tags))
}
