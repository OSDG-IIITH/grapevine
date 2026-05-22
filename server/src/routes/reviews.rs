use axum::{routing::get, Router};
use crate::{handlers::reviews, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reviews/courses", get(reviews::list).post(reviews::create))
        .route("/reviews/advisors", get(reviews::list).post(reviews::create))
}
