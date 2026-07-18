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
pub struct CreateCourse {
    pub code: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchCourse {
    pub code: Option<String>,
    pub name: String,
    pub description: String,
    pub predecessor_ids: Option<Vec<String>>,
    pub successor_ids: Option<Vec<String>>,
    pub shortnames: Option<Vec<String>>,
}



#[derive(Debug, Serialize)]
pub struct CourseLean {
    pub id: String,
    pub code: String,
    pub name: String,
    pub overall: f64,
    pub shortnames: Vec<String>,
    pub reviews_count: i64,
    pub difficulty: f64,
    pub workload: f64,
    pub teaching: f64,
    pub grading: f64,
    pub content: f64,
}

#[derive(Debug, Serialize)]
pub struct CourseRef {
    pub id: String,
    pub code: String,
    pub name: String,
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
    pub overall: f64,
    pub shortnames: Vec<String>,
    pub predecessors: Vec<CourseRef>,
    pub successors: Vec<CourseRef>,
    pub offerings: Vec<OfferingDetail>,
    pub proposed_offerings: Vec<ProposedOfferingLean>,
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

pub async fn list(pool: &PgPool, q: Option<&str>, instructor: Option<&str>, sort: Option<&str>) -> Result<Vec<CourseLean>, AppError> {
    let pattern = q.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"SELECT c.id, c.code, c.name, c.shortnames,
                  COALESCE(AVG(r.overall::float8), 0.0)::float8 as "overall!: f64",
                  COALESCE(AVG(r.difficulty::float8), 0.0)::float8 as "difficulty!: f64",
                  COALESCE(AVG(r.workload::float8), 0.0)::float8 as "workload!: f64",
                  COALESCE(AVG(r.teaching::float8), 0.0)::float8 as "teaching!: f64",
                  COALESCE(AVG(r.grading::float8), 0.0)::float8 as "grading!: f64",
                  COALESCE(AVG(r.content::float8), 0.0)::float8 as "content!: f64",
                  (COUNT(r.id) + COALESCE((SELECT COUNT(*) FROM legacy_course_reviews lr WHERE lr.course_id = c.id), 0) + COALESCE((SELECT COUNT(*) FROM external_course_reviews er WHERE er.course_id = c.id), 0))::int8 as "reviews_count!: i64"
           FROM courses c
           LEFT JOIN offerings o ON o.course_id = c.id AND o.approved = true AND o.deleted_at IS NULL
           LEFT JOIN course_reviews r ON r.offering_id = o.id AND r.deleted_at IS NULL
           WHERE c.deleted_at IS NULL
             AND ($1::text IS NULL OR c.code ILIKE $1 OR c.name ILIKE $1)
             AND ($2::text IS NULL OR EXISTS (
               SELECT 1 FROM offerings oi
               JOIN offering_faculty ofac ON ofac.offering_id = oi.id
               JOIN faculty f ON f.id = ofac.faculty_id
               WHERE oi.course_id = c.id AND oi.approved = true AND oi.deleted_at IS NULL AND f.slug = $2
             ))
           GROUP BY c.id, c.code, c.name, c.shortnames
           ORDER BY c.name"#,
        pattern,
        instructor
    )
    .fetch_all(pool)
    .await?;

    let mut results: Vec<CourseLean> = rows.into_iter().map(|r| CourseLean {
        id: r.id, code: r.code, name: r.name, overall: r.overall,
        shortnames: r.shortnames, reviews_count: r.reviews_count,
        difficulty: r.difficulty, workload: r.workload,
        teaching: r.teaching, grading: r.grading, content: r.content,
    }).collect();

    let rcmp = |a: i64, b: i64, desc: bool| -> std::cmp::Ordering {
        match (a == 0, b == 0) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => if desc { b.cmp(&a) } else { a.cmp(&b) },
        }
    };

    match sort {
        Some("name_asc")        => results.sort_by(|a, b| a.name.cmp(&b.name)),
        Some("name_desc")       => results.sort_by(|a, b| b.name.cmp(&a.name)),
        Some("reviews_desc")    => results.sort_by(|a, b| rcmp(a.reviews_count, b.reviews_count, true)),
        Some("reviews_asc")     => results.sort_by(|a, b| rcmp(a.reviews_count, b.reviews_count, false)),
        Some("rating_desc")     => results.sort_by(|a, b| fdesc(a.overall, b.overall)),
        Some("rating_asc")      => results.sort_by(|a, b| fasc(a.overall, b.overall)),
        Some("difficulty_desc") => results.sort_by(|a, b| fdesc(a.difficulty, b.difficulty)),
        Some("difficulty_asc")  => results.sort_by(|a, b| fasc(a.difficulty, b.difficulty)),
        Some("workload_desc")   => results.sort_by(|a, b| fdesc(a.workload, b.workload)),
        Some("workload_asc")    => results.sort_by(|a, b| fasc(a.workload, b.workload)),
        Some("teaching_desc")   => results.sort_by(|a, b| fdesc(a.teaching, b.teaching)),
        Some("teaching_asc")    => results.sort_by(|a, b| fasc(a.teaching, b.teaching)),
        Some("grading_desc")    => results.sort_by(|a, b| fdesc(a.grading, b.grading)),
        Some("grading_asc")     => results.sort_by(|a, b| fasc(a.grading, b.grading)),
        Some("content_desc")    => results.sort_by(|a, b| fdesc(a.content, b.content)),
        Some("content_asc")     => results.sort_by(|a, b| fasc(a.content, b.content)),
        _ => results.sort_by(|a, b| fdesc(a.overall, b.overall)),
    }

