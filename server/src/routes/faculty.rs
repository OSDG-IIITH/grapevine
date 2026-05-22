use axum::{routing::get, Router};
use crate::{handlers::faculty, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/faculty", get(faculty::list))
        .route("/faculty/:id", get(faculty::get))
}
