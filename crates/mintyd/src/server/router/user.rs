use super::{
    session::{CookieSession, SessionCookie, COOKIE},
    session::{OptionalUser, User},
    text::Text,
    AppState, Result, Router,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use cookie::Cookie;
use minty::{
    http::query::SetProfileName, text, Login, ProfileName, Source, Url, Uuid,
};

async fn add_source(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Json(url): Json<Url>,
) -> Result<Json<Source>> {
    Ok(Json(
        repo.with_user(user).edit_self().add_source(&url).await?,
    ))
}

async fn create_session(
    State(AppState { repo }): State<AppState>,
    jar: CookieJar,
    Json(login): Json<Login>,
) -> Result<(CookieJar, String)> {
    let session = repo.authenticate(&login).await?;

    Ok((jar.add(session.cookie()), session.user_id.to_string()))
}

async fn delete_alias(
    State(AppState { repo }): State<AppState>,
    Path(name): Path<String>,
    User(user): User,
) -> Result<Json<ProfileName>> {
    Ok(Json(
        repo.with_user(user).edit_self().delete_alias(&name).await?,
    ))
}

async fn delete_session(
    State(AppState { repo }): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar)> {
    if let Some(cookie) = jar.get(COOKIE) {
        if let Some(session) = cookie.session() {
            repo.sessions().delete(session).await?;
            return Ok((
                StatusCode::NO_CONTENT,
                jar.remove(Cookie::build(COOKIE).path("/")),
            ));
        }
    }

    Err(minty_core::Error::Unauthenticated(None).into())
}

async fn delete_source(
    State(AppState { repo }): State<AppState>,
    Path(source): Path<i64>,
    User(user): User,
) -> Result<StatusCode> {
    let status = if repo
        .with_user(user)
        .edit_self()
        .delete_source(source)
        .await?
    {
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
    repo.with_user(user)
        .edit_self()
        .delete_sources(&sources)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    State(AppState { repo }): State<AppState>,
    User(user): User,
) -> Result<StatusCode> {
    repo.with_user(user).edit_self().delete().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_authenticated_user(
    State(AppState { repo }): State<AppState>,
    User(user): User,
) -> Result<Json<minty::User>> {
    Ok(Json(repo.with_user(user).get_self()?))
}

async fn get_user(
    State(AppState { repo }): State<AppState>,
    Path(user): Path<Uuid>,
    OptionalUser(requester): OptionalUser,
) -> Result<Json<minty::User>> {
    Ok(Json(
        repo.optional_user(requester)?.other(user).await?.get()?,
    ))
}

async fn grant_admin(
    State(AppState { repo }): State<AppState>,
    User(admin): User,
    Path(user): Path<Uuid>,
) -> Result<StatusCode> {
    repo.admin(admin)?.user(user).await?.set_admin(true).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn revoke_admin(
    State(AppState { repo }): State<AppState>,
    User(admin): User,
    Path(user): Path<Uuid>,
) -> Result<StatusCode> {
    repo.admin(admin)?
        .user(user)
        .await?
        .set_admin(false)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_description(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Text(description): Text<text::Description>,
) -> Result<String> {
    Ok(repo
        .with_user(user)
        .edit_self()
        .set_description(description)
        .await?)
}

async fn set_email(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Text(email): Text<text::Email>,
) -> Result<StatusCode> {
    repo.with_user(user).edit_self().set_email(email).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_name(
    State(AppState { repo }): State<AppState>,
    Path(name): Path<text::Name>,
    Query(SetProfileName { main }): Query<SetProfileName>,
    User(user): User,
) -> Result<Json<ProfileName>> {
    let main = main.unwrap_or(false);
    let user = repo.with_user(user).edit_self();

    let names = if main {
        user.set_name(name).await
    } else {
        user.add_alias(name).await
    }?;

    Ok(Json(names))
}

async fn set_password(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Text(password): Text<text::Password>,
) -> Result<StatusCode> {
    repo.with_user(user)
        .edit_self()
        .set_password(password)
        .await?;
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
        .route("/:user/admin", put(grant_admin).delete(revoke_admin))
}
