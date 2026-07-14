use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ulid::Ulid;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct AuthorRef {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct CourseReview {
    pub id: String,
    pub offering_id: String,
    pub author: Option<AuthorRef>,
    pub anonymous: bool,
    pub difficulty: i16,
    pub teaching: i16,
    pub grading: i16,
    pub content: i16,
    pub workload: i16,
    pub overall: f32,
    pub body: String,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub user_vote: Option<i16>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdvisorReview {
    pub id: String,
    pub faculty_id: String,
    pub author: Option<AuthorRef>,
    pub anonymous: bool,
    pub research: i16,
    pub availability: i16,
    pub mentorship: i16,
    pub support: i16,
    pub workload: i16,
    pub overall: f32,
    pub body: String,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub user_vote: Option<i16>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCourseReview {
    pub anonymous: bool,
    pub difficulty: i16,
    pub teaching: i16,
    pub grading: i16,
    pub content: i16,
    pub workload: i16,
    pub overall: f32,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct EditCourseReview {
    pub difficulty: Option<i16>,
    pub teaching: Option<i16>,
    pub grading: Option<i16>,
    pub content: Option<i16>,
    pub workload: Option<i16>,
    pub overall: Option<f32>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdvisorReview {
    pub anonymous: bool,
    pub research: i16,
    pub availability: i16,
    pub mentorship: i16,
    pub support: i16,
    pub workload: i16,
    pub overall: f32,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct EditAdvisorReview {
    pub research: Option<i16>,
    pub availability: Option<i16>,
    pub mentorship: Option<i16>,
    pub support: Option<i16>,
    pub workload: Option<i16>,
    pub overall: Option<f32>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LegacyCourseReview {
    pub id: String,
    pub body: Option<String>,
    pub original_rating: Option<i16>,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub user_vote: Option<i16>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LegacyAdvisorReview {
    pub id: String,
    pub body: Option<String>,
    pub original_rating: Option<i16>,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub user_vote: Option<i16>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VoteBody {
    pub value: i16,
}

#[derive(Debug, Deserialize)]
pub struct FlagBody {
    pub reason: String,
}

pub async fn course_reviews_by_course(pool: &PgPool, course_id: &str, user_id: &str) -> Result<Vec<CourseReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                  cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                  cr.overall as "overall!: f32",
                  cr.body,
                  cr.edited_at as "edited_at?: chrono::DateTime<chrono::Utc>",
                  cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                    COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                    COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?"
           FROM course_reviews cr
           JOIN offerings o ON o.id = cr.offering_id AND o.deleted_at IS NULL
           JOIN users u ON u.id = cr.user_id
           LEFT JOIN course_review_votes v ON v.review_id = cr.id
           LEFT JOIN course_review_votes uv ON uv.review_id = cr.id AND uv.user_id = $2
           WHERE o.course_id = $1 AND o.approved = true AND cr.deleted_at IS NULL
           GROUP BY cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                    cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                    cr.overall, cr.body, cr.edited_at, cr.created_at, u.display_name, uv.vote
           ORDER BY "score!" DESC, cr.created_at DESC"#,
        course_id, user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let author = if r.anonymous { None } else { Some(AuthorRef { id: r.user_id, display_name: r.display_name }) };
        CourseReview { id: r.id, offering_id: r.offering_id, author, anonymous: r.anonymous,
            difficulty: r.difficulty, teaching: r.teaching, grading: r.grading, content: r.content,
            workload: r.workload, overall: r.overall, body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
            user_vote: r.user_vote, edited_at: r.edited_at, created_at: r.created_at }
    }).collect())
}

pub async fn course_reviews_by_offering(pool: &PgPool, offering_id: &str, user_id: &str) -> Result<Vec<CourseReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                  cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                  cr.overall as "overall!: f32",
                  cr.body,
                  cr.edited_at as "edited_at?: chrono::DateTime<chrono::Utc>",
                  cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                    COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                    COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?"
           FROM course_reviews cr
           JOIN offerings o ON o.id = cr.offering_id AND o.deleted_at IS NULL
           JOIN users u ON u.id = cr.user_id
           LEFT JOIN course_review_votes v ON v.review_id = cr.id
           LEFT JOIN course_review_votes uv ON uv.review_id = cr.id AND uv.user_id = $2
           WHERE cr.offering_id = $1 AND o.approved = true AND cr.deleted_at IS NULL
           GROUP BY cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                    cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                    cr.overall, cr.body, cr.edited_at, cr.created_at, u.display_name, uv.vote
           ORDER BY "score!" DESC, cr.created_at DESC"#,
        offering_id, user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let author = if r.anonymous { None } else { Some(AuthorRef { id: r.user_id, display_name: r.display_name }) };
        CourseReview { id: r.id, offering_id: r.offering_id, author, anonymous: r.anonymous,
            difficulty: r.difficulty, teaching: r.teaching, grading: r.grading, content: r.content,
            workload: r.workload, overall: r.overall, body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
            user_vote: r.user_vote, edited_at: r.edited_at, created_at: r.created_at }
    }).collect())
}

pub async fn proposed_course_reviews_by_course(pool: &PgPool, course_id: &str, user_id: &str) -> Result<Vec<CourseReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                  cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                  cr.overall as "overall!: f32",
                  cr.body,
                  cr.edited_at as "edited_at?: chrono::DateTime<chrono::Utc>",
                  cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                    COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                    COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?"
           FROM course_reviews cr
           JOIN offerings o ON o.id = cr.offering_id AND o.deleted_at IS NULL
           JOIN users u ON u.id = cr.user_id
           LEFT JOIN course_review_votes v ON v.review_id = cr.id
           LEFT JOIN course_review_votes uv ON uv.review_id = cr.id AND uv.user_id = $2
           WHERE o.course_id = $1 AND o.approved = false AND cr.deleted_at IS NULL
           GROUP BY cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                    cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                    cr.overall, cr.body, cr.edited_at, cr.created_at, u.display_name, uv.vote
           ORDER BY cr.created_at DESC"#,
        course_id, user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let author = if r.anonymous { None } else { Some(AuthorRef { id: r.user_id, display_name: r.display_name }) };
        CourseReview { id: r.id, offering_id: r.offering_id, author, anonymous: r.anonymous,
            difficulty: r.difficulty, teaching: r.teaching, grading: r.grading, content: r.content,
            workload: r.workload, overall: r.overall, body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
            user_vote: r.user_vote, edited_at: r.edited_at, created_at: r.created_at }
    }).collect())
}

