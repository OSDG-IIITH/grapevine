use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::AppError;
use super::offering::Season;

#[derive(Debug, Deserialize)]
pub struct CreateFaculty {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchFaculty {
    pub name: String,
    pub slug: String,
    pub bio: String,
    pub lab_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FacultyLean {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub labs: Vec<LabRef>,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
pub struct LabRef {
    pub id: String,
    pub short: String,
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
    pub code: String,
    pub season: Season,
    pub year: i16,
    pub course: CourseRef,
}

#[derive(Debug, Serialize)]
pub struct FacultyDetail {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub bio: String,
    pub title: String,
    pub overall: f64,
    pub labs: Vec<LabRef>,
    pub offerings: Vec<OfferingWithCourse>,
}

pub async fn list(pool: &PgPool, q: Option<&str>, sort: Option<&str>) -> Result<Vec<FacultyLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name,
                  COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM faculty f
           LEFT JOIN advisor_reviews r ON r.faculty_id = f.id AND r.deleted_at IS NULL
           WHERE f.deleted_at IS NULL AND ($1::text IS NULL OR f.name ILIKE $1)
           GROUP BY f.id, f.slug, f.name
           ORDER BY f.name"#,
        pattern
    )
    .fetch_all(pool)
    .await?;

    let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let lab_rows = sqlx::query!(
        r#"SELECT fl.faculty_id, l.id, l.shortname, l.name
           FROM faculty_labs fl
           JOIN labs l ON l.id = fl.lab_id AND l.deleted_at IS NULL
           WHERE fl.faculty_id = ANY($1)"#,
        &ids as &[String]
    )
    .fetch_all(pool)
    .await?;

    let mut labs_map: HashMap<String, Vec<LabRef>> = HashMap::new();
    for lr in lab_rows {
        labs_map.entry(lr.faculty_id).or_default().push(LabRef { id: lr.id, short: lr.shortname, name: lr.name });
    }

    let mut results: Vec<FacultyLean> = rows.into_iter().map(|r| FacultyLean {
        labs: labs_map.remove(&r.id).unwrap_or_default(),
        id: r.id, slug: r.slug, name: r.name, overall: r.overall,
    }).collect();

    match sort {
        Some("rating_desc") => results.sort_by(|a, b| {
            match (a.overall == 0.0, b.overall == 0.0) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b.overall.partial_cmp(&a.overall).unwrap_or(std::cmp::Ordering::Equal),
            }
        }),
        Some("rating_asc") => results.sort_by(|a, b| {
            match (a.overall == 0.0, b.overall == 0.0) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => a.overall.partial_cmp(&b.overall).unwrap_or(std::cmp::Ordering::Equal),
            }
        }),
        _ => {}
    }

    Ok(results)
}

pub async fn update_faculty(pool: &PgPool, current_slug: &str, patch: &PatchFaculty) -> Result<FacultyDetail, AppError> {
    if patch.slug != current_slug {
        let exists: Option<String> = sqlx::query_scalar!("SELECT id FROM faculty WHERE slug = $1", patch.slug)
            .fetch_optional(pool)
            .await?;
        if exists.is_some() { return Err(AppError::BadRequest("slug already in use".into())); }
    }
    let id: String = sqlx::query_scalar!("SELECT id FROM faculty WHERE slug = $1", current_slug)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    sqlx::query!(
        "UPDATE faculty SET name = $1, slug = $2, bio = $3 WHERE id = $4",
        patch.name, patch.slug, patch.bio, id
    )
    .execute(pool)
    .await?;
    sqlx::query!("DELETE FROM faculty_labs WHERE faculty_id = $1", id)
        .execute(pool)
        .await?;
    for lab_id in &patch.lab_ids {
        sqlx::query!("INSERT INTO faculty_labs (faculty_id, lab_id) VALUES ($1, $2)", id, lab_id)
            .execute(pool)
            .await?;
    }
    get_by_slug(pool, &patch.slug).await
}

pub async fn id_by_slug(pool: &PgPool, slug: &str) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT id FROM faculty WHERE slug = $1 AND deleted_at IS NULL", slug)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn soft_delete(pool: &PgPool, slug: &str) -> Result<(), AppError> {
    let n = sqlx::query!("UPDATE faculty SET deleted_at = NOW() WHERE slug = $1 AND deleted_at IS NULL", slug)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

pub async fn restore(pool: &PgPool, slug: &str) -> Result<(), AppError> {
    let n = sqlx::query!("UPDATE faculty SET deleted_at = NULL WHERE slug = $1 AND deleted_at IS NOT NULL", slug)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

#[derive(Debug, Serialize)]
pub struct DeletedFaculty {
    pub slug: String,
    pub name: String,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_deleted(pool: &PgPool) -> Result<Vec<DeletedFaculty>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT slug, name, deleted_at as "deleted_at!: chrono::DateTime<chrono::Utc>"
           FROM faculty WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC"#
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| DeletedFaculty { slug: r.slug, name: r.name, deleted_at: r.deleted_at }).collect())
}

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<FacultyDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name, f.bio,
                  COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM faculty f
           LEFT JOIN advisor_reviews r ON r.faculty_id = f.id AND r.deleted_at IS NULL
           WHERE f.slug = $1 AND f.deleted_at IS NULL
           GROUP BY f.id, f.slug, f.name, f.bio"#,
        slug
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let lab_rows = sqlx::query!(
        r#"SELECT l.id, l.shortname, l.name
           FROM faculty_labs fl
           JOIN labs l ON l.id = fl.lab_id AND l.deleted_at IS NULL
           WHERE fl.faculty_id = $1"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let labs: Vec<LabRef> = lab_rows.into_iter().map(|r| LabRef { id: r.id, short: r.shortname, name: r.name }).collect();

    let offering_rows = sqlx::query!(
        r#"SELECT o.id as oid, o.season as "season: Season", o.year,
                  c.id as cid, c.code, c.name as cname
           FROM offerings o
           JOIN offering_faculty ofac ON ofac.offering_id = o.id
           JOIN courses c ON c.id = o.course_id AND c.deleted_at IS NULL
           WHERE ofac.faculty_id = $1 AND o.deleted_at IS NULL
           ORDER BY o.year DESC, o.season"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let offerings = offering_rows.into_iter().map(|r| OfferingWithCourse {
        id: r.oid,
        code: format!("{}{}", match &r.season { Season::M => "M", Season::S => "S" }, r.year),
        season: r.season,
        year: r.year,
        course: CourseRef { id: r.cid, code: r.code, name: r.cname },
    }).collect();

    Ok(FacultyDetail {
        id: row.id, slug: row.slug, name: row.name,
        bio: row.bio.unwrap_or_default(),
        title: String::new(),
        overall: row.overall,
        labs, offerings,
    })
}

pub async fn create_faculty(pool: &PgPool, body: &CreateFaculty) -> Result<FacultyDetail, AppError> {
    let id = ulid::Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO faculty (id, name, slug) VALUES ($1, $2, $3)",
        id, body.name, body.slug
    )
    .execute(pool).await?;
    get_by_slug(pool, &body.slug).await
}
