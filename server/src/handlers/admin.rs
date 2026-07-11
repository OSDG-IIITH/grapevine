use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::{auth::session::AuthUser, error::AppError, state::AppState};

#[derive(Serialize)]
struct SeedLab { name: String, shortname: String, description: Option<String> }

#[derive(Serialize)]
struct SeedFaculty { name: String, slug: String, labs: Vec<String>, bio: Option<String> }

#[derive(Serialize)]
struct SeedCourse { code: String, name: String, description: String }

#[derive(Serialize)]
struct SeedOffering { course: String, season: String, year: i16, faculty: Vec<String> }

#[derive(Serialize)]
pub struct SeedExport {
    labs: Vec<SeedLab>,
    faculty: Vec<SeedFaculty>,
    courses: Vec<SeedCourse>,
    offerings: Vec<SeedOffering>,
}

pub async fn export(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<SeedExport>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let labs = sqlx::query!(
        "SELECT name, shortname, description FROM labs ORDER BY shortname"
    )
    .fetch_all(&s.pool).await?;

    let faculty = sqlx::query!(
        r#"SELECT f.name, f.slug, f.bio,
                  array_remove(array_agg(l.shortname ORDER BY l.shortname), NULL) as "labs!: Vec<String>"
           FROM faculty f
           LEFT JOIN faculty_labs fl ON fl.faculty_id = f.id
           LEFT JOIN labs l ON l.id = fl.lab_id
           GROUP BY f.id, f.name, f.slug, f.bio
           ORDER BY f.slug"#
    )
    .fetch_all(&s.pool).await?;

    let courses = sqlx::query!(
        r#"SELECT code, name, description FROM courses ORDER BY code"#
    )
    .fetch_all(&s.pool).await?;

    let offerings = sqlx::query!(
        r#"SELECT c.code as course, o.season::text as "season!", o.year,
                  array_remove(array_agg(f.slug ORDER BY f.slug), NULL) as "faculty!: Vec<String>"
           FROM offerings o
           JOIN courses c ON c.id = o.course_id
           LEFT JOIN offering_faculty ofac ON ofac.offering_id = o.id
           LEFT JOIN faculty f ON f.id = ofac.faculty_id
           GROUP BY o.id, c.code, o.season, o.year
           ORDER BY c.code, o.year, o.season"#
    )
    .fetch_all(&s.pool).await?;

    Ok(Json(SeedExport {
        labs: labs.into_iter().map(|r| SeedLab { name: r.name, shortname: r.shortname, description: r.description }).collect(),
        faculty: faculty.into_iter().map(|r| SeedFaculty { name: r.name, slug: r.slug, bio: r.bio, labs: r.labs }).collect(),
        courses: courses.into_iter().map(|r| SeedCourse { code: r.code, name: r.name, description: r.description.unwrap_or_default() }).collect(),
        offerings: offerings.into_iter().map(|r| SeedOffering { course: r.course, season: r.season, year: r.year, faculty: r.faculty }).collect(),
    }))
}

#[derive(Debug, Serialize)]
pub struct ReporterRef {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub id: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub review_type: &'static str,
    pub review_id: String,
    pub review_body: String,
    pub reporter: ReporterRef,
}

