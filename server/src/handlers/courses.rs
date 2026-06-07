use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use crate::{auth::session::{AuthUser, MaybeAuth}, error::AppError, models::{course, review}, state::AppState};

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    instructor: Option<String>,
    sort: Option<String>,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<course::CourseLean>>, AppError> {
    Ok(Json(course::list(&s.pool, q.q.as_deref(), q.instructor.as_deref(), q.sort.as_deref()).await?))
}

pub async fn get(
    State(s): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<course::CourseDetail>, AppError> {
    Ok(Json(course::get_by_code(&s.pool, &code).await?))
}

pub async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<course::PatchCourse>,
) -> Result<Json<course::CourseDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(course::update_course(&s.pool, &code, &body).await?))
}

pub async fn create_offering(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<course::CreateOffering>,
) -> Result<Json<course::OfferingDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(course::create_offering(&s.pool, &code, &body).await?))
}

pub async fn reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(code): Path<String>,
) -> Result<Json<Vec<review::CourseReview>>, AppError> {
    let id = course::id_by_code(&s.pool, &code).await?;
    Ok(Json(review::course_reviews_by_course(&s.pool, &id, &user_id).await?))
}
