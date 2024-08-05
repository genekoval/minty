use super::{session::OptionalUser, Accept, AppState, Content, Result, Router};

use crate::server::content::PostSearchResult;

use axum::{
    extract::{Query, State},
    routing::get,
};
use minty::http::query::PostQuery;

async fn get_posts(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    accept: Accept,
    Query(query): Query<PostQuery>,
) -> Result<Content<PostSearchResult>> {
    let query: minty::PostQuery = query.into();
    let result = repo
        .optional_user(user)?
        .posts()
        .find(query.clone())
        .await?;

    Ok(Content {
        accept,
        data: PostSearchResult { query, result },
    })
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_posts))
}