pub async fn create_course_review(pool: &PgPool, user_id: &str, offering_id: &str, req: CreateCourseReview) -> Result<CourseReview, AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO course_reviews (id, user_id, offering_id, anonymous, difficulty, teaching, grading, content, workload, overall, body)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        id, user_id, offering_id, req.anonymous, req.difficulty, req.teaching, req.grading, req.content, req.workload, req.overall, req.body
    )
    .execute(pool)
    .await?;

    let rows = course_reviews_by_offering(pool, offering_id, user_id).await?;
    rows.into_iter().find(|r| r.id == id).ok_or(AppError::Internal)
}

pub async fn edit_course_review(pool: &PgPool, review_id: &str, user_id: &str, req: EditCourseReview) -> Result<CourseReview, AppError> {
    let row = sqlx::query!(
        r#"UPDATE course_reviews SET
             difficulty = COALESCE($1, difficulty),
             teaching = COALESCE($2, teaching),
             grading = COALESCE($3, grading),
             content = COALESCE($4, content),
             workload = COALESCE($5, workload),
             body = COALESCE($6, body),
             overall = COALESCE($7, overall),
             edited_at = now()
           WHERE id = $8 AND user_id = $9 AND deleted_at IS NULL
           RETURNING id, offering_id"#,
        req.difficulty, req.teaching, req.grading, req.content, req.workload, req.body, req.overall,
        review_id, user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Forbidden)?;

    let reviews = course_reviews_by_offering(pool, &row.offering_id, user_id).await?;
    reviews.into_iter().find(|r| r.id == row.id).ok_or(AppError::Internal)
}

pub async fn delete_course_review(pool: &PgPool, review_id: &str, user_id: &str) -> Result<(), AppError> {
    let rows = sqlx::query!(
        "DELETE FROM course_reviews WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        review_id, user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 { return Err(AppError::Forbidden); }
    Ok(())
}

pub async fn upsert_course_vote(pool: &PgPool, review_id: &str, user_id: &str, value: i16) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO course_review_votes (id, user_id, review_id, vote)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, review_id) DO UPDATE SET vote = EXCLUDED.vote"#,
        id, user_id, review_id, value
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_course_vote(pool: &PgPool, review_id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM course_review_votes WHERE user_id = $1 AND review_id = $2",
        user_id, review_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn flag_course_review(pool: &PgPool, review_id: &str, user_id: &str, reason: String) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO course_review_flags (id, user_id, review_id, reason)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, review_id) DO NOTHING"#,
        id, user_id, review_id, reason
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn advisor_reviews_by_faculty(pool: &PgPool, faculty_id: &str, user_id: &str) -> Result<Vec<AdvisorReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT ar.id, ar.faculty_id, ar.user_id, ar.anonymous,
                  ar.research, ar.availability, ar.mentorship, ar.support, ar.workload,
                  ar.overall as "overall!: f32",
                  ar.body,
                  ar.edited_at as "edited_at?: chrono::DateTime<chrono::Utc>",
                  ar.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                    COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                    COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?"
           FROM advisor_reviews ar
           JOIN users u ON u.id = ar.user_id
           LEFT JOIN advisor_review_votes v ON v.review_id = ar.id
           LEFT JOIN advisor_review_votes uv ON uv.review_id = ar.id AND uv.user_id = $2
           WHERE ar.faculty_id = $1 AND ar.deleted_at IS NULL
           GROUP BY ar.id, ar.faculty_id, ar.user_id, ar.anonymous,
                    ar.research, ar.availability, ar.mentorship, ar.support, ar.workload,
                    ar.overall, ar.body, ar.edited_at, ar.created_at, u.display_name, uv.vote
           ORDER BY "score!" DESC, ar.created_at DESC"#,
        faculty_id, user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let author = if r.anonymous { None } else { Some(AuthorRef { id: r.user_id, display_name: r.display_name }) };
        AdvisorReview { id: r.id, faculty_id: r.faculty_id, author, anonymous: r.anonymous,
            research: r.research, availability: r.availability, mentorship: r.mentorship, support: r.support,
            workload: r.workload, overall: r.overall, body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
            user_vote: r.user_vote, edited_at: r.edited_at, created_at: r.created_at }
    }).collect())
}

