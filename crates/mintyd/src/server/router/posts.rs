use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{http::query::PostQuery, PostPreview, SearchResult};

async fn get_posts(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<PostQuery>,
) -> Result<Json<SearchResult<PostPreview>>> {
    Ok(Json(repo.get_posts(&query.into()).await?))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_posts))
}
