use axum::{routing::get, Router};
use axum::http::StatusCode;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/offerings", get(|| async { StatusCode::OK }))
}
