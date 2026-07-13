use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportTarget {
    Course,
    Faculty,
    Lab,
    Offering,
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub target_type: ReportTarget,
    pub target_id: String,
    pub reason: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct ReportResponse {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub target_label: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub reporter_id: String,
    pub reporter_name: String,
}

pub async fn create(pool: &PgPool, user_id: &str, body: CreateReport) -> Result<(), AppError> {
    let reason = body.reason.trim();
    if !(3..=1000).contains(&reason.chars().count()) {
        return Err(AppError::BadRequest(
            "reason must be between 3 and 1000 characters".into(),
        ));
    }

    let (exists_sql, insert_sql) = match body.target_type {
        ReportTarget::Course => (
            "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1 AND deleted_at IS NULL)",
            "INSERT INTO reports (user_id, course_id, reason) VALUES ($1, $2, $3)",
        ),
        ReportTarget::Faculty => (
            "SELECT EXISTS(SELECT 1 FROM faculty WHERE id = $1 AND deleted_at IS NULL)",
            "INSERT INTO reports (user_id, faculty_id, reason) VALUES ($1, $2, $3)",
        ),
        ReportTarget::Lab => (
            "SELECT EXISTS(SELECT 1 FROM labs WHERE id = $1 AND deleted_at IS NULL)",
            "INSERT INTO reports (user_id, lab_id, reason) VALUES ($1, $2, $3)",
        ),
        ReportTarget::Offering => (
            "SELECT EXISTS(SELECT 1 FROM offerings o JOIN courses c ON c.id = o.course_id WHERE o.id = $1 AND o.deleted_at IS NULL AND o.approved = true AND c.deleted_at IS NULL)",
            "INSERT INTO reports (user_id, offering_id, reason) VALUES ($1, $2, $3)",
        ),
    };

    let exists = sqlx::query_scalar::<_, bool>(exists_sql)
        .bind(&body.target_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound);
    }

    sqlx::query(insert_sql)
        .bind(user_id)
        .bind(&body.target_id)
        .bind(reason)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list(pool: &PgPool) -> Result<Vec<ReportResponse>, AppError> {
    let reports = sqlx::query_as::<_, ReportResponse>(
        r#"SELECT r.id,
                  CASE
                    WHEN r.course_id IS NOT NULL THEN 'course'
                    WHEN r.faculty_id IS NOT NULL THEN 'faculty'
                    WHEN r.lab_id IS NOT NULL THEN 'lab'
                    ELSE 'offering'
                  END AS target_type,
                  COALESCE(r.course_id, r.faculty_id, r.lab_id, r.offering_id) AS target_id,
                  CASE
                    WHEN r.course_id IS NOT NULL THEN c.code || ' · ' || c.name
                    WHEN r.faculty_id IS NOT NULL THEN f.name
                    WHEN r.lab_id IS NOT NULL THEN l.shortname || ' · ' || l.name
                    ELSE oc.code || ' · ' || oc.name || ' · ' || CASE o.season WHEN 'M' THEN 'Monsoon ' ELSE 'Spring ' END || o.year
                  END AS target_label,
                  r.reason, r.created_at,
                  u.id AS reporter_id, u.display_name AS reporter_name
           FROM reports r
           JOIN users u ON u.id = r.user_id
           LEFT JOIN courses c ON c.id = r.course_id
           LEFT JOIN faculty f ON f.id = r.faculty_id
           LEFT JOIN labs l ON l.id = r.lab_id
           LEFT JOIN offerings o ON o.id = r.offering_id
           LEFT JOIN courses oc ON oc.id = o.course_id
           ORDER BY r.created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(reports)
}

pub async fn dismiss(pool: &PgPool, id: &str, admin_id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let report = sqlx::query_as::<_, ReportResponse>(
        r#"SELECT r.id,
                  CASE
                    WHEN r.course_id IS NOT NULL THEN 'course'
                    WHEN r.faculty_id IS NOT NULL THEN 'faculty'
                    WHEN r.lab_id IS NOT NULL THEN 'lab'
                    ELSE 'offering'
                  END AS target_type,
                  COALESCE(r.course_id, r.faculty_id, r.lab_id, r.offering_id) AS target_id,
                  CASE
                    WHEN r.course_id IS NOT NULL THEN c.code || ' · ' || c.name
                    WHEN r.faculty_id IS NOT NULL THEN f.name
                    WHEN r.lab_id IS NOT NULL THEN l.shortname || ' · ' || l.name
                    ELSE oc.code || ' · ' || oc.name || ' · ' || CASE o.season WHEN 'M' THEN 'Monsoon ' ELSE 'Spring ' END || o.year
                  END AS target_label,
                  r.reason, r.created_at,
                  u.id AS reporter_id, u.display_name AS reporter_name
           FROM reports r
           JOIN users u ON u.id = r.user_id
           LEFT JOIN courses c ON c.id = r.course_id
           LEFT JOIN faculty f ON f.id = r.faculty_id
           LEFT JOIN labs l ON l.id = r.lab_id
           LEFT JOIN offerings o ON o.id = r.offering_id
           LEFT JOIN courses oc ON oc.id = o.course_id
           WHERE r.id = $1
           FOR UPDATE OF r"#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query("DELETE FROM reports WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    let previous = json!({
        "target_type": report.target_type,
        "target_id": report.target_id,
        "target_label": report.target_label,
        "reason": report.reason,
        "reporter_id": report.reporter_id,
        "reporter_name": report.reporter_name,
    });
    sqlx::query(
        "INSERT INTO audit_logs (id, admin_id, action, target_type, target_id, previous_state) VALUES ($1, $2, 'DISMISS_REPORT', 'report', $3, $4)",
    )
    .bind(Ulid::new().to_string())
    .bind(admin_id)
    .bind(id)
    .bind(previous)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
