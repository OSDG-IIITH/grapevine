use axum::{routing::{delete, patch, post}, Router};
use crate::{handlers::reviews, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reviews/course/:id", patch(reviews::edit_course_review).delete(reviews::delete_course_review))
        .route("/reviews/course/:id/vote", post(reviews::vote_course_review).delete(reviews::unvote_course_review))
        .route("/reviews/course/:id/flag", post(reviews::flag_course_review))
        .route("/reviews/advisor/:id", patch(reviews::edit_advisor_review).delete(reviews::delete_advisor_review))
        .route("/reviews/advisor/:id/vote", post(reviews::vote_advisor_review).delete(reviews::unvote_advisor_review))
        .route("/reviews/advisor/:id/flag", post(reviews::flag_advisor_review))
        .route("/reviews/legacy/course/:id", delete(reviews::delete_legacy_course_review))
        .route("/reviews/legacy/course/:id/vote", post(reviews::vote_legacy_course_review).delete(reviews::unvote_legacy_course_review))
        .route("/reviews/legacy/advisor/:id", delete(reviews::delete_legacy_advisor_review))
        .route("/reviews/legacy/advisor/:id/vote", post(reviews::vote_legacy_advisor_review).delete(reviews::unvote_legacy_advisor_review))
        .route("/reviews/external/course", post(reviews::create_external_course_review))
        .route("/reviews/external/course/:id", patch(reviews::edit_external_course_review).delete(reviews::delete_external_course_review))
        .route("/reviews/external/course/:id/vote", post(reviews::vote_external_course_review).delete(reviews::unvote_external_course_review))
        .route("/reviews/external/advisor", post(reviews::create_external_advisor_review))
        .route("/reviews/external/advisor/:id", patch(reviews::edit_external_advisor_review).delete(reviews::delete_external_advisor_review))
        .route("/reviews/external/advisor/:id/vote", post(reviews::vote_external_advisor_review).delete(reviews::unvote_external_advisor_review))
}
