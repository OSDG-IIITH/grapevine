use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::{auth::session::AuthUser, error::AppError, state::AppState};

#[derive(Debug, Serialize)]
pub struct ReporterRef {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub id: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub review_type: &'static str,
    pub review_id: String,
    pub review_body: String,
    pub reporter: ReporterRef,
}

pub async fn flags(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<FlagResponse>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let course_flags = sqlx::query!(
        r#"SELECT f.id, f.reason,
                  f.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                  f.review_id,
                  LEFT(cr.body, 200) as "review_body?",
                  u.id as reporter_id, u.display_name as reporter_name
           FROM course_review_flags f
           JOIN course_reviews cr ON cr.id = f.review_id
           JOIN users u ON u.id = f.user_id
           ORDER BY f.created_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    let advisor_flags = sqlx::query!(
        r#"SELECT f.id, f.reason,
                  f.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                  f.review_id,
                  LEFT(ar.body, 200) as "review_body?",
                  u.id as reporter_id, u.display_name as reporter_name
           FROM advisor_review_flags f
           JOIN advisor_reviews ar ON ar.id = f.review_id
           JOIN users u ON u.id = f.user_id
           ORDER BY f.created_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    let mut result: Vec<FlagResponse> = course_flags.into_iter().map(|r| FlagResponse {
        id: r.id,
        reason: r.reason,
        created_at: r.created_at,
        review_type: "course",
        review_id: r.review_id,
        review_body: r.review_body.unwrap_or_default(),
        reporter: ReporterRef { id: r.reporter_id, display_name: r.reporter_name },
    }).collect();

    result.extend(advisor_flags.into_iter().map(|r| FlagResponse {
        id: r.id,
        reason: r.reason,
        created_at: r.created_at,
        review_type: "advisor",
        review_id: r.review_id,
        review_body: r.review_body.unwrap_or_default(),
        reporter: ReporterRef { id: r.reporter_id, display_name: r.reporter_name },
    }));

    result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(result))
}

pub async fn dismiss_flag(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let rows = sqlx::query!("DELETE FROM course_review_flags WHERE id = $1", id)
        .execute(&s.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        sqlx::query!("DELETE FROM advisor_review_flags WHERE id = $1", id)
            .execute(&s.pool)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let course_flag = sqlx::query!(
        "SELECT review_id FROM course_review_flags WHERE id = $1",
        id
    )
    .fetch_optional(&s.pool)
    .await?;

    if let Some(f) = course_flag {
        sqlx::query!("DELETE FROM course_reviews WHERE id = $1", f.review_id)
            .execute(&s.pool)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let advisor_flag = sqlx::query!(
        "SELECT review_id FROM advisor_review_flags WHERE id = $1",
        id
    )
    .fetch_optional(&s.pool)
    .await?;

    if let Some(f) = advisor_flag {
        sqlx::query!("DELETE FROM advisor_reviews WHERE id = $1", f.review_id)
            .execute(&s.pool)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    Err(AppError::NotFound)
}
