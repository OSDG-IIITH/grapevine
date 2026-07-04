use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use ulid::Ulid;
use crate::{
    auth::{
        password,
        session::{AuthUser, AUTH_METHOD_KEY, USER_ID_KEY, VERIFIED_KEY},
        validate::{normalize_answer, normalize_username, validate_password},
    },
    error::AppError,
    models::review,
    state::AppState,
};

#[derive(Serialize)]
pub struct Me {
    pub id: String,
    pub display_name: String,
    pub is_admin: bool,
    pub verified: bool,
    pub username: Option<String>,
    pub auth_method: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    pub recovery_code: Option<String>,
    pub security_question: Option<String>,
    pub security_answer: Option<String>,
}

#[derive(Deserialize)]
pub struct ResetBody {
    pub username: String,
    pub new_password: String,
    pub recovery_code: Option<String>,
    pub security_answer: Option<String>,
}

#[derive(Serialize)]
pub struct ResetResponse {
    pub new_recovery_code: Option<String>,
}

#[derive(Serialize)]
pub struct RecoveryInfoResponse {
    pub has_recovery_code: bool,
    pub security_question: Option<String>,
}

#[derive(Serialize)]
pub struct MyReviewsResponse {
    pub course: Vec<review::CourseReview>,
    pub advisor: Vec<review::AdvisorReview>,
}

#[derive(Deserialize)]
pub struct UpdateMeBody {
    pub display_name: String,
}

/// Generate a recovery code: grapevine-XXXXXX-XXXXXX-XXXXXX-XXXXXX-XXXXXX
fn gen_recovery_code() -> String {
    use password_hash::rand_core::{OsRng, RngCore};
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut buf = [0u8; 30];
    OsRng.fill_bytes(&mut buf);
    let s: String = buf.iter().map(|b| CHARS[(*b as usize) % 62] as char).collect();
    format!("grapevine-{}-{}-{}-{}-{}", &s[0..6], &s[6..12], &s[12..18], &s[18..24], &s[24..30])
}

pub async fn me(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Me>, AppError> {
    let row = sqlx::query!(
        "SELECT display_name, username, cas_id, verified FROM users WHERE id = $1",
        user.id
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;
    let auth_method = if row.cas_id.is_some() { "cas" } else { "local" };
    Ok(Json(Me {
        id: user.id,
        display_name: row.display_name,
        is_admin: user.is_admin,
        verified: row.verified,
        username: row.username,
        auth_method: auth_method.to_string(),
    }))
}

pub async fn register(
    State(s): State<AppState>,
    session: Session,
    Json(body): Json<RegisterBody>,
) -> Result<Json<Me>, AppError> {
    validate_password(&body.password).map_err(AppError::BadRequest)?;
    let username = normalize_username(&body.username).map_err(AppError::BadRequest)?;
    let display_name = body.username.trim().to_string();
    let password_hash = password::hash(&body.password).map_err(|_| AppError::Internal)?;

    let recovery_code_hash = body.recovery_code
        .as_deref()
        .map(|c| password::hash(c))
        .transpose()
        .map_err(|_| AppError::Internal)?;

    let security_answer_hash = body.security_answer
        .as_deref()
        .map(|a| password::hash(&normalize_answer(a)))
        .transpose()
        .map_err(|_| AppError::Internal)?;

    let id = Ulid::new().to_string();
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash, display_name, verified,
                           recovery_code_hash, security_question, security_answer_hash)
        VALUES ($1, $2, $3, $4, false, $5, $6, $7)
        RETURNING id, display_name, is_admin, verified, username
        "#,
        id, username, password_hash, display_name,
        recovery_code_hash, body.security_question, security_answer_hash
    )
    .fetch_one(&s.pool)
    .await?;

    session.insert(USER_ID_KEY, &row.id).await.map_err(|_| AppError::Internal)?;
    session.insert(VERIFIED_KEY, false).await.map_err(|_| AppError::Internal)?;
    session.insert(AUTH_METHOD_KEY, "local").await.map_err(|_| AppError::Internal)?;

    Ok(Json(Me {
        id: row.id,
        display_name: row.display_name,
        is_admin: row.is_admin,
        verified: row.verified,
        username: row.username,
        auth_method: "local".to_string(),
    }))
}

