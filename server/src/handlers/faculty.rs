use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{auth::session::{AuthUser, MaybeAuth}, error::AppError, models::{faculty, review}, state::AppState};

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<faculty::CreateFaculty>,
) -> Result<(StatusCode, Json<faculty::FacultyDetail>), AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok((StatusCode::CREATED, Json(faculty::create_faculty(&s.pool, &body).await?)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    sort: Option<String>,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<faculty::FacultyLean>>, AppError> {
    Ok(Json(faculty::list(&s.pool, q.q.as_deref(), q.sort.as_deref()).await?))
}

pub async fn get(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<faculty::FacultyDetail>, AppError> {
    Ok(Json(faculty::get_by_slug(&s.pool, &slug).await?))
}

pub async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<faculty::PatchFaculty>,
) -> Result<Json<faculty::FacultyDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(faculty::update_faculty(&s.pool, &slug, &body).await?))
}

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    faculty::soft_delete(&s.pool, &slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<review::AdvisorReview>>, AppError> {
    let fac_id = faculty::id_by_slug(&s.pool, &slug).await?;
    Ok(Json(review::advisor_reviews_by_faculty(&s.pool, &fac_id, &user_id).await?))
}
