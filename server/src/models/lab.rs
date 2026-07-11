use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct CreateLab {
    pub name: String,
    pub short: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchLab {
    pub name: String,
    pub short: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct LabLean {
    pub id: String,
    pub short: String,
    pub name: String,
    pub description: String,
    pub facultycount: i64,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
pub struct AdvisorAxes {
    pub research: f64,
    pub availability: f64,
    pub mentorship: f64,
    pub support: f64,
    pub workload: f64,
}

#[derive(Debug, Serialize)]
pub struct FacultyRef {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub title: String,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
pub struct LabDetail {
    pub id: String,
    pub short: String,
    pub name: String,
    pub description: String,
    pub facultycount: i64,
    pub overall: f64,
    pub axes: AdvisorAxes,
    pub faculty: Vec<FacultyRef>,
}

pub async fn list(pool: &PgPool, q: Option<&str>) -> Result<Vec<LabLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT l.id, l.shortname, l.name, l.description,
                  COUNT(DISTINCT f.id)::int8 as "facultycount!: i64",
                  COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM labs l
           LEFT JOIN faculty_labs fl ON fl.lab_id = l.id
           LEFT JOIN faculty f ON f.id = fl.faculty_id
           LEFT JOIN advisor_reviews r ON r.faculty_id = f.id
           WHERE l.deleted_at IS NULL AND ($1::text IS NULL OR l.shortname ILIKE $1 OR l.name ILIKE $1)
           GROUP BY l.id, l.shortname, l.name, l.description
           ORDER BY l.name"#,
        pattern
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| LabLean {
        id: r.id, short: r.shortname, name: r.name,
        description: r.description.unwrap_or_default(),
        facultycount: r.facultycount, overall: r.overall,
    }).collect())
}

pub async fn update_lab(pool: &PgPool, current_short: &str, patch: &PatchLab) -> Result<LabDetail, AppError> {
    if patch.short != current_short {
        let exists: Option<String> = sqlx::query_scalar!("SELECT id FROM labs WHERE shortname = $1", patch.short)
            .fetch_optional(pool)
            .await?;
        if exists.is_some() { return Err(AppError::BadRequest("shortname already in use".into())); }
    }
    let rows = sqlx::query!(
        "UPDATE labs SET name = $1, shortname = $2, description = $3 WHERE shortname = $4",
        patch.name, patch.short, patch.description, current_short
    )
    .execute(pool)
    .await?
    .rows_affected();
    if rows == 0 { return Err(AppError::NotFound); }
    get_by_shortname(pool, &patch.short).await
}

pub async fn get_by_shortname(pool: &PgPool, shortname: &str) -> Result<LabDetail, AppError> {
    let row = sqlx::query!(
        "SELECT id, shortname, name, description FROM labs WHERE shortname = $1 AND deleted_at IS NULL",
        shortname
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let axes_row = sqlx::query!(
        r#"SELECT
                COALESCE(AVG(r.research::float), 0.0)::float8 as "research!: f64",
                COALESCE(AVG(r.availability::float), 0.0)::float8 as "availability!: f64",
                COALESCE(AVG(r.mentorship::float), 0.0)::float8 as "mentorship!: f64",
                COALESCE(AVG(r.support::float), 0.0)::float8 as "support!: f64",
                COALESCE(AVG(r.workload::float), 0.0)::float8 as "workload!: f64",
                COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM advisor_reviews r
           JOIN faculty f ON f.id = r.faculty_id
           JOIN faculty_labs fl ON fl.faculty_id = f.id
           WHERE fl.lab_id = $1"#,
        row.id
    )
    .fetch_one(pool)
    .await?;

    let faculty_rows = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name,
                  COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM faculty f
           JOIN faculty_labs fl ON fl.faculty_id = f.id
           LEFT JOIN advisor_reviews r ON r.faculty_id = f.id
           WHERE fl.lab_id = $1 AND f.deleted_at IS NULL
           GROUP BY f.id, f.slug, f.name
           ORDER BY f.name"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let faculty: Vec<FacultyRef> = faculty_rows.into_iter().map(|r| FacultyRef {
        id: r.id, slug: r.slug, name: r.name, title: String::new(), overall: r.overall,
    }).collect();

    let facultycount = faculty.len() as i64;

    Ok(LabDetail {
        id: row.id, short: row.shortname, name: row.name,
        description: row.description.unwrap_or_default(),
        facultycount, overall: axes_row.overall,
        axes: AdvisorAxes {
            research: axes_row.research, availability: axes_row.availability,
            mentorship: axes_row.mentorship, support: axes_row.support, workload: axes_row.workload,
        },
        faculty,
    })
}

pub async fn soft_delete(pool: &PgPool, shortname: &str) -> Result<(), AppError> {
    let n = sqlx::query!("UPDATE labs SET deleted_at = NOW() WHERE shortname = $1 AND deleted_at IS NULL", shortname)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

pub async fn restore(pool: &PgPool, shortname: &str) -> Result<(), AppError> {
    let n = sqlx::query!("UPDATE labs SET deleted_at = NULL WHERE shortname = $1 AND deleted_at IS NOT NULL", shortname)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

#[derive(Debug, Serialize)]
pub struct DeletedLab {
    pub shortname: String,
    pub name: String,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_deleted(pool: &PgPool) -> Result<Vec<DeletedLab>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT shortname, name, deleted_at as "deleted_at!: chrono::DateTime<chrono::Utc>"
           FROM labs WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC"#
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| DeletedLab { shortname: r.shortname, name: r.name, deleted_at: r.deleted_at }).collect())
}

pub async fn create_lab(pool: &PgPool, body: &CreateLab) -> Result<LabDetail, AppError> {
    let id = ulid::Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO labs (id, name, shortname) VALUES ($1, $2, $3)",
        id, body.name, body.short
    )
    .execute(pool).await?;
    get_by_shortname(pool, &body.short).await
}