pub async fn create_advisor_review(pool: &PgPool, user_id: &str, faculty_id: &str, req: CreateAdvisorReview) -> Result<AdvisorReview, AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO advisor_reviews (id, user_id, faculty_id, anonymous, research, availability, mentorship, support, workload, overall, body)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        id, user_id, faculty_id, req.anonymous, req.research, req.availability, req.mentorship, req.support, req.workload, req.overall, req.body
    )
    .execute(pool)
    .await?;

    let reviews = advisor_reviews_by_faculty(pool, faculty_id, user_id).await?;
    reviews.into_iter().find(|r| r.id == id).ok_or(AppError::Internal)
}

pub async fn edit_advisor_review(pool: &PgPool, review_id: &str, user_id: &str, req: EditAdvisorReview) -> Result<AdvisorReview, AppError> {
    let row = sqlx::query!(
        r#"UPDATE advisor_reviews SET
             research = COALESCE($1, research),
             availability = COALESCE($2, availability),
             mentorship = COALESCE($3, mentorship),
             support = COALESCE($4, support),
             workload = COALESCE($5, workload),
             body = COALESCE($6, body),
             overall = COALESCE($7, overall),
             edited_at = now()
           WHERE id = $8 AND user_id = $9 AND deleted_at IS NULL
           RETURNING id, faculty_id"#,
        req.research, req.availability, req.mentorship, req.support, req.workload, req.body, req.overall,
        review_id, user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Forbidden)?;

    let reviews = advisor_reviews_by_faculty(pool, &row.faculty_id, user_id).await?;
    reviews.into_iter().find(|r| r.id == row.id).ok_or(AppError::Internal)
}

pub async fn delete_advisor_review(pool: &PgPool, review_id: &str, user_id: &str) -> Result<(), AppError> {
    let rows = sqlx::query!(
        "DELETE FROM advisor_reviews WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        review_id, user_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 { return Err(AppError::Forbidden); }
    Ok(())
}

pub async fn upsert_advisor_vote(pool: &PgPool, review_id: &str, user_id: &str, value: i16) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO advisor_review_votes (id, user_id, review_id, vote)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, review_id) DO UPDATE SET vote = EXCLUDED.vote"#,
        id, user_id, review_id, value
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_advisor_vote(pool: &PgPool, review_id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM advisor_review_votes WHERE user_id = $1 AND review_id = $2",
        user_id, review_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn legacy_course_reviews_by_course(pool: &PgPool, course_id: &str, user_id: &str) -> Result<Vec<LegacyCourseReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT lcr.id, lcr.body, lcr.original_rating,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                  COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                  COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?",
                  lcr.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
           FROM legacy_course_reviews lcr
           LEFT JOIN legacy_course_review_votes v ON v.review_id = lcr.id
           LEFT JOIN legacy_course_review_votes uv ON uv.review_id = lcr.id AND uv.user_id = $2
           WHERE lcr.course_id = $1
           GROUP BY lcr.id, lcr.body, lcr.original_rating, lcr.created_at, uv.vote
           ORDER BY "score!" DESC, lcr.created_at DESC"#,
        course_id, user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| LegacyCourseReview {
        id: r.id, body: r.body, original_rating: r.original_rating,
        score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
        user_vote: r.user_vote, created_at: r.created_at
    }).collect())
}

