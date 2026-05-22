use axum::{routing::get, Router};
use crate::{handlers::reviews, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/offerings/:id/reviews", get(reviews::offering_reviews).post(reviews::create_course_review))
}
