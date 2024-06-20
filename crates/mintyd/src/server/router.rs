mod comment;
mod comments;
mod invitation;
mod login;
mod object;
mod objects;
mod post;
mod posts;
mod sign_up;
mod tag;
mod tags;
mod text;
mod timestamp;
mod user;
mod users;

use super::{error::Result, extract::OptionalUser, AppState};

use axum::{extract::State, routing::get, Json};
use minty::model::export::Data;
use minty_core::About;

pub type Router = axum::Router<AppState>;

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

pub fn routes() -> Router {
    Router::new()
        .route("/", get(about))
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
}
