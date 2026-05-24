use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use crate::{auth::session::MaybeAuth, error::AppError, models::{faculty, review}, state::AppState};

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

pub async fn reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<review::AdvisorReview>>, AppError> {
    let fac_id = faculty::id_by_slug(&s.pool, &slug).await?;
    Ok(Json(review::advisor_reviews_by_faculty(&s.pool, &fac_id, &user_id).await?))
}
