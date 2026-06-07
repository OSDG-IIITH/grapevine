use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::{auth::session::AuthUser, error::AppError, models::course, state::AppState};

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    course::delete_offering(&s.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_faculty(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<course::PatchOffering>,
) -> Result<Json<course::OfferingDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(course::update_offering_faculty(&s.pool, &id, &body.faculty_ids).await?))
}