pub async fn flags(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<FlagResponse>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let course_flags = sqlx::query!(
        r#"SELECT f.id, f.reason,
                  f.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                  f.review_id,
                  LEFT(cr.body, 200) as "review_body?",
                  u.id as reporter_id, u.display_name as reporter_name
           FROM course_review_flags f
           JOIN course_reviews cr ON cr.id = f.review_id
           JOIN users u ON u.id = f.user_id
           ORDER BY f.created_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    let advisor_flags = sqlx::query!(
        r#"SELECT f.id, f.reason,
                  f.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                  f.review_id,
                  LEFT(ar.body, 200) as "review_body?",
                  u.id as reporter_id, u.display_name as reporter_name
           FROM advisor_review_flags f
           JOIN advisor_reviews ar ON ar.id = f.review_id
           JOIN users u ON u.id = f.user_id
           ORDER BY f.created_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    let mut result: Vec<FlagResponse> = course_flags.into_iter().map(|r| FlagResponse {
        id: r.id,
        reason: r.reason,
        created_at: r.created_at,
        review_type: "course",
        review_id: r.review_id,
        review_body: r.review_body.unwrap_or_default(),
        reporter: ReporterRef { id: r.reporter_id, display_name: r.reporter_name },
    }).collect();

    result.extend(advisor_flags.into_iter().map(|r| FlagResponse {
        id: r.id,
        reason: r.reason,
        created_at: r.created_at,
        review_type: "advisor",
        review_id: r.review_id,
        review_body: r.review_body.unwrap_or_default(),
        reporter: ReporterRef { id: r.reporter_id, display_name: r.reporter_name },
    }));

    result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(result))
}

pub async fn dismiss_flag(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let rows = sqlx::query!("DELETE FROM course_review_flags WHERE id = $1", id)
        .execute(&s.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        sqlx::query!("DELETE FROM advisor_review_flags WHERE id = $1", id)
            .execute(&s.pool)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let course_flag = sqlx::query!(
        "SELECT review_id FROM course_review_flags WHERE id = $1",
        id
    )
    .fetch_optional(&s.pool)
    .await?;

    if let Some(f) = course_flag {
        sqlx::query!("DELETE FROM course_reviews WHERE id = $1", f.review_id)
            .execute(&s.pool)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let advisor_flag = sqlx::query!(
        "SELECT review_id FROM advisor_review_flags WHERE id = $1",
        id
    )
    .fetch_optional(&s.pool)
    .await?;

    if let Some(f) = advisor_flag {
        sqlx::query!("DELETE FROM advisor_reviews WHERE id = $1", f.review_id)
            .execute(&s.pool)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    Err(AppError::NotFound)
}

use crate::models::{course, faculty, lab, offering::Season};

#[derive(Serialize)]
pub struct ProposedOfferingResponse {
    pub id: String,
    pub course_code: String,
    pub course_name: String,
    pub season: String,
    pub year: i16,
    pub faculty: Vec<String>,
    pub reviews: Vec<ProposedReviewResponse>,
}

#[derive(Serialize)]
pub struct ProposedReviewResponse {
    pub id: String,
    pub body: String,
    pub difficulty: i16,
    pub teaching: i16,
    pub grading: i16,
    pub content: i16,
    pub workload: i16,
    pub author_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_proposed(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ProposedOfferingResponse>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let offerings = sqlx::query!(
        r#"SELECT o.id, o.season as "season: Season", o.year,
                  c.code as course_code, c.name as course_name,
                  COALESCE(array_remove(array_agg(f.name ORDER BY f.name), NULL), '{}') as "faculty!: Vec<String>"
           FROM offerings o
           JOIN courses c ON c.id = o.course_id
           LEFT JOIN offering_faculty ofac ON ofac.offering_id = o.id
           LEFT JOIN faculty f ON f.id = ofac.faculty_id
           WHERE o.approved = false
           GROUP BY o.id, c.code, c.name, o.season, o.year, o.created_at
           ORDER BY o.created_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    let mut result = Vec::new();
    for o in offerings {
        let reviews = sqlx::query!(
            r#"SELECT cr.id, cr.body, cr.difficulty, cr.teaching, cr.grading,
                      cr.content, cr.workload,
                      cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                      cr.anonymous,
                      u.display_name
               FROM course_reviews cr
               JOIN users u ON u.id = cr.user_id
               WHERE cr.offering_id = $1
               ORDER BY cr.created_at DESC"#,
            o.id
        )
        .fetch_all(&s.pool)
        .await?;

        let mapped_reviews = reviews.into_iter().map(|r| ProposedReviewResponse {
            id: r.id,
            body: r.body,
            difficulty: r.difficulty,
            teaching: r.teaching,
            grading: r.grading,
            content: r.content,
            workload: r.workload,
            author_name: if r.anonymous { None } else { Some(r.display_name) },
            created_at: r.created_at,
        }).collect();

        result.push(ProposedOfferingResponse {
            id: o.id,
            course_code: o.course_code,
            course_name: o.course_name,
            season: match o.season { Season::M => "M".to_string(), Season::S => "S".to_string() },
            year: o.year,
            faculty: o.faculty,
            reviews: mapped_reviews,
        });
    }

    Ok(Json(result))
}

pub async fn approve_proposed(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let rows = sqlx::query!(
        "UPDATE offerings SET approved = true WHERE id = $1 AND approved = false",
        id
    )
    .execute(&s.pool)
    .await?
    .rows_affected();

    if rows == 0 { return Err(AppError::NotFound); }
    Ok(StatusCode::OK)
}

pub async fn reject_proposed(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let rows = sqlx::query!("DELETE FROM offerings WHERE id = $1 AND approved = false", id)
        .execute(&s.pool)
        .await?
        .rows_affected();

    if rows == 0 { return Err(AppError::NotFound); }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn deleted_courses(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<course::DeletedCourse>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(course::list_deleted(&s.pool).await?))
}

pub async fn deleted_faculty(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<faculty::DeletedFaculty>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(faculty::list_deleted(&s.pool).await?))
}

pub async fn restore_faculty(
    State(s): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    faculty::restore(&s.pool, &slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn deleted_labs(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<lab::DeletedLab>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(lab::list_deleted(&s.pool).await?))
}

pub async fn restore_lab(
    State(s): State<AppState>,
    user: AuthUser,
    Path(shortname): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    lab::restore(&s.pool, &shortname).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_course(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    course::restore(&s.pool, &code).await?;
    Ok(StatusCode::NO_CONTENT)
}
