use super::{AppState, Result, Router};

use crate::server::extract::OptionalUser;

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{http::query::PostQuery, PostPreview, SearchResult};

async fn get_posts(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    Query(query): Query<PostQuery>,
) -> Result<Json<SearchResult<PostPreview>>> {
    Ok(Json(
        repo.posts()
            .find(user.map(|user| user.id), query.into())
            .await?,
    ))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_posts))
}
