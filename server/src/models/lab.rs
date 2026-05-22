use serde::Serialize;
use sqlx::PgPool;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct LabLean {
    pub id: String,
    pub shortname: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FacultyRef {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct LabDetail {
    pub id: String,
    pub shortname: String,
    pub name: String,
    pub description: Option<String>,
    pub faculty: Vec<FacultyRef>,
}

pub async fn list(pool: &PgPool) -> Result<Vec<LabLean>, AppError> {
    let rows = sqlx::query!("SELECT id, shortname, name, description FROM labs ORDER BY name")
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| LabLean { id: r.id, shortname: r.shortname, name: r.name, description: r.description }).collect())
}

pub async fn get_by_shortname(pool: &PgPool, shortname: &str) -> Result<LabDetail, AppError> {
    let row = sqlx::query!(
        "SELECT id, shortname, name, description FROM labs WHERE shortname = $1",
        shortname
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let faculty = sqlx::query!(
        "SELECT id, slug, name FROM faculty WHERE lab_id = $1 ORDER BY name",
        row.id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| FacultyRef { id: r.id, slug: r.slug, name: r.name })
    .collect();

    Ok(LabDetail { id: row.id, shortname: row.shortname, name: row.name, description: row.description, faculty })
}
