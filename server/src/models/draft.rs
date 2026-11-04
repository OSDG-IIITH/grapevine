use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use crate::error::AppError;

#[derive(Debug, Serialize, FromRow)]
pub struct Draft {
    pub id: String,
    pub course_id: Option<String>,
    pub course_code: Option<String>,
    pub course_name: Option<String>,
    pub faculty_id: Option<String>,
    pub faculty_slug: Option<String>,
    pub faculty_name: Option<String>,
    pub data: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SaveDraft {
    pub course_id: Option<String>,
    pub faculty_id: Option<String>,
    pub data: serde_json::Value,
}

const SELECT: &str = r#"SELECT d.id, d.course_id, c.code AS course_code, c.name AS course_name,
                               d.faculty_id, f.slug AS faculty_slug, f.name AS faculty_name,
                               d.data, d.updated_at
                        FROM review_drafts d
                        LEFT JOIN courses c ON c.id = d.course_id AND c.deleted_at IS NULL
                        LEFT JOIN faculty f ON f.id = d.faculty_id AND f.deleted_at IS NULL
                        WHERE d.user_id = $1 AND (c.id IS NOT NULL OR f.id IS NOT NULL)"#;

/// reject drafts without exactly one target or with an oversized payload
fn check(body: &SaveDraft) -> Result<(), AppError> {
    if body.course_id.is_some() == body.faculty_id.is_some() {
        return Err(AppError::BadRequest("draft needs a course or a faculty".into()));
    }
    if !body.data.is_object() || body.data.to_string().len() > 32_000 {
        return Err(AppError::BadRequest("invalid draft".into()));
    }
    Ok(())
}

pub async fn list(pool: &PgPool, user_id: &str) -> Result<Vec<Draft>, AppError> {
    Ok(sqlx::query_as::<_, Draft>(&format!("{SELECT} ORDER BY d.updated_at DESC"))
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

async fn get(pool: &PgPool, id: &str, user_id: &str) -> Result<Draft, AppError> {
    sqlx::query_as::<_, Draft>(&format!("{SELECT} AND d.id = $2"))
        .bind(user_id)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create(pool: &PgPool, user_id: &str, body: SaveDraft) -> Result<Draft, AppError> {
    check(&body)?;
    let id: String = sqlx::query_scalar(
        "INSERT INTO review_drafts (user_id, course_id, faculty_id, data) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(user_id)
    .bind(&body.course_id)
    .bind(&body.faculty_id)
    .bind(&body.data)
    .fetch_one(pool)
    .await?;
    get(pool, &id, user_id).await
}

pub async fn update(pool: &PgPool, id: &str, user_id: &str, body: SaveDraft) -> Result<Draft, AppError> {
    check(&body)?;
    let res = sqlx::query(
        "UPDATE review_drafts SET data = $3, updated_at = now() WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(&body.data)
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 { return Err(AppError::NotFound); }
    get(pool, id, user_id).await
}

pub async fn delete(pool: &PgPool, id: &str, user_id: &str) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM review_drafts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 { return Err(AppError::NotFound); }
    Ok(())
}
