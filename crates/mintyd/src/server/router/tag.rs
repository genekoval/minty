use super::{
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
use minty::{
    http::query::SetProfileName, text, PostQuery, ProfileName, Source, Url,
    Uuid,
};

async fn add_source(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path(tag): Path<Uuid>,
    Json(url): Json<Url>,
) -> Result<Json<Source>> {
    Ok(Json(
        repo.with_user(user)
            .tag(tag)
            .await?
            .add_source(&url)
            .await?,
    ))
}

async fn add_tag(
    State(AppState { repo }): State<AppState>,
    Path(tag): Path<text::Name>,
    User(user): User,
) -> Result<String> {
    Ok(repo.with_user(user).tags().add(tag).await?.id().to_string())
}

async fn delete_alias(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path((tag, name)): Path<(Uuid, String)>,
) -> Result<Json<ProfileName>> {
    Ok(Json(
        repo.with_user(user)
            .tag(tag)
            .await?
            .delete_alias(&name)
            .await?,
    ))
}

async fn delete_source(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path((tag, source)): Path<(Uuid, i64)>,
) -> Result<StatusCode> {
    let status = if repo
        .with_user(user)
        .tag(tag)
        .await?
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
    Path(tag): Path<Uuid>,
    Json(sources): Json<Vec<String>>,
) -> Result<StatusCode> {
    repo.with_user(user)
        .tag(tag)
        .await?
        .delete_sources(&sources)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_tag(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path(tag): Path<Uuid>,
) -> Result<StatusCode> {
    repo.with_user(user).tag(tag).await?.delete().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_tag(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    Path(tag): Path<Uuid>,
    accept: Accept,
) -> Result<Content<content::Tag>> {
    let tag = repo.optional_user(user.clone())?.tag(tag).await?.get()?;

    let posts = if accept.is_api() {
        None
    } else {
        let query = PostQuery {
            tags: vec![tag.id],
            ..Default::default()
        };

        let result = repo
            .optional_user(user)?
            .posts()
            .find(query.clone())
            .await?;

        Some(PostSearchResult { query, result })
    };

    let data = content::Tag { tag, posts };

    Ok(Content { accept, data })
}

async fn set_description(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path(tag): Path<Uuid>,
    Text(description): Text<text::Description>,
) -> Result<String> {
    Ok(repo
        .with_user(user)
        .tag(tag)
        .await?
        .set_description(description)
        .await?)
}

async fn set_name(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Path((tag, name)): Path<(Uuid, text::Name)>,
    Query(SetProfileName { main }): Query<SetProfileName>,
) -> Result<Json<ProfileName>> {
    let main = main.unwrap_or(false);
    let tag = repo.with_user(user).tag(tag).await?;

    let result = if main {
        tag.set_name(name).await
    } else {
        tag.add_alias(name).await
    }?;

    Ok(Json(result))
}

pub fn routes() -> Router {
    Router::new()
        .route("/:tag", get(get_tag).post(add_tag).delete(delete_tag))
        .route("/:tag/name/:name", put(set_name).delete(delete_alias))
        .route("/:tag/description", put(set_description))
        .route("/:tag/source", post(add_source).delete(delete_sources))
        .route("/:tag/source/:source", delete(delete_source))
}
