use axum::{Router, routing::post};

use crate::{handlers::reports, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/reports", post(reports::create))
}
