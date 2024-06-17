use super::{AppState, Result, Router};

use crate::server::extract::OptionalUser;

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{http::query::ProfileQuery, SearchResult, TagPreview};

async fn get_tags(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<ProfileQuery>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<SearchResult<TagPreview>>> {
    Ok(Json(
        repo.optional_user(user)?.tags().find(&query.into()).await?,
    ))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_tags))
}
