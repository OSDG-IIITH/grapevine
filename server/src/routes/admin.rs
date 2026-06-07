use axum::{routing::{get, post}, Router};
use crate::{handlers::admin, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/export", get(admin::export))
        .route("/admin/flags", get(admin::flags))
        .route("/admin/flags/:id/dismiss", post(admin::dismiss_flag))
        .route("/admin/flags/:id/delete-review", post(admin::delete_review))
}