    Ok(results)
}

pub async fn update_course(pool: &PgPool, code: &str, patch: &PatchCourse) -> Result<CourseDetail, AppError> {
    let mut tx = pool.begin().await?;

    let new_code = patch.code.as_deref().unwrap_or(code);
    let shortnames = patch.shortnames.clone().unwrap_or_default();
    let id = sqlx::query_scalar!(
        r#"UPDATE courses SET name = $1, description = $2, shortnames = $3, code = $5 WHERE code = $4 AND deleted_at IS NULL RETURNING id"#,
        patch.name, patch.description, &shortnames, code, new_code
    )
    .fetch_optional(&mut *tx).await?
    .ok_or(AppError::NotFound)?;

    if let Some(ref pids) = patch.predecessor_ids {
        sqlx::query!("DELETE FROM course_succession WHERE successor_id = $1", id)
            .execute(&mut *tx).await?;
        for pid in pids {
            sqlx::query!(
                "INSERT INTO course_succession (predecessor_id, successor_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                pid, id
            )
            .execute(&mut *tx).await?;
        }
    }

    if let Some(ref sids) = patch.successor_ids {
        sqlx::query!("DELETE FROM course_succession WHERE predecessor_id = $1", id)
            .execute(&mut *tx).await?;
        for sid in sids {
            sqlx::query!(
                "INSERT INTO course_succession (predecessor_id, successor_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                id, sid
            )
            .execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;
    get_by_code(pool, new_code).await
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

pub async fn delete_offering(pool: &PgPool, offering_id: &str, admin_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!(
        "UPDATE offerings SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2 AND deleted_at IS NULL",
        admin_id, offering_id
    )
    .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

pub async fn restore_offering(pool: &PgPool, offering_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!(
        "UPDATE offerings SET deleted_at = NULL, deleted_by = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        offering_id
    )
    .execute(pool).await?.rows_affected();
    if n == 0 { Err(AppError::NotFound) } else { Ok(()) }
}

pub async fn update_offering_faculty(pool: &PgPool, offering_id: &str, faculty_ids: &[String]) -> Result<OfferingDetail, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, season as "season: Season", year FROM offerings WHERE id = $1 AND deleted_at IS NULL"#,
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
        r#"SELECT c.id, c.code, c.name, c.description, c.shortnames,
                  COALESCE(AVG(r.overall::float8), 0.0)::float8 as "overall!: f64"
           FROM courses c
           LEFT JOIN offerings o ON o.course_id = c.id AND o.approved = true AND o.deleted_at IS NULL
           LEFT JOIN course_reviews r ON r.offering_id = o.id AND r.deleted_at IS NULL
           WHERE c.code = $1 AND c.deleted_at IS NULL
           GROUP BY c.id, c.code, c.name, c.description, c.shortnames"#,
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
           WHERE o.course_id = $1 AND o.approved = true AND o.deleted_at IS NULL
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
        r#"SELECT id, season as "season: Season", year FROM offerings WHERE course_id = $1 AND approved = false AND deleted_at IS NULL ORDER BY year DESC, season"#,
        row.id
    )
    .fetch_all(pool)
    .await?;

    let proposed_offerings: Vec<ProposedOfferingLean> = proposed_rows.into_iter().map(|pr| ProposedOfferingLean {
        id: pr.id, season: pr.season, year: pr.year
    }).collect();

    let pred_rows = sqlx::query!(
        "SELECT c.id, c.code, c.name FROM courses c
         JOIN course_succession cs ON cs.predecessor_id = c.id
         WHERE cs.successor_id = $1 AND c.deleted_at IS NULL",
        row.id
    )
    .fetch_all(pool).await?;

    let succ_rows = sqlx::query!(
        "SELECT c.id, c.code, c.name FROM courses c
         JOIN course_succession cs ON cs.successor_id = c.id
         WHERE cs.predecessor_id = $1 AND c.deleted_at IS NULL",
        row.id
    )
    .fetch_all(pool).await?;

    let predecessors = pred_rows.into_iter().map(|r| CourseRef { id: r.id, code: r.code, name: r.name }).collect();
    let successors = succ_rows.into_iter().map(|r| CourseRef { id: r.id, code: r.code, name: r.name }).collect();

    Ok(CourseDetail {
        id: row.id, code: row.code, name: row.name,
        description: row.description.unwrap_or_default(),
        overall: row.overall,
        shortnames: row.shortnames,
        predecessors, successors, offerings, proposed_offerings,
    })
}

pub async fn create_course(pool: &PgPool, body: &CreateCourse) -> Result<CourseDetail, AppError> {
    let id = ulid::Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO courses (id, code, name, description, shortnames) VALUES ($1, $2, $3, $4, '{}')"#,
        id, body.code, body.name, body.description
    )
    .execute(pool).await?;
    get_by_code(pool, &body.code).await
}
