use axum::{routing::get, Router};
use crate::{handlers::admin, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin", get(admin::dashboard))
}
