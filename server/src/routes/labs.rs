use axum::{routing::get, Router};
use crate::{handlers::labs, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/labs", get(labs::list))
        .route("/labs/:id", get(labs::get))
}
