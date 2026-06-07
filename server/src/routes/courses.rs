use axum::{routing::{get, post}, Router};
use crate::{handlers::courses, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/courses", get(courses::list))
        .route("/courses/:code", get(courses::get).patch(courses::update))
        .route("/courses/:code/reviews", get(courses::reviews))
        .route("/courses/:code/offerings", post(courses::create_offering))
}
