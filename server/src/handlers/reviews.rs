use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::{auth::session::{AuthUser, MaybeAuth}, error::AppError, models::{faculty, review::{self, *}}, state::AppState};

pub async fn offering_reviews(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Path(id): Path<String>,
) -> Result<Json<Vec<CourseReview>>, AppError> {
    Ok(Json(review::course_reviews_by_offering(&s.pool, &id, &user_id).await?))
}

pub async fn create_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(offering_id): Path<String>,
    Json(body): Json<CreateCourseReview>,
) -> Result<(StatusCode, Json<CourseReview>), AppError> {
    let r = review::create_course_review(&s.pool, &user.id, &offering_id, body).await?;
    Ok((StatusCode::CREATED, Json(r)))
}

pub async fn edit_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<EditCourseReview>,
) -> Result<Json<CourseReview>, AppError> {
    Ok(Json(review::edit_course_review(&s.pool, &id, &user.id, body).await?))
}

pub async fn delete_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    review::delete_course_review(&s.pool, &id, &user.id, user.is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vote_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<StatusCode, AppError> {
    if body.value != 1 && body.value != -1 {
        return Err(AppError::BadRequest("value must be 1 or -1".into()));
    }
    review::upsert_course_vote(&s.pool, &id, &user.id, body.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unvote_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    review::delete_course_vote(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn flag_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<FlagBody>,
) -> Result<StatusCode, AppError> {
    review::flag_course_review(&s.pool, &id, &user.id, body.reason).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateAdvisorReview>,
) -> Result<(StatusCode, Json<AdvisorReview>), AppError> {
    let fac_id = faculty::id_by_slug(&s.pool, &slug).await?;
    let r = review::create_advisor_review(&s.pool, &user.id, &fac_id, body).await?;
    Ok((StatusCode::CREATED, Json(r)))
}

pub async fn edit_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<EditAdvisorReview>,
) -> Result<Json<AdvisorReview>, AppError> {
    Ok(Json(review::edit_advisor_review(&s.pool, &id, &user.id, body).await?))
}

pub async fn delete_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    review::delete_advisor_review(&s.pool, &id, &user.id, user.is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vote_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<StatusCode, AppError> {
    if body.value != 1 && body.value != -1 {
        return Err(AppError::BadRequest("value must be 1 or -1".into()));
    }
    review::upsert_advisor_vote(&s.pool, &id, &user.id, body.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unvote_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    review::delete_advisor_vote(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vote_legacy_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<StatusCode, AppError> {
    if body.value != 1 && body.value != -1 {
        return Err(AppError::BadRequest("value must be 1 or -1".into()));
    }
    review::upsert_legacy_course_vote(&s.pool, &id, &user.id, body.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unvote_legacy_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    review::delete_legacy_course_vote(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vote_legacy_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<StatusCode, AppError> {
    if body.value != 1 && body.value != -1 {
        return Err(AppError::BadRequest("value must be 1 or -1".into()));
    }
    review::upsert_legacy_advisor_vote(&s.pool, &id, &user.id, body.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unvote_legacy_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    review::delete_legacy_advisor_vote(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn flag_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<FlagBody>,
) -> Result<StatusCode, AppError> {
    review::flag_advisor_review(&s.pool, &id, &user.id, body.reason).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_legacy_course_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    review::delete_legacy_course_review(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_legacy_advisor_review(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin { return Err(AppError::Forbidden); }
    review::delete_legacy_advisor_review(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
