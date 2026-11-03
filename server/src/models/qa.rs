use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use ulid::Ulid;
use crate::{error::AppError, models::{audit, review::AuthorRef}};

#[derive(Debug, Serialize)]
pub struct Question {
    pub id: String,
    pub author: Option<AuthorRef>,
    pub anonymous: bool,
    pub body: String,
    pub votes: i64,
    pub voted: bool,
    pub mine: bool,
    pub answers: Vec<Answer>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct Answer {
    pub id: String,
    pub question_id: String,
    pub offering_id: Option<String>,
    pub author: Option<AuthorRef>,
    pub anonymous: bool,
    pub body: String,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub user_vote: Option<i16>,
    pub mine: bool,
    pub reviewer: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuestion {
    pub course: Option<String>,
    pub faculty: Option<String>,
    pub body: String,
    pub anonymous: bool,
}

#[derive(Debug, Deserialize)]
pub struct EditQuestion {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnswer {
    pub body: String,
    pub anonymous: bool,
    pub offering_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EditAnswer {
    pub body: String,
    pub offering_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Created {
    pub id: String,
}

/// trimmed body if its length is within bounds
fn checkbody(body: &str, min: usize, max: usize, what: &str) -> Result<String, AppError> {
    let body = body.trim();
    let n = body.chars().count();
    if n < min || n > max {
        return Err(AppError::BadRequest(format!("{what} must be {min}–{max} characters")));
    }
    Ok(body.to_string())
}

fn checkquestion(body: &str) -> Result<String, AppError> {
    checkbody(body, 10, 500, "question")
}

fn checkanswer(body: &str) -> Result<String, AppError> {
    checkbody(body, 2, 5000, "answer")
}

/// questions for a course or faculty, with answers inlined
pub async fn list(
    pool: &PgPool,
    course_id: Option<&str>,
    faculty_id: Option<&str>,
    user_id: &str,
    q: Option<&str>,
    sort: Option<&str>,
) -> Result<Vec<Question>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT q.id, q.user_id, q.anonymous, q.body,
                  q.edited_at as "edited_at?: DateTime<Utc>",
                  q.created_at as "created_at!: DateTime<Utc>",
                  u.display_name,
                  (SELECT COUNT(*) FROM question_votes v WHERE v.question_id = q.id) as "votes!",
                  EXISTS (SELECT 1 FROM question_votes v WHERE v.question_id = q.id AND v.user_id = $3) as "voted!",
                  CASE WHEN $4::text IS NULL THEN 0
                       ELSE ts_rank(q.search, websearch_to_tsquery('english', $4)) + word_similarity($4, q.body)
                  END::real as "rank!"
           FROM questions q
           JOIN users u ON u.id = q.user_id
           WHERE q.deleted_at IS NULL
             AND ($1::text IS NULL OR q.course_id = $1)
             AND ($2::text IS NULL OR q.faculty_id = $2)
             AND ($4::text IS NULL
                  OR q.search @@ websearch_to_tsquery('english', $4)
                  OR word_similarity($4, q.body) > 0.3
                  OR EXISTS (SELECT 1 FROM answers a
                             WHERE a.question_id = q.id AND a.deleted_at IS NULL
                               AND a.search @@ websearch_to_tsquery('english', $4)))"#,
        course_id, faculty_id, user_id, q
    )
    .fetch_all(pool)
    .await?;

    let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let mut answers = answers_for(pool, &ids, user_id).await?;

    let mut out: Vec<(f32, Question)> = rows.into_iter().map(|r| {
        let (mine, author) = (r.user_id == user_id, (!r.anonymous).then(|| AuthorRef { id: r.user_id, display_name: r.display_name }));
        let list = answers.remove(&r.id).unwrap_or_default();
        (r.rank, Question { id: r.id, author, anonymous: r.anonymous, body: r.body, votes: r.votes, voted: r.voted,
            mine, answers: list, edited_at: r.edited_at, created_at: r.created_at })
    }).collect();

    if sort == Some("unanswered") {
        out.retain(|(_, q)| q.answers.is_empty());
    }
    out.sort_by(|(ra, a), (rb, b)| {
        let byrank = rb.partial_cmp(ra).unwrap_or(std::cmp::Ordering::Equal);
        let bysort = match sort {
            Some("new") => b.created_at.cmp(&a.created_at),
            _ => b.votes.cmp(&a.votes).then(b.created_at.cmp(&a.created_at)),
        };
        byrank.then(bysort)
    });
    Ok(out.into_iter().map(|(_, q)| q).collect())
}

async fn answers_for(pool: &PgPool, ids: &[String], user_id: &str) -> Result<std::collections::HashMap<String, Vec<Answer>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT a.id, a.question_id, a.offering_id, a.user_id, a.anonymous, a.body,
                  a.edited_at as "edited_at?: DateTime<Utc>",
                  a.created_at as "created_at!: DateTime<Utc>",
                  u.display_name,
                  COALESCE(SUM(v.vote), 0)::bigint as "score!",
                  COALESCE(SUM(CASE WHEN v.vote = 1 THEN 1 ELSE 0 END), 0)::bigint as "upvotes!",
                  COALESCE(SUM(CASE WHEN v.vote = -1 THEN 1 ELSE 0 END), 0)::bigint as "downvotes!",
                  uv.vote as "user_vote?",
                  (EXISTS (SELECT 1 FROM course_reviews cr JOIN offerings o ON o.id = cr.offering_id
                           WHERE cr.user_id = a.user_id AND o.course_id = q.course_id AND cr.deleted_at IS NULL)
                   OR EXISTS (SELECT 1 FROM advisor_reviews ar
                              WHERE ar.user_id = a.user_id AND ar.faculty_id = q.faculty_id AND ar.deleted_at IS NULL)) as "reviewer!"
           FROM answers a
           JOIN questions q ON q.id = a.question_id
           JOIN users u ON u.id = a.user_id
           LEFT JOIN answer_votes v ON v.answer_id = a.id
           LEFT JOIN answer_votes uv ON uv.answer_id = a.id AND uv.user_id = $2
           WHERE a.question_id = ANY($1) AND a.deleted_at IS NULL
           GROUP BY a.id, q.id, u.display_name, uv.vote
           ORDER BY "score!" DESC, a.created_at ASC"#,
        ids, user_id
    )
    .fetch_all(pool)
    .await?;

    let mut map: std::collections::HashMap<String, Vec<Answer>> = std::collections::HashMap::new();
    for r in rows {
        let (mine, author) = (r.user_id == user_id, (!r.anonymous).then(|| AuthorRef { id: r.user_id, display_name: r.display_name }));
        map.entry(r.question_id.clone()).or_default().push(Answer {
            id: r.id, question_id: r.question_id, offering_id: r.offering_id, author, anonymous: r.anonymous,
            body: r.body, score: r.score, upvotes: r.upvotes, downvotes: r.downvotes, user_vote: r.user_vote,
            mine, reviewer: r.reviewer, edited_at: r.edited_at, created_at: r.created_at,
        });
    }
    Ok(map)
}

pub async fn create_question(pool: &PgPool, user_id: &str, course_id: Option<&str>, faculty_id: Option<&str>, body: &str, anonymous: bool) -> Result<String, AppError> {
    let body = checkquestion(body)?;
    let id = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO questions (id, user_id, course_id, faculty_id, anonymous, body) VALUES ($1, $2, $3, $4, $5, $6)",
        id, user_id, course_id, faculty_id, anonymous, body
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn edit_question(pool: &PgPool, id: &str, user_id: &str, body: &str) -> Result<(), AppError> {
    let body = checkquestion(body)?;
    let n = sqlx::query!(
        "UPDATE questions SET body = $1, edited_at = now() WHERE id = $2 AND user_id = $3 AND deleted_at IS NULL",
        body, id, user_id
    )
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(AppError::Forbidden); }
    Ok(())
}

/// course or faculty context of a question, for audit entries
async fn questioncontext(pool: &PgPool, id: &str) -> Result<Option<Value>, AppError> {
    let r = sqlx::query!(
        r#"SELECT q.body, c.code as "course_code?", c.name as "course_name?", f.slug as "faculty_slug?", f.name as "faculty_name?"
           FROM questions q
           LEFT JOIN courses c ON c.id = q.course_id
           LEFT JOIN faculty f ON f.id = q.faculty_id
           WHERE q.id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| serde_json::json!({
        "question": r.body, "course_code": r.course_code, "course_name": r.course_name,
        "faculty_slug": r.faculty_slug, "faculty_name": r.faculty_name,
    })))
}

