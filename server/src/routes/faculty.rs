use axum::{routing::get, Router};
use crate::{handlers::{faculty, reviews}, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/faculty", get(faculty::list).post(faculty::create))
        .route("/faculty/:slug", get(faculty::get).patch(faculty::update).delete(faculty::delete))
        .route("/faculty/:slug/reviews", get(faculty::reviews).post(reviews::create_advisor_review))
}
