use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{auth::session::AuthUser, error::AppError, models::lab, state::AppState};

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<lab::CreateLab>,
) -> Result<(StatusCode, Json<lab::LabDetail>), AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok((StatusCode::CREATED, Json(lab::create_lab(&s.pool, &body).await?)))
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
    Ok(Json(lab::update_lab(&s.pool, &shortname, &body).await?))
}

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(shortname): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    lab::soft_delete(&s.pool, &shortname).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get(
    State(s): State<AppState>,
    Path(shortname): Path<String>,
) -> Result<Json<lab::LabDetail>, AppError> {
    Ok(Json(lab::get_by_shortname(&s.pool, &shortname).await?))
}
