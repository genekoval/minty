mod comment;
mod comments;
mod object;
mod post;
mod posts;
mod tag;
mod tags;

use super::{error::Result, AppState};

use axum::{extract::State, routing::get, Json};
use minty_core::About;

pub type Router = axum::Router<AppState>;

async fn about(State(AppState { repo }): State<AppState>) -> Json<About> {
    Json(*repo.about())
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(about))
        .nest("/comment", comment::routes())
        .nest("/comments", comments::routes())
        .nest("/object", object::routes())
        .nest("/post", post::routes())
        .nest("/posts", posts::routes())
        .nest("/tag", tag::routes())
        .nest("/tags", tags::routes())
}
