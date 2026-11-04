use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::{auth::session::AuthUser, error::AppError, models::draft::{self, Draft, SaveDraft}, state::AppState};

pub async fn list(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Draft>>, AppError> {
    Ok(Json(draft::list(&s.pool, &user.id).await?))
}

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<SaveDraft>,
) -> Result<(StatusCode, Json<Draft>), AppError> {
    Ok((StatusCode::CREATED, Json(draft::create(&s.pool, &user.id, body).await?)))
}

pub async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<SaveDraft>,
) -> Result<Json<Draft>, AppError> {
    Ok(Json(draft::update(&s.pool, &id, &user.id, body).await?))
}

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    draft::delete(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
