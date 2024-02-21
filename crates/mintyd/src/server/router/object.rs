use super::{AppState, Result, Router};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json,
};
use minty::{Object, ObjectPreview, Uuid};

async fn add_object(
    State(AppState { repo }): State<AppState>,
) -> Result<Json<ObjectPreview>> {
    todo!()
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
