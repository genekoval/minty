mod comment;
mod comments;
mod invitation;
mod login;
mod object;
mod objects;
mod post;
mod posts;
mod session;
mod sign_up;
mod tag;
mod tags;
mod text;
mod timestamp;
mod user;
mod users;

use session::OptionalUser;

use super::{
    content::{Content, Home},
    error::Result,
    Accept, AppState,
};

use axum::{extract::State, routing::get, Json};
use minty::model::export::Data;
use minty_core::About;
use std::path::Path;
use tower_http::services::ServeDir;

pub type Router = axum::Router<AppState>;

async fn home(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    accept: Accept,
) -> Result<Content<Home>> {
    let posts = repo
        .optional_user(user)?
        .posts()
        .find(Default::default())
        .await?;

    Ok(Content {
        accept,
        data: Home(posts),
    })
}

async fn about(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
) -> Result<Json<About>> {
    Ok(Json(repo.optional_user(user)?.about()))
}

async fn export(
    State(AppState { repo }): State<AppState>,
) -> Result<Json<Data>> {
    Ok(Json(repo.export().await?))
}

pub fn routes(assets: &Path) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .route("/export", get(export))
        .nest("/comment", comment::routes())
        .nest("/comments", comments::routes())
        .nest("/invitation", invitation::routes())
        .nest("/login", login::routes())
        .nest("/object", object::routes())
        .nest("/objects", objects::routes())
        .nest("/post", post::routes())
        .nest("/posts", posts::routes())
        .nest("/signup", sign_up::routes())
        .nest("/tag", tag::routes())
        .nest("/tags", tags::routes())
        .nest("/user", user::routes())
        .nest("/users", users::routes())
        .nest_service("/assets", ServeDir::new(assets))
}
