use axum::{routing::{delete, get}, Router};
use crate::{handlers::{offerings, reviews}, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/offerings/:id/reviews", get(reviews::offering_reviews).post(reviews::create_course_review))
        .route("/offerings/:id", delete(offerings::delete).patch(offerings::update_faculty))
}
