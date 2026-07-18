use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{
    auth::{session::AuthUser, validate::normalize_username},
    error::AppError,
    models::{audit, course, faculty, lab, offering::Season, report, review},
    state::AppState,
};

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
           WHERE o.deleted_at IS NULL
           GROUP BY o.id, c.code, o.season, o.year
           ORDER BY c.code, o.year, o.season"#
    )
    .fetch_all(&s.pool).await?;

    audit::logaction(&s.pool, &user.id, "EXPORT_SEED_DATA", "database", "all", None).await?;
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
           JOIN course_reviews cr ON cr.id = f.review_id AND cr.deleted_at IS NULL
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
           JOIN advisor_reviews ar ON ar.id = f.review_id AND ar.deleted_at IS NULL
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

    audit::logaction(&s.pool, &user.id, "DISMISS_FLAG", "flag", &id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reports(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<report::ReportResponse>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(report::list(&s.pool).await?))
}

pub async fn dismiss_report(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    report::dismiss(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn approve_report(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    report::approve_faculty_suggestion(&s.pool, &id, &user.id).await?;
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
        let review = sqlx::query!(
            r#"SELECT cr.body, c.code as course_code, c.name as course_name
               FROM course_reviews cr
               JOIN offerings o ON o.id = cr.offering_id
               JOIN courses c ON c.id = o.course_id
               WHERE cr.id = $1 AND cr.deleted_at IS NULL"#,
            f.review_id
        )
        .fetch_optional(&s.pool)
        .await?;

        sqlx::query!(
            "UPDATE course_reviews SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2 AND deleted_at IS NULL",
            user.id, f.review_id
        )
        .execute(&s.pool)
        .await?;

        let prev = review.map(|r| serde_json::json!({ "body": r.body, "course_code": r.course_code, "course_name": r.course_name }));
        audit::logaction(&s.pool, &user.id, "DELETE_REVIEW", "course_review", &f.review_id, prev).await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let advisor_flag = sqlx::query!(
        "SELECT review_id FROM advisor_review_flags WHERE id = $1",
        id
    )
    .fetch_optional(&s.pool)
    .await?;

    if let Some(f) = advisor_flag {
        let review = sqlx::query!(
            r#"SELECT ar.body, f.slug as faculty_slug, f.name as faculty_name
               FROM advisor_reviews ar
               JOIN faculty f ON f.id = ar.faculty_id
               WHERE ar.id = $1 AND ar.deleted_at IS NULL"#,
            f.review_id
        )
        .fetch_optional(&s.pool)
        .await?;

        sqlx::query!(
            "UPDATE advisor_reviews SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2 AND deleted_at IS NULL",
            user.id, f.review_id
        )
        .execute(&s.pool)
        .await?;

        let prev = review.map(|r| serde_json::json!({ "body": r.body, "faculty_slug": r.faculty_slug, "faculty_name": r.faculty_name }));
        audit::logaction(&s.pool, &user.id, "DELETE_REVIEW", "advisor_review", &f.review_id, prev).await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    Err(AppError::NotFound)
}

pub async fn restore_review_course(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let n = sqlx::query!(
        "UPDATE course_reviews SET deleted_at = NULL, deleted_by = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        id
    )
    .execute(&s.pool)
    .await?
    .rows_affected();

    if n == 0 { return Err(AppError::NotFound); }

    let ctx = sqlx::query!(
        r#"SELECT c.code as course_code, c.name as course_name
           FROM course_reviews cr
           JOIN offerings o ON o.id = cr.offering_id
           JOIN courses c ON c.id = o.course_id
           WHERE cr.id = $1"#,
        id
    )
    .fetch_optional(&s.pool)
    .await?;
    let prev = ctx.map(|r| serde_json::json!({ "course_code": r.course_code, "course_name": r.course_name }));
    audit::logaction(&s.pool, &user.id, "RESTORE_REVIEW", "course_review", &id, prev).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_review_advisor(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let n = sqlx::query!(
        "UPDATE advisor_reviews SET deleted_at = NULL, deleted_by = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        id
    )
    .execute(&s.pool)
    .await?
    .rows_affected();

    if n == 0 { return Err(AppError::NotFound); }

    let ctx = sqlx::query!(
        r#"SELECT f.slug as faculty_slug, f.name as faculty_name
           FROM advisor_reviews ar
           JOIN faculty f ON f.id = ar.faculty_id
           WHERE ar.id = $1"#,
        id
    )
    .fetch_optional(&s.pool)
    .await?;
    let prev = ctx.map(|r| serde_json::json!({ "faculty_slug": r.faculty_slug, "faculty_name": r.faculty_name }));
    audit::logaction(&s.pool, &user.id, "RESTORE_REVIEW", "advisor_review", &id, prev).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_offering_handler(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    course::restore_offering(&s.pool, &id).await?;
    audit::logaction(&s.pool, &user.id, "RESTORE_OFFERING", "offering", &id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

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
    pub overall: f32,
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
           WHERE o.approved = false AND o.deleted_at IS NULL
           GROUP BY o.id, c.code, c.name, o.season, o.year, o.created_at
           ORDER BY o.created_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    let mut result = Vec::new();
    for o in offerings {
        let reviews = sqlx::query!(
            r#"SELECT cr.id, cr.body, cr.difficulty, cr.teaching, cr.grading,
                      cr.content, cr.workload, cr.overall as "overall!: f32",
                      cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                      cr.anonymous,
                      u.display_name
               FROM course_reviews cr
               JOIN users u ON u.id = cr.user_id
               WHERE cr.offering_id = $1 AND cr.deleted_at IS NULL
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
            overall: r.overall,
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
        "UPDATE offerings SET approved = true WHERE id = $1 AND approved = false AND deleted_at IS NULL",
        id
    )
    .execute(&s.pool)
    .await?
    .rows_affected();

    if rows == 0 { return Err(AppError::NotFound); }
    audit::logaction(&s.pool, &user.id, "APPROVE_PROPOSED", "offering", &id, None).await?;
    Ok(StatusCode::OK)
}

pub async fn reject_proposed(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let offering = sqlx::query!(
        r#"SELECT o.id, c.code as course_code, o.season::text as "season!", o.year
           FROM offerings o
           JOIN courses c ON c.id = o.course_id
           WHERE o.id = $1 AND o.approved = false AND o.deleted_at IS NULL"#,
        id
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query!(
        "UPDATE offerings SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2 AND approved = false AND deleted_at IS NULL",
        user.id, id
    )
    .execute(&s.pool)
    .await?;

    let prev = serde_json::json!({ "course_code": offering.course_code, "season": offering.season, "year": offering.year });
    audit::logaction(&s.pool, &user.id, "REJECT_PROPOSED", "offering", &id, Some(prev)).await?;
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
    audit::logaction(&s.pool, &user.id, "RESTORE_FACULTY", "faculty", &slug, None).await?;
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
    audit::logaction(&s.pool, &user.id, "RESTORE_LAB", "lab", &shortname, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn restore_course(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    course::restore(&s.pool, &code).await?;
    audit::logaction(&s.pool, &user.id, "RESTORE_COURSE", "course", &code, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AuditQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    admin_id: Option<String>,
    action: Option<String>,
    target_type: Option<String>,
}

pub async fn audit_logs(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<AuditQuery>,
) -> Result<Json<audit::AuditPage>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);
    Ok(Json(audit::list(&s.pool, limit, offset, q.admin_id.as_deref(), q.action.as_deref(), q.target_type.as_deref()).await?))
}

#[derive(Serialize)]
pub struct ModeratorRow {
    pub id: String,
    pub display_name: String,
    pub username: Option<String>,
    pub cas_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_moderators(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ModeratorRow>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    let rows = sqlx::query!(
        r#"SELECT id, display_name, username, cas_id,
                  created_at as "created_at!: DateTime<Utc>"
           FROM users WHERE is_admin = true ORDER BY display_name"#
    )
    .fetch_all(&s.pool).await?;
    Ok(Json(rows.into_iter().map(|r| ModeratorRow {
        id: r.id,
        display_name: r.display_name,
        username: r.username,
        cas_id: r.cas_id,
        created_at: r.created_at,
    }).collect()))
}

fn name_from_cas_id(cas_id: &str) -> String {
    let local = cas_id.split('@').next().unwrap_or(cas_id);
    let mut parts = Vec::new();
    for part in local.split(|c| c == '.' || c == '_' || c == '-') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        let first = chars.next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
        let rest = chars.as_str().to_lowercase();
        parts.push(format!("{}{}", first, rest));
    }
    if parts.is_empty() {
        cas_id.to_string()
    } else {
        parts.join(" ")
    }
}

#[derive(Deserialize)]
pub struct AddCasMod {
    pub cas_id: String,
}

pub async fn add_cas_moderator(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<AddCasMod>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let cas_id = body.cas_id.trim().to_lowercase();
    if cas_id.is_empty() { return Err(AppError::BadRequest("cas_id required".into())); }

    let display_name = name_from_cas_id(&cas_id);
    let id = ulid::Ulid::new().to_string();

    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, cas_id, display_name, is_admin, verified)
        VALUES ($1, $2, $3, true, true)
        ON CONFLICT (cas_id) DO UPDATE
            SET is_admin = true
        RETURNING id
        "#,
        id, cas_id, display_name
    )
    .fetch_one(&s.pool)
    .await?;

    audit::logaction(&s.pool, &user.id, "ADD_MODERATOR", "user", &row.id,
        Some(serde_json::json!({ "cas_id": cas_id }))).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AddLocalMod {
    pub username: String,
    pub display_name: Option<String>,
    pub password: Option<String>,
}

pub async fn add_local_moderator(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<AddLocalMod>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let username = normalize_username(&body.username).map_err(AppError::BadRequest)?;

    // Check if user already exists
    let existing = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_optional(&s.pool)
        .await?;

    let user_id = if let Some(row) = existing {
        sqlx::query!("UPDATE users SET is_admin = true WHERE id = $1", row.id)
            .execute(&s.pool)
            .await?;
        row.id
    } else {
        let password = body.password.as_deref()
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| AppError::BadRequest("Password is required to create a new local account".into()))?;
        
        crate::auth::validate::validate_password(password).map_err(AppError::BadRequest)?;

        let display_name = body.display_name.as_deref()
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| body.username.trim().to_string());

        let password_hash = crate::auth::password::hash(password).map_err(|_| AppError::Internal)?;
        let id = ulid::Ulid::new().to_string();

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, password_hash, display_name, is_admin, verified)
            VALUES ($1, $2, $3, $4, true, true)
            "#,
            id, username, password_hash, display_name
        )
        .execute(&s.pool)
        .await?;
        
        id
    };

    audit::logaction(&s.pool, &user.id, "ADD_MODERATOR", "user", &user_id,
        Some(serde_json::json!({ "username": username }))).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn demote_moderator(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    if id == user.id { return Err(AppError::BadRequest("cannot demote yourself".into())); }

    let rows = sqlx::query!(
        "UPDATE users SET is_admin = false WHERE id = $1 AND is_admin = true RETURNING id",
        id
    )
    .fetch_optional(&s.pool).await?;

    if rows.is_none() { return Err(AppError::NotFound); }
    audit::logaction(&s.pool, &user.id, "REMOVE_MODERATOR", "user", &id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct DeletedOffering {
    pub id: String,
    pub course_code: String,
    pub course_name: String,
    pub season: String,
    pub year: i16,
    pub deleted_at: DateTime<Utc>,
}

pub async fn deleted_offerings(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<DeletedOffering>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }

    let rows = sqlx::query!(
        r#"SELECT o.id, c.code as course_code, c.name as course_name,
                  o.season::text as "season!", o.year,
                  o.deleted_at as "deleted_at!: DateTime<Utc>"
           FROM offerings o
           JOIN courses c ON c.id = o.course_id
           WHERE o.deleted_at IS NOT NULL
           ORDER BY o.deleted_at DESC"#
    )
    .fetch_all(&s.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| DeletedOffering {
        id: r.id,
        course_code: r.course_code,
        course_name: r.course_name,
        season: r.season,
        year: r.year,
        deleted_at: r.deleted_at,
    }).collect()))
}

pub async fn list_external_course_reviews(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<review::AdminExternalCourseReview>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(review::all_external_course_reviews(&s.pool).await?))
}

pub async fn list_external_advisor_reviews(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<review::AdminExternalAdvisorReview>>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(review::all_external_advisor_reviews(&s.pool).await?))
}
