use axum::{routing::get, Router};
use crate::{handlers::labs, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/labs", get(labs::list).post(labs::create))
        .route("/labs/:shortname", get(labs::get).patch(labs::update).delete(labs::delete))
}