pub async fn legacy_advisor_reviews_by_faculty(pool: &PgPool, faculty_id: &str, user_id: &str) -> Result<Vec<LegacyAdvisorReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT lar.id, lar.body, lar.original_rating,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                  COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                  COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?",
                  lar.created_at as "created_at!: chrono::DateTime<chrono::Utc>"
           FROM legacy_advisor_reviews lar
           LEFT JOIN legacy_advisor_review_votes v ON v.review_id = lar.id
           LEFT JOIN legacy_advisor_review_votes uv ON uv.review_id = lar.id AND uv.user_id = $2
           WHERE lar.faculty_id = $1
           GROUP BY lar.id, lar.body, lar.original_rating, lar.created_at, uv.vote
           ORDER BY "score!" DESC, lar.created_at DESC"#,
        faculty_id, user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| LegacyAdvisorReview {
        id: r.id, body: r.body, original_rating: r.original_rating,
        score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
        user_vote: r.user_vote, created_at: r.created_at
    }).collect())
}

pub async fn upsert_legacy_course_vote(pool: &PgPool, review_id: &str, user_id: &str, value: i16) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO legacy_course_review_votes (id, user_id, review_id, vote)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, review_id) DO UPDATE SET vote = EXCLUDED.vote"#,
        id, user_id, review_id, value
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_legacy_course_vote(pool: &PgPool, review_id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM legacy_course_review_votes WHERE user_id = $1 AND review_id = $2",
        user_id, review_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_legacy_advisor_vote(pool: &PgPool, review_id: &str, user_id: &str, value: i16) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO legacy_advisor_review_votes (id, user_id, review_id, vote)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, review_id) DO UPDATE SET vote = EXCLUDED.vote"#,
        id, user_id, review_id, value
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_legacy_advisor_vote(pool: &PgPool, review_id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM legacy_advisor_review_votes WHERE user_id = $1 AND review_id = $2",
        user_id, review_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn flag_advisor_review(pool: &PgPool, review_id: &str, user_id: &str, reason: String) -> Result<(), AppError> {
    let id = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO advisor_review_flags (id, user_id, review_id, reason)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, review_id) DO NOTHING"#,
        id, user_id, review_id, reason
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn my_course_reviews(pool: &PgPool, user_id: &str) -> Result<Vec<CourseReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                  cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                  cr.overall as "overall!: f32",
                  cr.body,
                  cr.edited_at as "edited_at?: chrono::DateTime<chrono::Utc>",
                  cr.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                  COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                  COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?"
           FROM course_reviews cr
           JOIN users u ON u.id = cr.user_id
           LEFT JOIN course_review_votes v ON v.review_id = cr.id
           LEFT JOIN course_review_votes uv ON uv.review_id = cr.id AND uv.user_id = $1
           WHERE cr.user_id = $1 AND cr.deleted_at IS NULL
           GROUP BY cr.id, cr.offering_id, cr.user_id, cr.anonymous,
                    cr.difficulty, cr.teaching, cr.grading, cr.content, cr.workload,
                    cr.overall, cr.body, cr.edited_at, cr.created_at, u.display_name, uv.vote
           ORDER BY cr.created_at DESC"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let author = Some(AuthorRef { id: r.user_id, display_name: r.display_name });
        CourseReview { id: r.id, offering_id: r.offering_id, author, anonymous: r.anonymous,
            difficulty: r.difficulty, teaching: r.teaching, grading: r.grading, content: r.content,
            workload: r.workload, overall: r.overall, body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
            user_vote: r.user_vote, edited_at: r.edited_at, created_at: r.created_at }
    }).collect())
}

pub async fn my_advisor_reviews(pool: &PgPool, user_id: &str) -> Result<Vec<AdvisorReview>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT ar.id, ar.faculty_id, ar.user_id, ar.anonymous,
                  ar.research, ar.availability, ar.mentorship, ar.support, ar.workload,
                  ar.overall as "overall!: f32",
                  ar.body,
                  ar.edited_at as "edited_at?: chrono::DateTime<chrono::Utc>",
                  ar.created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                  COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                  COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?"
           FROM advisor_reviews ar
           JOIN users u ON u.id = ar.user_id
           LEFT JOIN advisor_review_votes v ON v.review_id = ar.id
           LEFT JOIN advisor_review_votes uv ON uv.review_id = ar.id AND uv.user_id = $1
           WHERE ar.user_id = $1 AND ar.deleted_at IS NULL
           GROUP BY ar.id, ar.faculty_id, ar.user_id, ar.anonymous,
                    ar.research, ar.availability, ar.mentorship, ar.support, ar.workload,
                    ar.overall, ar.body, ar.edited_at, ar.created_at, u.display_name, uv.vote
           ORDER BY ar.created_at DESC"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let author = Some(AuthorRef { id: r.user_id, display_name: r.display_name });
        AdvisorReview { id: r.id, faculty_id: r.faculty_id, author, anonymous: r.anonymous,
            research: r.research, availability: r.availability, mentorship: r.mentorship, support: r.support,
            workload: r.workload, overall: r.overall, body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes,
            user_vote: r.user_vote, edited_at: r.edited_at, created_at: r.created_at }
    }).collect())
}
