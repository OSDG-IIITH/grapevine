use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use ulid::Ulid;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct AuditLog {
    pub id: String,
    pub admin_id: String,
    pub admin_name: String,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub target_name: Option<String>,
    pub target_course_code: Option<String>,
    pub previous_state: Option<Value>,
    pub created_at: DateTime<Utc>,
}

pub async fn logaction(
    pool: &PgPool,
    admin_id: &str,
    action: &str,
    target_type: &str,
    target_id: &str,
    previous_state: Option<Value>,
) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO audit_logs (id, admin_id, action, target_type, target_id, previous_state) VALUES ($1, $2, $3, $4, $5, $6)",
        id, admin_id, action, target_type, target_id, previous_state
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct AuditPage {
    pub logs: Vec<AuditLog>,
    pub total: i64,
    pub has_more: bool,
}

pub async fn list(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    admin_id: Option<&str>,
    action: Option<&str>,
    target_type: Option<&str>,
) -> Result<AuditPage, AppError> {
    let rows = sqlx::query!(
        r#"SELECT a.id, a.admin_id, u.display_name as admin_name, a.action, a.target_type, a.target_id,
                  a.previous_state as "previous_state?: Value",
                  a.created_at as "created_at!: DateTime<Utc>",
                  COALESCE(tc.name, tu.display_name, tf.name, tl.name, oc.name) as "target_name?: String",
                  oc.code as "target_course_code?: String",
                  COUNT(*) OVER() as "total!: i64"
           FROM audit_logs a
           JOIN users u ON u.id = a.admin_id
           LEFT JOIN courses tc ON a.target_type = 'course' AND tc.code = a.target_id
           LEFT JOIN users tu ON a.target_type = 'user' AND tu.id = a.target_id
           LEFT JOIN faculty tf ON a.target_type = 'faculty' AND tf.slug = a.target_id
           LEFT JOIN labs tl ON a.target_type = 'lab' AND tl.shortname = a.target_id
           LEFT JOIN offerings o ON a.target_type = 'offering' AND o.id = a.target_id
           LEFT JOIN courses oc ON o.course_id = oc.id
           WHERE ($3::text IS NULL OR a.admin_id = $3)
             AND ($4::text IS NULL OR a.action = $4)
             AND ($5::text IS NULL OR a.target_type = $5)
           ORDER BY a.created_at DESC
           LIMIT $1 OFFSET $2"#,
        limit,
        offset,
        admin_id,
        action,
        target_type
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map(|r| r.total).unwrap_or(0);
    let logs: Vec<AuditLog> = rows.into_iter().map(|r| AuditLog {
        id: r.id,
        admin_id: r.admin_id,
        admin_name: r.admin_name,
        action: r.action,
        target_type: r.target_type,
        target_id: r.target_id,
        target_name: r.target_name,
        target_course_code: r.target_course_code,
        previous_state: r.previous_state,
        created_at: r.created_at,
    }).collect();
    let has_more = offset + limit < total;
    Ok(AuditPage { logs, total, has_more })
}
