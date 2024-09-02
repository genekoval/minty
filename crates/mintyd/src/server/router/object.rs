use super::{
    session::{OptionalUser, User},
    Accept, AppState, Content, Result, Router,
};

use crate::server::content::Object;

use axum::{
    extract::{Path, Request, State},
    http::header::{CONTENT_LENGTH, CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use axum_extra::body::AsyncReadBody;
use minty::{ObjectPreview, ObjectSummary, Uuid};
use sync_wrapper::SyncStream;
use tokio_util::io::StreamReader;

async fn add_object(
    State(AppState { repo }): State<AppState>,
    User(user): User,
    request: Request,
) -> Result<Json<ObjectPreview>> {
    let stream = request.into_body().into_data_stream();
    let objects = repo.with_user(user).objects();

    let object = objects.upload(SyncStream::new(stream)).await?;

    Ok(Json(object))
}

async fn get_object(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
    accept: Accept,
    OptionalUser(user): OptionalUser,
) -> Result<Content<Object>> {
    let object = repo.optional_user(user.clone())?.object(id).get().await?;

    Ok(Content {
        accept,
        user,
        data: Object(object),
    })
}

async fn get_object_data(
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    Path((id, _name)): Path<(Uuid, String)>,
) -> Result<Response> {
    let (ObjectSummary { media_type, size }, stream) =
        repo.optional_user(user)?.object(id).get_data().await?;

    let headers = [
        (CONTENT_LENGTH, size.to_string()),
        (CONTENT_TYPE, media_type),
    ];

    let reader = StreamReader::new(stream);
    let body = AsyncReadBody::new(reader);

    Ok((headers, body).into_response())
}

pub fn routes() -> Router {
    Router::new()
        .route("/", post(add_object))
        .route("/:id", get(get_object))
        .route("/:id/:name", get(get_object_data))
}
