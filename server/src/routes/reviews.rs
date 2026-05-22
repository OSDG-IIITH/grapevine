use axum::{routing::{patch, post}, Router};
use crate::{handlers::reviews, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reviews/course/:id", patch(reviews::edit_course_review).delete(reviews::delete_course_review))
        .route("/reviews/course/:id/vote", post(reviews::vote_course_review).delete(reviews::unvote_course_review))
        .route("/reviews/course/:id/flag", post(reviews::flag_course_review))
        .route("/reviews/advisor/:id", patch(reviews::edit_advisor_review).delete(reviews::delete_advisor_review))
        .route("/reviews/advisor/:id/vote", post(reviews::vote_advisor_review).delete(reviews::unvote_advisor_review))
        .route("/reviews/advisor/:id/flag", post(reviews::flag_advisor_review))
}
