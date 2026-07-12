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

pub async fn list(pool: &PgPool) -> Result<Vec<AuditLog>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT a.id, a.admin_id, u.display_name as admin_name, a.action, a.target_type, a.target_id,
                  a.previous_state as "previous_state?: Value",
                  a.created_at as "created_at!: DateTime<Utc>"
           FROM audit_logs a
           JOIN users u ON u.id = a.admin_id
           ORDER BY a.created_at DESC
           LIMIT 500"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| AuditLog {
        id: r.id,
        admin_id: r.admin_id,
        admin_name: r.admin_name,
        action: r.action,
        target_type: r.target_type,
        target_id: r.target_id,
        previous_state: r.previous_state,
        created_at: r.created_at,
    }).collect())
}
