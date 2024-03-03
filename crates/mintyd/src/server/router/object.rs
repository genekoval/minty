use super::{AppState, Result, Router};

use axum::{
    extract::{Path, Request, State},
    routing::{get, post},
    Json,
};
use minty::{Object, ObjectPreview, Uuid};

async fn add_object(
    State(AppState { repo }): State<AppState>,
    request: Request,
) -> Result<Json<ObjectPreview>> {
    let stream = request.into_body().into_data_stream();
    let object = repo.add_object(stream).await?;

    Ok(Json(object))
}

async fn get_object(
    State(AppState { repo }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>> {
    Ok(Json(repo.get_object(id).await?))
}

pub fn routes() -> Router {
    Router::new()
        .route("/", post(add_object))
        .route("/:id", get(get_object))
}
