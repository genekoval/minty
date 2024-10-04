use super::{
    session::{OptionalUser, User},
    AppState, Result, Router,
};

use axum::{
    extract::{Path, Request, State},
    routing::{get, post},
    Json,
};
use axum_extra::TypedHeader;
use fstore::http::{ProxyMethod, ProxyResponse, Range};
use minty::{Object, ObjectPreview, Uuid};
use minty_core::{Bytes, Stream};
use std::io;
use sync_wrapper::SyncStream;

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
    OptionalUser(user): OptionalUser,
) -> Result<Json<Object>> {
    let object = repo.optional_user(user.clone())?.object(id).get().await?;
    Ok(Json(object))
}

async fn proxy(
    method: ProxyMethod,
    State(AppState { repo }): State<AppState>,
    OptionalUser(user): OptionalUser,
    id: Uuid,
    range: Option<TypedHeader<Range>>,
) -> Result<ProxyResponse<impl Stream<Item = io::Result<Bytes>>>> {
    let range = range.map(|TypedHeader(range)| range);

    Ok(repo
        .optional_user(user)?
        .object(id)
        .proxy(method, range)
        .await?)
}

async fn proxy_get(
    state: State<AppState>,
    user: OptionalUser,
    Path((id, _name)): Path<(Uuid, String)>,
    range: Option<TypedHeader<Range>>,
) -> Result<ProxyResponse<impl Stream<Item = io::Result<Bytes>>>> {
    proxy(ProxyMethod::Get, state, user, id, range).await
}

async fn proxy_head(
    state: State<AppState>,
    user: OptionalUser,
    Path((id, _name)): Path<(Uuid, String)>,
    range: Option<TypedHeader<Range>>,
) -> Result<ProxyResponse<impl Stream<Item = io::Result<Bytes>>>> {
    proxy(ProxyMethod::Head, state, user, id, range).await
}

pub fn routes() -> Router {
    Router::new()
        .route("/", post(add_object))
        .route("/:id", get(get_object))
        .route("/:id/:name", get(proxy_get).head(proxy_head))
}
