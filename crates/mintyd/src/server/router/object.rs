use super::{AppState, Result, Router};

use axum::{
    extract::{Path, Request, State},
    http::header::{CONTENT_LENGTH, CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use axum_extra::body::AsyncReadBody;
use minty::{Object, ObjectPreview, ObjectSummary, Uuid};
use tokio_util::io::StreamReader;

async fn add_object(
    State(AppState { repo }): State<AppState>,
    request: Request,
) -> Result<Json<ObjectPreview>> {
    let stream = request.into_body().into_data_stream();
    let object = repo.add_object_stream(stream).await?;

    Ok(Json(object))
}

async fn get_object(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>> {
    Ok(Json(repo.get_object(id).await?))
}

async fn get_object_data(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let (ObjectSummary { media_type, size }, stream) =
        repo.get_object_data(id).await?;

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
        .route("/:id/data", get(get_object_data))
}
