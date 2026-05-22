use axum::{routing::get, Router};
use crate::{handlers::courses, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/courses", get(courses::list))
        .route("/courses/:code", get(courses::get))
        .route("/courses/:code/reviews", get(courses::reviews))
}
