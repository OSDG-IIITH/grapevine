use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::AppError;
use super::offering::Season;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
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
    pub overall: f64,
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
    pub code: String,
    pub season: Season,
    pub year: i16,
    pub faculty: Vec<FacultyRef>,
}

#[derive(Debug, Serialize)]
pub struct CourseDetail {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: CourseType,
    pub overall: f64,
    pub offerings: Vec<OfferingDetail>,
}

pub async fn list(pool: &PgPool, q: Option<&str>, instructor: Option<&str>, sort: Option<&str>) -> Result<Vec<CourseLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT c.id, c.code, c.name, c.type as "kind: CourseType",
                  COALESCE(AVG((r.difficulty + r.teaching + r.grading + r.content + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM courses c
           LEFT JOIN offerings o ON o.course_id = c.id
           LEFT JOIN course_reviews r ON r.offering_id = o.id
           WHERE ($1::text IS NULL OR c.code ILIKE $1 OR c.name ILIKE $1)
             AND ($2::text IS NULL OR EXISTS (
               SELECT 1 FROM offerings oi
               JOIN offering_faculty ofac ON ofac.offering_id = oi.id
               JOIN faculty f ON f.id = ofac.faculty_id
               WHERE oi.course_id = c.id AND f.slug = $2
             ))
           GROUP BY c.id, c.code, c.name, c.type
           ORDER BY c.name"#,
        pattern,
        instructor
    )
    .fetch_all(pool)
    .await?;

    let mut results: Vec<CourseLean> = rows.into_iter().map(|r| CourseLean {
        id: r.id, code: r.code, name: r.name, kind: r.kind, overall: r.overall,
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

pub async fn id_by_code(pool: &PgPool, code: &str) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT id FROM courses WHERE code = $1", code)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn get_by_code(pool: &PgPool, code: &str) -> Result<CourseDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT c.id, c.code, c.name, c.description,
                  c.type as "kind: CourseType",
                  COALESCE(AVG((r.difficulty + r.teaching + r.grading + r.content + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM courses c
           LEFT JOIN offerings o ON o.course_id = c.id
           LEFT JOIN course_reviews r ON r.offering_id = o.id
           WHERE c.code = $1
           GROUP BY c.id, c.code, c.name, c.description, c.type"#,
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
        let code = format!("{}{}", match &r.season { Season::M => "M", Season::S => "S" }, r.year);
        if let Some(o) = offerings.iter_mut().find(|o| o.id == r.oid) {
            if let (Some(fid), Some(fslug), Some(fname)) = (r.fid, r.fslug, r.fname) {
                o.faculty.push(FacultyRef { id: fid, slug: fslug, name: fname });
            }
        } else {
            let mut faculty = vec![];
            if let (Some(fid), Some(fslug), Some(fname)) = (r.fid, r.fslug, r.fname) {
                faculty.push(FacultyRef { id: fid, slug: fslug, name: fname });
            }
            offerings.push(OfferingDetail {
                id: r.oid, code, season: r.season, year: r.year, faculty,
            });
        }
    }

    Ok(CourseDetail {
        id: row.id, code: row.code, name: row.name,
        description: row.description.unwrap_or_default(),
        kind: row.kind, overall: row.overall, offerings,
    })
}
