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
    pub reviews_count: i64,
    pub research: f64,
    pub availability: f64,
    pub mentorship: f64,
    pub support: f64,
    pub workload: f64,
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

fn fdesc(a: f64, b: f64) -> std::cmp::Ordering {
    match (a == 0.0, b == 0.0) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _ => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
    }
}

fn fasc(a: f64, b: f64) -> std::cmp::Ordering {
    match (a == 0.0, b == 0.0) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _ => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
    }
}

pub async fn list(pool: &PgPool, q: Option<&str>, sort: Option<&str>) -> Result<Vec<FacultyLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name,
                  COALESCE(AVG(r.overall::float8), 0.0)::float8 as "overall!: f64",
                  COALESCE(AVG(r.research::float8), 0.0)::float8 as "research!: f64",
                  COALESCE(AVG(r.availability::float8), 0.0)::float8 as "availability!: f64",
                  COALESCE(AVG(r.mentorship::float8), 0.0)::float8 as "mentorship!: f64",
                  COALESCE(AVG(r.support::float8), 0.0)::float8 as "support!: f64",
                  COALESCE(AVG(r.workload::float8), 0.0)::float8 as "workload!: f64",
                  COUNT(r.id)::int8 as "reviews_count!: i64"
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
        reviews_count: r.reviews_count, research: r.research,
        availability: r.availability, mentorship: r.mentorship,
        support: r.support, workload: r.workload,
    }).collect();

    let rcmp = |a: i64, b: i64, desc: bool| -> std::cmp::Ordering {
        match (a == 0, b == 0) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => if desc { b.cmp(&a) } else { a.cmp(&b) },
        }
    };

    match sort {
        Some("name_asc")           => results.sort_by(|a, b| a.name.cmp(&b.name)),
        Some("name_desc")          => results.sort_by(|a, b| b.name.cmp(&a.name)),
        Some("reviews_desc")       => results.sort_by(|a, b| rcmp(a.reviews_count, b.reviews_count, true)),
        Some("reviews_asc")        => results.sort_by(|a, b| rcmp(a.reviews_count, b.reviews_count, false)),
        Some("rating_desc")        => results.sort_by(|a, b| fdesc(a.overall, b.overall)),
        Some("rating_asc")         => results.sort_by(|a, b| fasc(a.overall, b.overall)),
        Some("mentorship_desc")    => results.sort_by(|a, b| fdesc(a.mentorship, b.mentorship)),
        Some("mentorship_asc")     => results.sort_by(|a, b| fasc(a.mentorship, b.mentorship)),
        Some("availability_desc")  => results.sort_by(|a, b| fdesc(a.availability, b.availability)),
        Some("availability_asc")   => results.sort_by(|a, b| fasc(a.availability, b.availability)),
        Some("support_desc")       => results.sort_by(|a, b| fdesc(a.support, b.support)),
        Some("support_asc")        => results.sort_by(|a, b| fasc(a.support, b.support)),
        Some("research_desc")      => results.sort_by(|a, b| fdesc(a.research, b.research)),
        Some("research_asc")       => results.sort_by(|a, b| fasc(a.research, b.research)),
        Some("workload_desc")      => results.sort_by(|a, b| fdesc(a.workload, b.workload)),
        Some("workload_asc")       => results.sort_by(|a, b| fasc(a.workload, b.workload)),
        _ => results.sort_by(|a, b| fdesc(a.overall, b.overall)),
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
                  COALESCE(AVG(r.overall::float8), 0.0)::float8 as "overall!: f64"
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
