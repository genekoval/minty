use super::{
    session::{CookieJarSession, SessionCookie},
    session::{OptionalUser, User},
    text::Text,
    Accept, AppState, Result, Router,
};

use crate::server::content::{self, Content, PostSearchResult};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use minty::{
    http::query::SetProfileName, text, Login, PostQuery, ProfileName, Source,
    Url, Uuid,
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
    if let Some(session) = jar.get_session() {
        repo.sessions().delete(session).await?;
    }

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
    if let Some(session) = jar.get_session() {
        repo.sessions().delete(session).await?;
    }

    Ok((StatusCode::NO_CONTENT, jar.remove_session_cookie()))
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
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar)> {
    let mut status = StatusCode::UNAUTHORIZED;

    if let Some(session) = jar.get_session() {
        if let Some(session) = repo.sessions().get(session).await? {
            repo.with_user(session.user()).edit_self().delete().await?;
            status = StatusCode::NO_CONTENT;
        }
    };

    Ok((status, jar.remove_session_cookie()))
}

async fn get_authenticated_user(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    accept: Accept,
) -> Result<Content<content::User>> {
    let info = repo.with_user(user.clone()).get_self()?;

    let posts = if accept.is_api() {
        None
    } else {
        let query = PostQuery {
            poster: Some(user.id),
            ..Default::default()
        };

        let result = repo
            .optional_user(Some(user.clone()))?
            .posts()
            .find(query.clone())
            .await?;

        Some(PostSearchResult::new(query, result))
    };

    let data = content::User { user: info, posts };

    Ok(Content {
        accept,
        data,
        user: Some(user),
    })
}

async fn get_user(
    State(AppState { repo }): State<AppState>,
    Path(user): Path<Uuid>,
    OptionalUser(requester): OptionalUser,
    accept: Accept,
) -> Result<Content<content::User>> {
    let user = repo
        .optional_user(requester.clone())?
        .other(user)
        .await?
        .get()?;

    let posts = if accept.is_api() {
        None
    } else {
        let query = PostQuery {
            poster: Some(user.id),
            ..Default::default()
        };

        let result = repo
            .optional_user(requester.clone())?
            .posts()
            .find(query.clone())
            .await?;

        Some(PostSearchResult::new(query, result))
    };

    let data = content::User { user, posts };

    Ok(Content {
        accept,
        data,
        user: requester,
    })
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
