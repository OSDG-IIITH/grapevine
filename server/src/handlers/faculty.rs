use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{auth::session::{AuthUser, MaybeAuth}, error::AppError, models::{audit, faculty, review}, state::AppState};

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<faculty::CreateFaculty>,
) -> Result<(StatusCode, Json<faculty::FacultyDetail>), AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    let result = faculty::create_faculty(&s.pool, &body).await?;
    audit::logaction(&s.pool, &user.id, "CREATE_FACULTY", "faculty", &result.slug, None).await?;
    Ok((StatusCode::CREATED, Json(result)))
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

    let prev_row = sqlx::query!(
        "SELECT id, name, slug, bio FROM faculty WHERE slug = $1 AND deleted_at IS NULL",
        slug
    )
    .fetch_optional(&s.pool)
    .await?;

    let result = faculty::update_faculty(&s.pool, &slug, &body).await?;

    if let Some(p) = prev_row {
        let prev = serde_json::json!({ "name": p.name, "slug": p.slug, "bio": p.bio });
        audit::logaction(&s.pool, &user.id, "UPDATE_FACULTY", "faculty", &p.id, Some(prev)).await?;
    }

    Ok(Json(result))
}

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let row = sqlx::query!(
        "SELECT name FROM faculty WHERE slug = $1 AND deleted_at IS NULL",
        slug
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    faculty::soft_delete(&s.pool, &slug).await?;

    let prev = serde_json::json!({ "name": row.name, "slug": slug });
    audit::logaction(&s.pool, &user.id, "DELETE_FACULTY", "faculty", &slug, Some(prev)).await?;

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

pub async fn legacy_reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<review::LegacyAdvisorReview>>, AppError> {
    let fac_id = faculty::id_by_slug(&s.pool, &slug).await?;
    Ok(Json(review::legacy_advisor_reviews_by_faculty(&s.pool, &fac_id, &user_id).await?))
}
