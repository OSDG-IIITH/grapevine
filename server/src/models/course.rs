use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::AppError;
use super::offering::Season;

#[derive(Debug, Deserialize)]
pub struct CreateOffering {
    pub season: Season,
    pub year: i16,
    pub faculty_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct PatchOffering {
    pub faculty_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PatchCourse {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: CourseType,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
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
pub struct ProposedOfferingLean {
    pub id: String,
    pub season: Season,
    pub year: i16,
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
    pub proposed_offerings: Vec<ProposedOfferingLean>,
}

pub async fn list(pool: &PgPool, q: Option<&str>, instructor: Option<&str>, sort: Option<&str>) -> Result<Vec<CourseLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT c.id, c.code, c.name, c.type as "kind: CourseType",
                  COALESCE(AVG((r.difficulty + r.teaching + r.grading + r.content + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM courses c
           LEFT JOIN offerings o ON o.course_id = c.id AND o.approved = true
           LEFT JOIN course_reviews r ON r.offering_id = o.id
           WHERE c.deleted_at IS NULL
             AND ($1::text IS NULL OR c.code ILIKE $1 OR c.name ILIKE $1)
             AND ($2::text IS NULL OR EXISTS (
               SELECT 1 FROM offerings oi
               JOIN offering_faculty ofac ON ofac.offering_id = oi.id
               JOIN faculty f ON f.id = ofac.faculty_id
               WHERE oi.course_id = c.id AND oi.approved = true AND f.slug = $2
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

pub async fn update_course(pool: &PgPool, code: &str, patch: &PatchCourse) -> Result<CourseDetail, AppError> {
    let rows = sqlx::query!(
        r#"UPDATE courses SET name = $1, description = $2, type = $3 WHERE code = $4"#,
        patch.name,
        patch.description,
        patch.kind.clone() as CourseType,
        code
    )
    .execute(pool)
    .await?
    .rows_affected();
    if rows == 0 { return Err(AppError::NotFound); }
    get_by_code(pool, code).await
}

pub async fn create_offering(pool: &PgPool, course_code: &str, body: &CreateOffering) -> Result<OfferingDetail, AppError> {
    let course_id = id_by_code(pool, course_code).await?;
    let mut tx = pool.begin().await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO offerings (course_id, season, year, approved) VALUES ($1, $2, $3, true) RETURNING id"#,
        course_id, body.season.clone() as Season, body.year
    )
    .fetch_one(&mut *tx).await?;

    if let Some(ref fids) = body.faculty_ids {
        for fid in fids {
            sqlx::query!(
                "INSERT INTO offering_faculty (offering_id, faculty_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                id, fid
            )
            .execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;

    let frows = sqlx::query!(
        "SELECT f.id, f.slug, f.name FROM offering_faculty ofac JOIN faculty f ON f.id = ofac.faculty_id WHERE ofac.offering_id = $1",
        id
    )
    .fetch_all(pool).await?;

    let code = format!("{}{}", match &body.season { Season::M => "M", Season::S => "S" }, body.year);
    Ok(OfferingDetail {
        id, code, season: body.season.clone(), year: body.year,
        faculty: frows.into_iter().map(|f| FacultyRef { id: f.id, slug: f.slug, name: f.name }).collect(),
    })
}

pub async fn propose_offering(pool: &PgPool, course_code: &str, body: &CreateOffering) -> Result<OfferingDetail, AppError> {
    let course_id = id_by_code(pool, course_code).await?;
    let mut tx = pool.begin().await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO offerings (course_id, season, year, approved) VALUES ($1, $2, $3, false) RETURNING id"#,
        course_id, body.season.clone() as Season, body.year
    )
    .fetch_one(&mut *tx).await?;

    if let Some(ref fids) = body.faculty_ids {
        for fid in fids {
            sqlx::query!(
                "INSERT INTO offering_faculty (offering_id, faculty_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                id, fid
            )
            .execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;

    let frows = sqlx::query!(
        "SELECT f.id, f.slug, f.name FROM offering_faculty ofac JOIN faculty f ON f.id = ofac.faculty_id WHERE ofac.offering_id = $1",
        id
    )
    .fetch_all(pool).await?;

    let code = format!("{}{}", match &body.season { Season::M => "M", Season::S => "S" }, body.year);
    Ok(OfferingDetail {
        id, code, season: body.season.clone(), year: body.year,
        faculty: frows.into_iter().map(|f| FacultyRef { id: f.id, slug: f.slug, name: f.name }).collect(),
    })
}

pub async fn delete_offering(pool: &PgPool, offering_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!("DELETE FROM offerings WHERE id = $1", offering_id)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

pub async fn update_offering_faculty(pool: &PgPool, offering_id: &str, faculty_ids: &[String]) -> Result<OfferingDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, season as "season: Season", year FROM offerings WHERE id = $1"#,
        offering_id
    )
    .fetch_optional(pool).await?.ok_or(AppError::NotFound)?;

    sqlx::query!("DELETE FROM offering_faculty WHERE offering_id = $1", offering_id)
        .execute(pool).await?;
    for fid in faculty_ids {
        sqlx::query!(
            "INSERT INTO offering_faculty (offering_id, faculty_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            offering_id, fid
        )
        .execute(pool).await?;
    }

    let frows = sqlx::query!(
        "SELECT f.id, f.slug, f.name FROM offering_faculty ofac JOIN faculty f ON f.id = ofac.faculty_id WHERE ofac.offering_id = $1",
        offering_id
    )
    .fetch_all(pool).await?;

    let code = format!("{}{}", match &row.season { Season::M => "M", Season::S => "S" }, row.year);
    Ok(OfferingDetail {
        id: row.id, code, season: row.season, year: row.year,
        faculty: frows.into_iter().map(|f| FacultyRef { id: f.id, slug: f.slug, name: f.name }).collect(),
    })
}

pub async fn id_by_code(pool: &PgPool, code: &str) -> Result<String, AppError> {
    sqlx::query_scalar!("SELECT id FROM courses WHERE code = $1 AND deleted_at IS NULL", code)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn soft_delete(pool: &PgPool, code: &str) -> Result<(), AppError> {
    let n = sqlx::query!("UPDATE courses SET deleted_at = NOW() WHERE code = $1 AND deleted_at IS NULL", code)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

pub async fn restore(pool: &PgPool, code: &str) -> Result<(), AppError> {
    let n = sqlx::query!("UPDATE courses SET deleted_at = NULL WHERE code = $1 AND deleted_at IS NOT NULL", code)
        .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

#[derive(Debug, Serialize)]
pub struct DeletedCourse {
    pub code: String,
    pub name: String,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_deleted(pool: &PgPool) -> Result<Vec<DeletedCourse>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT code, name, deleted_at as "deleted_at!: chrono::DateTime<chrono::Utc>"
           FROM courses WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC"#
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| DeletedCourse { code: r.code, name: r.name, deleted_at: r.deleted_at }).collect())
}

pub async fn get_by_code(pool: &PgPool, code: &str) -> Result<CourseDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT c.id, c.code, c.name, c.description,
                  c.type as "kind: CourseType",
                  COALESCE(AVG((r.difficulty + r.teaching + r.grading + r.content + r.workload)::float / 5.0), 0.0)::float8 as "overall!: f64"
           FROM courses c
           LEFT JOIN offerings o ON o.course_id = c.id AND o.approved = true
           LEFT JOIN course_reviews r ON r.offering_id = o.id
           WHERE c.code = $1 AND c.deleted_at IS NULL
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
           WHERE o.course_id = $1 AND o.approved = true
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

    let proposed_rows = sqlx::query!(
        r#"SELECT id, season as "season: Season", year FROM offerings WHERE course_id = $1 AND approved = false ORDER BY year DESC, season"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let proposed_offerings: Vec<ProposedOfferingLean> = proposed_rows.into_iter().map(|pr| ProposedOfferingLean {
        id: pr.id, season: pr.season, year: pr.year
    }).collect();

    Ok(CourseDetail {
        id: row.id, code: row.code, name: row.name,
        description: row.description.unwrap_or_default(),
        kind: row.kind, overall: row.overall, offerings, proposed_offerings,
    })
}
