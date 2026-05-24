use serde::Serialize;
use sqlx::PgPool;
use crate::error::AppError;
use super::offering::Season;

#[derive(Debug, Serialize)]
pub struct FacultyLean {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub lab: Option<String>,
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
    pub lab: Option<LabRef>,
    pub offerings: Vec<OfferingWithCourse>,
}

pub async fn list(pool: &PgPool, q: Option<&str>, sort: Option<&str>) -> Result<Vec<FacultyLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name, l.shortname as "lab?",
                  COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM faculty f
           LEFT JOIN labs l ON l.id = f.lab_id
           LEFT JOIN advisor_reviews r ON r.faculty_id = f.id
           WHERE ($1::text IS NULL OR f.name ILIKE $1)
           GROUP BY f.id, f.slug, f.name, l.shortname
           ORDER BY f.name"#,
        pattern
    )
    .fetch_all(pool)
    .await?;

    let mut results: Vec<FacultyLean> = rows.into_iter().map(|r| FacultyLean {
        id: r.id, slug: r.slug, name: r.name, lab: r.lab, overall: r.overall,
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

pub async fn id_by_slug(pool: &PgPool, slug: &str) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT id FROM faculty WHERE slug = $1", slug)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<FacultyDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT f.id, f.slug, f.name, f.bio,
                  l.id as "lid?", l.shortname as "lshort?", l.name as "lname?",
                  COALESCE(AVG((r.research + r.availability + r.mentorship + r.support + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM faculty f
           LEFT JOIN labs l ON l.id = f.lab_id
           LEFT JOIN advisor_reviews r ON r.faculty_id = f.id
           WHERE f.slug = $1
           GROUP BY f.id, f.slug, f.name, f.bio, l.id, l.shortname, l.name"#,
        slug
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let lab = match (row.lid, row.lshort, row.lname) {
        (Some(id), Some(short), Some(name)) => Some(LabRef { id, short, name }),
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
        lab, offerings,
    })
}
