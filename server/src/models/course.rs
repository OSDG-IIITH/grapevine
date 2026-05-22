use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::AppError;
use super::offering::Season;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "course_type", rename_all = "lowercase")]
pub enum CourseType {
    Core,
    Open,
    Breadth,
    Stream,
    Bouquet,
    Hs,
    Sci,
    Math,
}

#[derive(Debug, Serialize)]
pub struct CourseLean {
    pub id: String,
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: CourseType,
}

#[derive(Debug, Serialize)]
pub struct FacultyRef {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct OfferingDetail {
    pub id: String,
    pub season: Season,
    pub year: i16,
    pub faculty: Vec<FacultyRef>,
}

#[derive(Debug, Serialize)]
pub struct CourseDetail {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub kind: CourseType,
    pub offerings: Vec<OfferingDetail>,
}

pub async fn list(pool: &PgPool, q: Option<&str>) -> Result<Vec<CourseLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT id, code, name, type as "kind: CourseType"
           FROM courses
           WHERE ($1::text IS NULL OR code ILIKE $1 OR name ILIKE $1)
           ORDER BY name"#,
        pattern
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| CourseLean { id: r.id, code: r.code, name: r.name, kind: r.kind }).collect())
}

pub async fn id_by_code(pool: &PgPool, code: &str) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT id FROM courses WHERE code = $1", code)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn get_by_code(pool: &PgPool, code: &str) -> Result<CourseDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, code, name, description, type as "kind: CourseType" FROM courses WHERE code = $1"#,
        code
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let offering_rows = sqlx::query!(
        r#"SELECT o.id as oid, o.season as "season: Season", o.year,
                  f.id as "fid?", f.slug as "fslug?", f.name as "fname?"
           FROM offerings o
           LEFT JOIN offering_faculty ofac ON ofac.offering_id = o.id
           LEFT JOIN faculty f ON f.id = ofac.faculty_id
           WHERE o.course_id = $1
           ORDER BY o.year DESC, o.season"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let mut offerings: Vec<OfferingDetail> = vec![];
    for r in offering_rows {
        if let Some(o) = offerings.iter_mut().find(|o| o.id == r.oid) {
            if let (Some(fid), Some(fslug), Some(fname)) = (r.fid, r.fslug, r.fname) {
                o.faculty.push(FacultyRef { id: fid, slug: fslug, name: fname });
            }
        } else {
            let mut faculty = vec![];
            if let (Some(fid), Some(fslug), Some(fname)) = (r.fid, r.fslug, r.fname) {
                faculty.push(FacultyRef { id: fid, slug: fslug, name: fname });
            }
            offerings.push(OfferingDetail { id: r.oid, season: r.season, year: r.year, faculty });
        }
    }

    Ok(CourseDetail { id: row.id, code: row.code, name: row.name, description: row.description, kind: row.kind, offerings })
}
