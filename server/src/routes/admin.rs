use axum::{routing::{get, post}, Router};
use crate::{handlers::admin, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/export", get(admin::export))
        .route("/admin/flags", get(admin::flags))
        .route("/admin/flags/:id/dismiss", post(admin::dismiss_flag))
        .route("/admin/flags/:id/delete-review", post(admin::delete_review))
        .route("/admin/proposed", get(admin::list_proposed))
        .route("/admin/proposed/:id/approve", post(admin::approve_proposed))
        .route("/admin/proposed/:id/reject", post(admin::reject_proposed))
        .route("/admin/deleted-courses", get(admin::deleted_courses))
        .route("/admin/deleted-courses/:code/restore", post(admin::restore_course))
}
