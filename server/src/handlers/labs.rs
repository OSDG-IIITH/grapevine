use axum::{
    extract::{Path, State},
    Json,
};
use crate::{error::AppError, models::lab, state::AppState};

pub async fn list(
    State(s): State<AppState>,
) -> Result<Json<Vec<lab::LabLean>>, AppError> {
    Ok(Json(lab::list(&s.pool).await?))
}

pub async fn get(
    State(s): State<AppState>,
    Path(shortname): Path<String>,
) -> Result<Json<lab::LabDetail>, AppError> {
    Ok(Json(lab::get_by_shortname(&s.pool, &shortname).await?))
}
