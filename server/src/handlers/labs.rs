use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{auth::session::AuthUser, error::AppError, models::{audit, lab}, state::AppState};

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<lab::CreateLab>,
) -> Result<(StatusCode, Json<lab::LabDetail>), AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    let result = lab::create_lab(&s.pool, &body).await?;
    audit::logaction(&s.pool, &user.id, "CREATE_LAB", "lab", &result.short, None).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<lab::LabLean>>, AppError> {
    Ok(Json(lab::list(&s.pool, q.q.as_deref()).await?))
}

pub async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(shortname): Path<String>,
    Json(body): Json<lab::PatchLab>,
) -> Result<Json<lab::LabDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let prev_row = sqlx::query!(
        "SELECT id, name, shortname, description FROM labs WHERE shortname = $1 AND deleted_at IS NULL",
        shortname
    )
    .fetch_optional(&s.pool)
    .await?;

    let result = lab::update_lab(&s.pool, &shortname, &body).await?;

    if let Some(p) = prev_row {
        let prev = serde_json::json!({ "name": p.name, "short": p.shortname, "description": p.description });
        audit::logaction(&s.pool, &user.id, "UPDATE_LAB", "lab", &p.id, Some(prev)).await?;
    }

    Ok(Json(result))
}

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(shortname): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let row = sqlx::query!(
        "SELECT name FROM labs WHERE shortname = $1 AND deleted_at IS NULL",
        shortname
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    lab::soft_delete(&s.pool, &shortname).await?;

    let prev = serde_json::json!({ "name": row.name, "short": shortname });
    audit::logaction(&s.pool, &user.id, "DELETE_LAB", "lab", &shortname, Some(prev)).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get(
    State(s): State<AppState>,
    Path(shortname): Path<String>,
) -> Result<Json<lab::LabDetail>, AppError> {
    Ok(Json(lab::get_by_shortname(&s.pool, &shortname).await?))
}
