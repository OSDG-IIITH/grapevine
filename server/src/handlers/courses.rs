use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use crate::{auth::session::MaybeAuth, error::AppError, models::{course, review}, state::AppState};

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<course::CourseLean>>, AppError> {
    Ok(Json(course::list(&s.pool, q.q.as_deref()).await?))
}

pub async fn get(
    State(s): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<course::CourseDetail>, AppError> {
    Ok(Json(course::get_by_code(&s.pool, &code).await?))
}

pub async fn reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(code): Path<String>,
) -> Result<Json<Vec<review::CourseReview>>, AppError> {
    let id = course::id_by_code(&s.pool, &code).await?;
    Ok(Json(review::course_reviews_by_course(&s.pool, &id, &user_id).await?))
}
