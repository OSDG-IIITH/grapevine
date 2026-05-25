use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::{auth::session::AuthUser, error::AppError, models::review, state::AppState};

#[derive(Serialize)]
pub struct Me {
    pub id: String,
    pub display_name: String,
    pub is_admin: bool,
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
    let row = sqlx::query!("SELECT display_name FROM users WHERE id = $1", user.id)
        .fetch_one(&s.pool)
        .await?;
    Ok(Json(Me { id: user.id, display_name: row.display_name, is_admin: user.is_admin }))
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
    sqlx::query!("UPDATE users SET display_name = $1 WHERE id = $2", name, user.id)
        .execute(&s.pool)
        .await?;
    Ok(Json(Me { id: user.id, display_name: name, is_admin: user.is_admin }))
}

pub async fn my_reviews(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<MyReviewsResponse>, AppError> {
    let course = review::my_course_reviews(&s.pool, &user.id).await?;
    let advisor = review::my_advisor_reviews(&s.pool, &user.id).await?;
    Ok(Json(MyReviewsResponse { course, advisor }))
}
