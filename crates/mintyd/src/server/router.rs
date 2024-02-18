use super::AppState;

use axum::{extract::State, routing::get, Json, Router};
use minty_core::About;

async fn about(State(AppState { repo }): State<AppState>) -> Json<About> {
    Json(*repo.about())
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(about))
}
