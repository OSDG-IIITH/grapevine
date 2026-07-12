use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::{auth::session::AuthUser, error::AppError, models::{audit, course}, state::AppState};

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let info = sqlx::query!(
        r#"SELECT c.code as course_code, o.season::text as "season!", o.year
           FROM offerings o
           JOIN courses c ON c.id = o.course_id
           WHERE o.id = $1 AND o.deleted_at IS NULL"#,
        id
    )
    .fetch_optional(&s.pool)
    .await?;

    course::delete_offering(&s.pool, &id, &user.id).await?;

    if let Some(info) = info {
        let prev = serde_json::json!({ "course_code": info.course_code, "season": info.season, "year": info.year });
        audit::logaction(&s.pool, &user.id, "DELETE_OFFERING", "offering", &id, Some(prev)).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_faculty(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<course::PatchOffering>,
) -> Result<Json<course::OfferingDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let old_faculty = sqlx::query!(
        "SELECT f.slug FROM offering_faculty ofac JOIN faculty f ON f.id = ofac.faculty_id WHERE ofac.offering_id = $1 ORDER BY f.slug",
        id
    )
    .fetch_all(&s.pool)
    .await?;

    let result = course::update_offering_faculty(&s.pool, &id, &body.faculty_ids).await?;

    let prev = serde_json::json!({ "faculty": old_faculty.iter().map(|r| &r.slug).collect::<Vec<_>>() });
    audit::logaction(&s.pool, &user.id, "UPDATE_OFFERING_FACULTY", "offering", &id, Some(prev)).await?;

    Ok(Json(result))
}
