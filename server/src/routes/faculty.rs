use axum::{routing::get, Router};
use crate::{handlers::{faculty, reviews}, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/faculty", get(faculty::list))
        .route("/faculty/:slug", get(faculty::get).patch(faculty::update))
        .route("/faculty/:slug/reviews", get(faculty::reviews).post(reviews::create_advisor_review))
}
