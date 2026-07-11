use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{
    auth::session::{AuthUser, MaybeAuth},
    error::AppError,
    models::{course, review, offering::Season},
    state::AppState,
};

pub async fn create(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<course::CreateCourse>,
) -> Result<(StatusCode, Json<course::CourseDetail>), AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok((StatusCode::CREATED, Json(course::create_course(&s.pool, &body).await?)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    instructor: Option<String>,
    sort: Option<String>,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<course::CourseLean>>, AppError> {
    Ok(Json(course::list(&s.pool, q.q.as_deref(), q.instructor.as_deref(), q.sort.as_deref()).await?))
}

pub async fn get(
    State(s): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<course::CourseDetail>, AppError> {
    Ok(Json(course::get_by_code(&s.pool, &code).await?))
}

pub async fn update(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<course::PatchCourse>,
) -> Result<Json<course::CourseDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(course::update_course(&s.pool, &code, &body).await?))
}

pub async fn create_offering(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<course::CreateOffering>,
) -> Result<Json<course::OfferingDetail>, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    Ok(Json(course::create_offering(&s.pool, &code, &body).await?))
}

pub async fn reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(code): Path<String>,
) -> Result<Json<Vec<review::CourseReview>>, AppError> {
    let id = course::id_by_code(&s.pool, &code).await?;
    Ok(Json(review::course_reviews_by_course(&s.pool, &id, &user_id).await?))
}

pub async fn propose_offering(
    State(s): State<AppState>,
    _user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<course::CreateOffering>,
) -> Result<Json<course::OfferingDetail>, AppError> {
    Ok(Json(course::propose_offering(&s.pool, &code, &body).await?))
}

pub async fn proposed_reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(code): Path<String>,
) -> Result<Json<Vec<review::CourseReview>>, AppError> {
    let id = course::id_by_code(&s.pool, &code).await?;
    Ok(Json(review::proposed_course_reviews_by_course(&s.pool, &id, &user_id).await?))
}

#[derive(Deserialize)]
pub struct ProposeReviewRequest {
    pub season: Season,
    pub year: i16,
    pub anonymous: bool,
    pub difficulty: i16,
    pub teaching: i16,
    pub grading: i16,
    pub content: i16,
    pub workload: i16,
    pub body: String,
    pub faculty_ids: Option<Vec<String>>,
}

pub async fn delete(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    course::soft_delete(&s.pool, &code).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn propose_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<ProposeReviewRequest>,
) -> Result<(StatusCode, Json<review::CourseReview>), AppError> {
    let mut tx = s.pool.begin().await?;

    let course_id = course::id_by_code(&s.pool, &code).await?;

    let existing_id = sqlx::query_scalar!(
        r#"SELECT id FROM offerings WHERE course_id = $1 AND season = $2::offering_season AND year = $3"#,
        course_id, body.season.clone() as Season, body.year
    )
    .fetch_optional(&mut *tx)
    .await?;

    let offering_id = match existing_id {
        Some(id) => id,
        None => {
            let id = ulid::Ulid::new().to_string();
            sqlx::query!(
                "INSERT INTO offerings (id, course_id, season, year, approved) VALUES ($1, $2, $3::offering_season, $4, false)",
                id, course_id, body.season.clone() as Season, body.year
            )
            .execute(&mut *tx)
            .await?;
            id
        }
    };

    if let Some(ref fids) = body.faculty_ids {
        for fid in fids {
            sqlx::query!(
                "INSERT INTO offering_faculty (offering_id, faculty_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                offering_id, fid
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    let review_id = ulid::Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO course_reviews (id, user_id, offering_id, anonymous, difficulty, teaching, grading, content, workload, body)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        review_id, user.id, offering_id, body.anonymous, body.difficulty, body.teaching, body.grading, body.content, body.workload, body.body
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let r_row = sqlx::query!(
        r#"SELECT cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                  cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                  cr.body,
                  cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name
           FROM course_reviews cr
           JOIN users u ON u.id = cr.user_id
           WHERE cr.id = $1"#,
        review_id
    )
    .fetch_one(&s.pool)
    .await?;

    let overall = (r_row.difficulty + r_row.teaching + r_row.grading + r_row.content + r_row.workload) as f32 / 5.0;
    let overall = (overall * 100.0).round() / 100.0;

    let author = if r_row.anonymous { None } else { Some(review::AuthorRef { id: r_row.user_id, display_name: r_row.display_name }) };
    let r = review::CourseReview {
        id: r_row.id,
        offering_id: r_row.offering_id,
        author,
        anonymous: r_row.anonymous,
        difficulty: r_row.difficulty,
        teaching: r_row.teaching,
        grading: r_row.grading,
        content: r_row.content,
        workload: r_row.workload,
        overall,
        body: r_row.body,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        user_vote: None,
        edited_at: None,
        created_at: r_row.created_at,
    };

    Ok((StatusCode::CREATED, Json(r)))
}
