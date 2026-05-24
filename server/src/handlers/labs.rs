use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use crate::{error::AppError, models::lab, state::AppState};

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

pub async fn get(
    State(s): State<AppState>,
    Path(shortname): Path<String>,
) -> Result<Json<lab::LabDetail>, AppError> {
    Ok(Json(lab::get_by_shortname(&s.pool, &shortname).await?))
}