pub async fn login_local(
    State(s): State<AppState>,
    session: Session,
    Json(body): Json<Credentials>,
) -> Result<Json<Me>, AppError> {
    let invalid = || AppError::BadRequest("invalid username or password".into());
    let username = normalize_username(&body.username).map_err(|_| invalid())?;

    let row = sqlx::query!(
        "SELECT id, display_name, is_admin, verified, username, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&s.pool)
    .await?;

    let Some(row) = row else {
        password::dummy_verify(&body.password);
        return Err(invalid());
    };
    let stored = row.password_hash.as_deref().ok_or_else(invalid)?;
    if !password::verify(&body.password, stored) {
        return Err(invalid());
    }

    session.insert(USER_ID_KEY, &row.id).await.map_err(|_| AppError::Internal)?;
    session.insert(VERIFIED_KEY, row.verified).await.map_err(|_| AppError::Internal)?;
    session.insert(AUTH_METHOD_KEY, "local").await.map_err(|_| AppError::Internal)?;

    Ok(Json(Me {
        id: row.id,
        display_name: row.display_name,
        is_admin: row.is_admin,
        verified: row.verified,
        username: row.username,
        auth_method: "local".to_string(),
    }))
}

pub async fn recovery_info(
    State(s): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<RecoveryInfoResponse>, AppError> {
    let username = normalize_username(&username).map_err(AppError::BadRequest)?;
    let row = sqlx::query!(
        "SELECT recovery_code_hash, security_question FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("user not found".into()))?;

    Ok(Json(RecoveryInfoResponse {
        has_recovery_code: row.recovery_code_hash.is_some(),
        security_question: row.security_question,
    }))
}

pub async fn reset_password(
    State(s): State<AppState>,
    Json(body): Json<ResetBody>,
) -> Result<Json<ResetResponse>, AppError> {
    validate_password(&body.new_password).map_err(AppError::BadRequest)?;
    let username = normalize_username(&body.username).map_err(AppError::BadRequest)?;
    let invalid = || AppError::BadRequest("invalid credentials".into());

    let row = sqlx::query!(
        "SELECT id, recovery_code_hash, security_answer_hash FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or_else(invalid)?;

    let new_password_hash = password::hash(&body.new_password).map_err(|_| AppError::Internal)?;

    if let Some(code) = &body.recovery_code {
        let stored = row.recovery_code_hash.as_deref()
            .ok_or_else(|| AppError::BadRequest("no recovery code set".into()))?;
        if !password::verify(code, stored) {
            return Err(invalid());
        }
        let new_code = gen_recovery_code();
        let new_code_hash = password::hash(&new_code).map_err(|_| AppError::Internal)?;
        sqlx::query!(
            "UPDATE users SET password_hash = $1, recovery_code_hash = $2 WHERE id = $3",
            new_password_hash, new_code_hash, row.id
        )
        .execute(&s.pool)
        .await?;
        return Ok(Json(ResetResponse { new_recovery_code: Some(new_code) }));
    }

    if let Some(answer) = &body.security_answer {
        let stored = row.security_answer_hash.as_deref()
            .ok_or_else(|| AppError::BadRequest("no security question set".into()))?;
        if !password::verify(&normalize_answer(answer), stored) {
            return Err(invalid());
        }
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            new_password_hash, row.id
        )
        .execute(&s.pool)
        .await?;
        return Ok(Json(ResetResponse { new_recovery_code: None }));
    }

    Err(AppError::BadRequest("provide recovery_code or security_answer".into()))
}

pub async fn update_me(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateMeBody>,
) -> Result<Json<Me>, AppError> {
    let name = body.display_name.trim().to_string();
    if name.is_empty() || name.len() > 80 {
        return Err(AppError::BadRequest("display_name must be 1–80 chars".into()));
    }
    let row = sqlx::query!(
        "UPDATE users SET display_name = $1 WHERE id = $2 RETURNING username, cas_id, verified",
        name, user.id
    )
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;
    let auth_method = if row.cas_id.is_some() { "cas" } else { "local" };
    Ok(Json(Me {
        id: user.id,
        display_name: name,
        is_admin: user.is_admin,
        verified: row.verified,
        username: row.username,
        auth_method: auth_method.to_string(),
    }))
}

pub async fn my_reviews(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<MyReviewsResponse>, AppError> {
    let course = review::my_course_reviews(&s.pool, &user.id).await?;
    let advisor = review::my_advisor_reviews(&s.pool, &user.id).await?;
    Ok(Json(MyReviewsResponse { course, advisor }))
}
