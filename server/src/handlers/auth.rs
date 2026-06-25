use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use ulid::Ulid;
use crate::{
    auth::{
        password,
        session::{AuthUser, AUTH_METHOD_KEY, USER_ID_KEY, VERIFIED_KEY},
        validate::{normalize_username, validate_password},
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

#[derive(Serialize)]
pub struct MyReviewsResponse {
    pub course: Vec<review::CourseReview>,
    pub advisor: Vec<review::AdvisorReview>,
}

#[derive(Deserialize)]
pub struct UpdateMeBody {
    pub display_name: String,
}

pub async fn me(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Me>, AppError> {
    let row = sqlx::query!(
        "SELECT display_name, username, cas_id, verified FROM users WHERE id = $1",
        user.id
    )
    .fetch_one(&s.pool)
    .await?;
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
    Json(body): Json<Credentials>,
) -> Result<Json<Me>, AppError> {
    validate_password(&body.password).map_err(AppError::BadRequest)?;
    let username = normalize_username(&body.username).map_err(AppError::BadRequest)?;
    let display_name = body.username.trim().to_string();
    let password_hash = password::hash(&body.password).map_err(|_| AppError::Internal)?;

    let id = Ulid::new().to_string();
    // Duplicate username hits the UNIQUE constraint -> 23505 -> BadRequest("already exists").
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash, display_name, verified)
        VALUES ($1, $2, $3, $4, false)
        RETURNING id, display_name, is_admin, verified, username
        "#,
        id, username, password_hash, display_name
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
    // Generic error for both missing user and bad password (no enumeration).
    let invalid = || AppError::BadRequest("invalid username or password".into());
    let username = normalize_username(&body.username).map_err(|_| invalid())?;

    let row = sqlx::query!(
        r#"
        SELECT id, display_name, is_admin, verified, username, password_hash
        FROM users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(&s.pool)
    .await?;

    // Always run exactly one argon2 verification, even when the username doesn't
    // exist, so a missing user and a wrong password take the same time.
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
    .fetch_one(&s.pool)
    .await?;
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