/// moderators soft delete; authors can only delete questions nobody has answered
pub async fn delete_question(pool: &PgPool, id: &str, user_id: &str, is_admin: bool) -> Result<(), AppError> {
    if is_admin {
        return moddelete_question(pool, id, user_id).await;
    }
    let r = sqlx::query!(
        r#"SELECT EXISTS (SELECT 1 FROM answers a WHERE a.question_id = q.id AND a.deleted_at IS NULL) as "answered!"
           FROM questions q WHERE q.id = $1 AND q.user_id = $2 AND q.deleted_at IS NULL"#,
        id, user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Forbidden)?;
    if r.answered {
        return Err(AppError::BadRequest("questions with answers can't be deleted".into()));
    }
    sqlx::query!("DELETE FROM questions WHERE id = $1 AND user_id = $2", id, user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn moddelete_question(pool: &PgPool, id: &str, admin_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!(
        "UPDATE questions SET deleted_at = now(), deleted_by = $1 WHERE id = $2 AND deleted_at IS NULL",
        admin_id, id
    )
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(AppError::NotFound); }
    let prev = questioncontext(pool, id).await?;
    audit::logaction(pool, admin_id, "DELETE_QUESTION", "question", id, prev).await
}

pub async fn restore_question(pool: &PgPool, id: &str, admin_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!(
        "UPDATE questions SET deleted_at = NULL, deleted_by = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        id
    )
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(AppError::NotFound); }
    let prev = questioncontext(pool, id).await?;
    audit::logaction(pool, admin_id, "RESTORE_QUESTION", "question", id, prev).await
}

