use axum::{Json, extract::State, http::StatusCode};

use crate::{
    auth::session::AuthUser,
    error::AppError,
    models::report::{self, CreateReport},
    state::AppState,
};

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateReport>,
) -> Result<StatusCode, AppError> {
    report::create(&s.pool, &user.id, body).await?;
    Ok(StatusCode::CREATED)
}
