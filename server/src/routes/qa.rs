use axum::{routing::{get, patch, post}, Router};
use crate::{handlers::qa, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/questions", get(qa::list).post(qa::create_question))
        .route("/questions/:id", patch(qa::edit_question).delete(qa::delete_question))
        .route("/questions/:id/vote", post(qa::vote_question).delete(qa::unvote_question))
        .route("/questions/:id/flag", post(qa::flag_question))
        .route("/questions/:id/answers", post(qa::create_answer))
        .route("/answers/:id", patch(qa::edit_answer).delete(qa::delete_answer))
        .route("/answers/:id/vote", post(qa::vote_answer).delete(qa::unvote_answer))
        .route("/answers/:id/flag", post(qa::flag_answer))
}
