use serde::Serialize;
use sqlx::PgPool;
use crate::error::AppError;
use super::offering::Season;

#[derive(Debug, Serialize)]
pub struct FacultyLean {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub lab_id: Option<String>,
    pub lab_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LabRef {
    pub id: String,
    pub shortname: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CourseRef {
    pub id: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct OfferingWithCourse {
    pub id: String,
    pub season: Season,
    pub year: i16,
    pub course: CourseRef,
}

#[derive(Debug, Serialize)]
pub struct FacultyDetail {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub bio: Option<String>,
    pub lab: Option<LabRef>,
    pub offerings: Vec<OfferingWithCourse>,
}

pub async fn list(pool: &PgPool, q: Option<&str>) -> Result<Vec<FacultyLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name, f.lab_id, l.name as "lab_name?"
           FROM faculty f
           LEFT JOIN labs l ON l.id = f.lab_id
           WHERE ($1::text IS NULL OR f.name ILIKE $1)
           ORDER BY f.name"#,
        pattern
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| FacultyLean {
        id: r.id, slug: r.slug, name: r.name, lab_id: r.lab_id, lab_name: r.lab_name,
    }).collect())
}

pub async fn id_by_slug(pool: &PgPool, slug: &str) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT id FROM faculty WHERE slug = $1", slug)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<FacultyDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name, f.bio,
                  l.id as "lid?", l.shortname as "lshortname?", l.name as "lname?"
           FROM faculty f
           LEFT JOIN labs l ON l.id = f.lab_id
           WHERE f.slug = $1"#,
        slug
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let lab = match (row.lid, row.lshortname, row.lname) {
        (Some(id), Some(shortname), Some(name)) => Some(LabRef { id, shortname, name }),
        _ => None,
    };

    let offering_rows = sqlx::query!(
        r#"SELECT o.id as oid, o.season as "season: Season", o.year,
                  c.id as cid, c.code, c.name as cname
           FROM offerings o
           JOIN offering_faculty ofac ON ofac.offering_id = o.id
           JOIN courses c ON c.id = o.course_id
           WHERE ofac.faculty_id = $1
           ORDER BY o.year DESC, o.season"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let offerings = offering_rows.into_iter().map(|r| OfferingWithCourse {
        id: r.oid,
        season: r.season,
        year: r.year,
        course: CourseRef { id: r.cid, code: r.code, name: r.cname },
    }).collect();

    Ok(FacultyDetail { id: row.id, slug: row.slug, name: row.name, bio: row.bio, lab, offerings })
}
