use super::{text::Text, AppState, Result, Router};

use crate::server::extract::{Session, User};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json,
};
use minty::{
    http::query::SetProfileName, text, Login, ProfileName, Source, Url, Uuid,
};

async fn add_source(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Json(url): Json<Url>,
) -> Result<Json<Source>> {
    Ok(Json(repo.user(user).add_source(&url).await?))
}

async fn create_session(
    State(AppState { repo }): State<AppState>,
    Json(login): Json<Login>,
) -> Result<String> {
    Ok(repo.users().authenticate(&login).await?.to_string())
}

async fn delete_alias(
    State(AppState { repo }): State<AppState>,
    Path(name): Path<String>,
    User(user): User,
) -> Result<Json<ProfileName>> {
    Ok(Json(repo.user(user).delete_alias(&name).await?))
}

async fn delete_session(
    State(AppState { repo }): State<AppState>,
    Session(session): Session,
) -> Result<StatusCode> {
    repo.users().delete_session(session).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_source(
    State(AppState { repo }): State<AppState>,
    Path(source): Path<i64>,
    User(user): User,
) -> Result<StatusCode> {
    let status = if repo.user(user).delete_source(source).await? {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    };

    Ok(status)
}

async fn delete_sources(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Json(sources): Json<Vec<String>>,
) -> Result<StatusCode> {
    repo.user(user).delete_sources(&sources).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    State(AppState { repo }): State<AppState>,
    User(user): User,
) -> Result<StatusCode> {
    repo.user(user).delete().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_authenticated_user(
    State(AppState { repo }): State<AppState>,
    User(user): User,
) -> Result<Json<minty::User>> {
    Ok(Json(repo.user(user).get().await?))
}

async fn get_user(
    State(AppState { repo }): State<AppState>,
    Path(user): Path<Uuid>,
) -> Result<Json<minty::User>> {
    Ok(Json(repo.user(user).get().await?))
}

async fn set_description(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Text(description): Text<text::Description>,
) -> Result<String> {
    Ok(repo.user(user).set_description(description).await?)
}

async fn set_email(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Text(email): Text<text::Email>,
) -> Result<StatusCode> {
    repo.user(user).set_email(email).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_name(
    State(AppState { repo }): State<AppState>,
    Path(name): Path<text::Name>,
    Query(SetProfileName { main }): Query<SetProfileName>,
    User(user): User,
) -> Result<Json<ProfileName>> {
    let main = main.unwrap_or(false);

    let result = if main {
        repo.user(user).set_name(name).await
    } else {
        repo.user(user).add_alias(name).await
    }?;

    Ok(Json(result))
}

async fn set_password(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Text(password): Text<text::Password>,
) -> Result<StatusCode> {
    repo.user(user).set_password(password).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_authenticated_user).delete(delete_user))
        .route("/description", put(set_description))
        .route("/email", put(set_email))
        .route("/name/:name", put(set_name).delete(delete_alias))
        .route("/password", put(set_password))
        .route("/session", post(create_session).delete(delete_session))
        .route("/source", post(add_source).delete(delete_sources))
        .route("/source/:source", delete(delete_source))
        .route("/:user", get(get_user))
}
