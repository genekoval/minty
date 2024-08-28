mod assets;
mod comment;
mod comments;
mod invitation;
mod object;
mod objects;
mod post;
mod posts;
mod session;
mod sign_in;
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
use minty::{model::export::Data, PostQuery};
use minty_core::About;

pub type Router = axum::Router<AppState>;

async fn home(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    accept: Accept,
) -> Result<Content<Home>> {
    let query = PostQuery::default();

    let result = repo
        .optional_user(user.clone())?
        .posts()
        .find(query.clone())
        .await?;

    Ok(Content {
        accept,
        user,
        data: Home::new(query, result),
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

pub fn routes() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .route("/export", get(export))
        .nest("/assets", assets::routes())
        .nest("/comment", comment::routes())
        .nest("/comments", comments::routes())
        .nest("/invitation", invitation::routes())
        .nest("/object", object::routes())
        .nest("/objects", objects::routes())
        .nest("/post", post::routes())
        .nest("/posts", posts::routes())
        .nest("/signin", sign_in::routes())
        .nest("/signup", sign_up::routes())
        .nest("/tag", tag::routes())
        .nest("/tags", tags::routes())
        .nest("/user", user::routes())
        .nest("/users", users::routes())
}