pub async fn vote_question(pool: &PgPool, id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO question_votes (user_id, question_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        user_id, id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unvote_question(pool: &PgPool, id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM question_votes WHERE user_id = $1 AND question_id = $2", user_id, id)
        .execute(pool)
        .await?;
    Ok(())
}

/// offering must be a live offering of the question's course
async fn checkoffering(pool: &PgPool, question_id: &str, offering_id: Option<&str>) -> Result<(), AppError> {
    let Some(offering_id) = offering_id else { return Ok(()) };
    let ok = sqlx::query!(
        r#"SELECT 1 as "one!" FROM questions q
           JOIN offerings o ON o.course_id = q.course_id
           WHERE q.id = $1 AND o.id = $2 AND o.approved = true AND o.deleted_at IS NULL"#,
        question_id, offering_id
    )
    .fetch_optional(pool)
    .await?;
    if ok.is_none() {
        return Err(AppError::BadRequest("offering doesn't belong to this course".into()));
    }
    Ok(())
}

pub async fn create_answer(pool: &PgPool, user_id: &str, question_id: &str, req: CreateAnswer) -> Result<String, AppError> {
    let body = checkanswer(&req.body)?;
    let r = sqlx::query!(
        r#"SELECT EXISTS (SELECT 1 FROM answers a WHERE a.question_id = q.id AND a.user_id = $2 AND a.deleted_at IS NULL) as "answered!"
           FROM questions q WHERE q.id = $1 AND q.deleted_at IS NULL"#,
        question_id, user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    if r.answered {
        return Err(AppError::BadRequest("you've already answered this question — edit your answer instead".into()));
    }
    checkoffering(pool, question_id, req.offering_id.as_deref()).await?;
    let id = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO answers (id, user_id, question_id, offering_id, anonymous, body) VALUES ($1, $2, $3, $4, $5, $6)",
        id, user_id, question_id, req.offering_id, req.anonymous, body
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn edit_answer(pool: &PgPool, id: &str, user_id: &str, req: EditAnswer) -> Result<(), AppError> {
    let body = checkanswer(&req.body)?;
    let r = sqlx::query!(
        "SELECT question_id FROM answers WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        id, user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Forbidden)?;
    checkoffering(pool, &r.question_id, req.offering_id.as_deref()).await?;
    sqlx::query!(
        "UPDATE answers SET body = $1, offering_id = $2, edited_at = now() WHERE id = $3",
        body, req.offering_id, id
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn answercontext(pool: &PgPool, id: &str) -> Result<Option<Value>, AppError> {
    let r = sqlx::query!("SELECT question_id, body FROM answers WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    let Some(r) = r else { return Ok(None) };
    let mut ctx = questioncontext(pool, &r.question_id).await?.unwrap_or_else(|| serde_json::json!({}));
    ctx["body"] = Value::String(r.body);
    Ok(Some(ctx))
}

pub async fn delete_answer(pool: &PgPool, id: &str, user_id: &str, is_admin: bool) -> Result<(), AppError> {
    if is_admin {
        return moddelete_answer(pool, id, user_id).await;
    }
    let n = sqlx::query!("DELETE FROM answers WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL", id, user_id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 { return Err(AppError::Forbidden); }
    Ok(())
}

pub async fn moddelete_answer(pool: &PgPool, id: &str, admin_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!(
        "UPDATE answers SET deleted_at = now(), deleted_by = $1 WHERE id = $2 AND deleted_at IS NULL",
        admin_id, id
    )
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(AppError::NotFound); }
    let prev = answercontext(pool, id).await?;
    audit::logaction(pool, admin_id, "DELETE_ANSWER", "answer", id, prev).await
}

/// fails if the author has since posted a new answer to the same question
pub async fn restore_answer(pool: &PgPool, id: &str, admin_id: &str) -> Result<(), AppError> {
    let n = sqlx::query!(
        "UPDATE answers SET deleted_at = NULL, deleted_by = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        id
    )
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(AppError::NotFound); }
    let prev = answercontext(pool, id).await?;
    audit::logaction(pool, admin_id, "RESTORE_ANSWER", "answer", id, prev).await
}

pub async fn vote_answer(pool: &PgPool, id: &str, user_id: &str, value: i16) -> Result<(), AppError> {
    let vid = Ulid::new().to_string();
    sqlx::query!(
        r#"INSERT INTO answer_votes (id, user_id, answer_id, vote) VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, answer_id) DO UPDATE SET vote = EXCLUDED.vote"#,
        vid, user_id, id, value
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unvote_answer(pool: &PgPool, id: &str, user_id: &str) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM answer_votes WHERE user_id = $1 AND answer_id = $2", user_id, id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn flag_question(pool: &PgPool, id: &str, user_id: &str, reason: String) -> Result<(), AppError> {
    let fid = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO qa_flags (id, user_id, question_id, reason) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        fid, user_id, id, reason
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn flag_answer(pool: &PgPool, id: &str, user_id: &str, reason: String) -> Result<(), AppError> {
    let fid = Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO qa_flags (id, user_id, answer_id, reason) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        fid, user_id, id, reason
    )
    .execute(pool)
    .await?;
    Ok(())
}
