use super::{
    session::{OptionalUser, User},
    text::Text,
    timestamp::Timestamp,
    AppState, Result, Router,
};

use crate::server::{
    content::{Content, ObjectViewer, Post, PostEdit, SavedChanges},
    query, Accept,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post, put},
    Json,
};
use minty::{text, PostParts, Uuid, Visibility};

async fn add_objects(
    State(AppState { repo }): State<AppState>,
    Path((id, destination)): Path<(Uuid, Uuid)>,
    User(user): User,
    Json(objects): Json<Vec<Uuid>>,
) -> Result<Timestamp> {
    Ok(repo
        .with_user(user)
        .post(id)
        .await?
        .edit()?
        .add_objects(&objects, Some(destination))
        .await?
        .into())
}

async fn append_objects(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    User(user): User,
    Json(objects): Json<Vec<Uuid>>,
) -> Result<Timestamp> {
    Ok(repo
        .with_user(user)
        .post(id)
        .await?
        .edit()?
        .add_objects(&objects, None)
        .await?
        .into())
}

async fn add_related_post(
    State(AppState { repo }): State<AppState>,
    Path((id, related)): Path<(Uuid, Uuid)>,
    User(user): User,
) -> Result<StatusCode> {
    repo.with_user(user)
        .post(id)
        .await?
        .edit()?
        .add_related_post(related)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_tag(
    State(AppState { repo }): State<AppState>,
    Path((id, tag)): Path<(Uuid, Uuid)>,
    User(user): User,
) -> Result<StatusCode> {
    repo.with_user(user)
        .post(id)
        .await?
        .edit()?
        .add_tag(tag)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_draft(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
) -> Result<Redirect> {
    if let Some(user) = user {
        let parts = PostParts {
            visibility: Some(Visibility::Draft),
            ..Default::default()
        };

        let id = repo.with_user(user).posts().add(&parts).await?.id();
        let path = format!("/post/{id}");

        Ok(Redirect::to(&path))
    } else {
        Ok(Redirect::to("/signin"))
    }
}

async fn create_post(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    Json(parts): Json<PostParts>,
) -> Result<String> {
    Ok(repo
        .with_user(user)
        .posts()
        .add(&parts)
        .await?
        .id()
        .to_string())
}

async fn delete_objects(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    User(user): User,
    Json(objects): Json<Vec<Uuid>>,
) -> Result<Timestamp> {
    Ok(repo
        .with_user(user)
        .post(id)
        .await?
        .edit()?
        .delete_objects(&objects)
        .await?
        .into())
}

async fn delete_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    User(user): User,
) -> Result<StatusCode> {
    repo.with_user(user)
        .post(id)
        .await?
        .edit()?
        .delete()
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_related_post(
    State(AppState { repo }): State<AppState>,
    Path((id, related)): Path<(Uuid, Uuid)>,
    User(user): User,
) -> Result<StatusCode> {
    repo.with_user(user)
        .post(id)
        .await?
        .edit()?
        .delete_related_post(related)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_tag(
    State(AppState { repo }): State<AppState>,
    Path((id, tag)): Path<(Uuid, Uuid)>,
    User(user): User,
) -> Result<StatusCode> {
    let status = if repo
        .with_user(user)
        .post(id)
        .await?
        .edit()?
        .delete_tag(tag)
        .await?
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    };

    Ok(status)
}

async fn edit_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    OptionalUser(user): OptionalUser,
    accept: Accept,
) -> Result<Content<PostEdit>> {
    let post = repo
        .optional_user(user.clone())?
        .post(id)
        .await?
        .get()
        .await?;

    Ok(Content {
        accept,
        user,
        data: PostEdit(post),
    })
}

async fn get_objects(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<query::ObjectViewer>,
    accept: Accept,
    OptionalUser(user): OptionalUser,
) -> Result<Content<ObjectViewer>> {
    let post = repo.optional_user(user.clone())?.post(id).await?;
    let preview = post.preview()?;
    let objects = post.get_objects()?;

    Ok(Content {
        accept,
        user,
        data: ObjectViewer::new(preview, objects, query.index),
    })
}

async fn get_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    accept: Accept,
    OptionalUser(user): OptionalUser,
) -> Result<Content<Post>> {
    let repo = repo.optional_user(user.clone())?.post(id).await?;
    let post = repo.get().await?;

    let comments = if accept.is_api() {
        Vec::new()
    } else {
        repo.get_comments().await?
    };

    let data = Post { post, comments };

    Ok(Content { accept, user, data })
}

async fn publish_post(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    User(user): User,
) -> Result<StatusCode> {
    repo.with_user(user)
        .post(id)
        .await?
        .edit()?
        .publish()
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_description(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    accept: Accept,
    User(user): User,
    Text(description): Text<text::Description>,
) -> Result<Content<SavedChanges>> {
    let modified = repo
        .with_user(user.clone())
        .post(id)
        .await?
        .edit()?
        .set_description(description)
        .await?
        .date_modified;

    Ok(Content {
        accept,
        user: Some(user),
        data: SavedChanges {
            title: None,
            modified,
        },
    })
}

async fn set_title(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    accept: Accept,
    User(user): User,
    Text(title): Text<text::PostTitle>,
) -> Result<Content<SavedChanges>> {
    let modified = repo
        .with_user(user.clone())
        .post(id)
        .await?
        .edit()?
        .set_title(title.clone())
        .await?
        .date_modified;

    Ok(Content {
        accept,
        user: Some(user),
        data: SavedChanges {
            title: Some(title.into()),
            modified,
        },
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(create_draft).post(create_post))
        .route("/:id", get(get_post).put(publish_post).delete(delete_post))
        .route("/:id/description", put(set_description))
        .route("/:id/edit", get(edit_post))
        .route(
            "/:id/objects",
            get(get_objects).post(append_objects).delete(delete_objects),
        )
        .route("/:id/objects/:destination", post(add_objects))
        .route(
            "/:id/related/:related",
            put(add_related_post).delete(delete_related_post),
        )
        .route("/:id/tag/:tag", put(add_tag).delete(delete_tag))
        .route("/:id/title", put(set_title))
}
