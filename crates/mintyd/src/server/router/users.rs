use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{http::query::ProfileQuery, SearchResult, UserPreview};

async fn get_users(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<ProfileQuery>,
) -> Result<Json<SearchResult<UserPreview>>> {
    Ok(Json(repo.get_users(&query.into()).await?))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_users))
}
