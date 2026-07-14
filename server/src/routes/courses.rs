use axum::{routing::{get, post}, Router};
use crate::{handlers::courses, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/courses", get(courses::list).post(courses::create))
        .route("/courses/:code", get(courses::get).patch(courses::update).delete(courses::delete))
        .route("/courses/:code/reviews", get(courses::reviews))
        .route("/courses/:code/proposed-reviews", get(courses::proposed_reviews))
        .route("/courses/:code/legacy-reviews", get(courses::legacy_reviews))
        .route("/courses/:code/offerings", post(courses::create_offering))
        .route("/courses/:code/propose-offering", post(courses::propose_offering))
        .route("/courses/:code/propose-review", post(courses::propose_review))
}
