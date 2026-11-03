use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{
    auth::session::{AuthUser, MaybeAuth},
    error::AppError,
    models::{course, faculty, qa::{self, *}, review::{FlagBody, VoteBody}},
    state::AppState,
};

#[derive(Deserialize)]
pub struct ListQuery {
    course: Option<String>,
    faculty: Option<String>,
    q: Option<String>,
    sort: Option<String>,
}

/// resolve exactly one of a course code or faculty slug to (course_id, faculty_id)
async fn target(s: &AppState, course: Option<&str>, faculty: Option<&str>) -> Result<(Option<String>, Option<String>), AppError> {
    match (course, faculty) {
        (Some(code), None) => Ok((Some(course::id_by_code(&s.pool, code).await?), None)),
        (None, Some(slug)) => Ok((None, Some(faculty::id_by_slug(&s.pool, slug).await?))),
        _ => Err(AppError::BadRequest("pass exactly one of course or faculty".into())),
    }
}

pub async fn list(
    State(s): State<AppState>,
    MaybeAuth(user_id): MaybeAuth,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Question>>, AppError> {
    let (cid, fid) = target(&s, q.course.as_deref(), q.faculty.as_deref()).await?;
    let search = q.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    Ok(Json(qa::list(&s.pool, cid.as_deref(), fid.as_deref(), &user_id, search, q.sort.as_deref()).await?))
}

pub async fn create_question(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateQuestion>,
) -> Result<(StatusCode, Json<Created>), AppError> {
    let (cid, fid) = target(&s, body.course.as_deref(), body.faculty.as_deref()).await?;
    let id = qa::create_question(&s.pool, &user.id, cid.as_deref(), fid.as_deref(), &body.body, body.anonymous).await?;
    Ok((StatusCode::CREATED, Json(Created { id })))
}

pub async fn edit_question(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<EditQuestion>,
) -> Result<StatusCode, AppError> {
    qa::edit_question(&s.pool, &id, &user.id, &body.body).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_question(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    qa::delete_question(&s.pool, &id, &user.id, user.is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vote_question(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    qa::vote_question(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unvote_question(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    qa::unvote_question(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn flag_question(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<FlagBody>,
) -> Result<StatusCode, AppError> {
    qa::flag_question(&s.pool, &id, &user.id, body.reason).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_answer(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateAnswer>,
) -> Result<(StatusCode, Json<Created>), AppError> {
    let id = qa::create_answer(&s.pool, &user.id, &id, body).await?;
    Ok((StatusCode::CREATED, Json(Created { id })))
}

pub async fn edit_answer(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<EditAnswer>,
) -> Result<StatusCode, AppError> {
    qa::edit_answer(&s.pool, &id, &user.id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_answer(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    qa::delete_answer(&s.pool, &id, &user.id, user.is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vote_answer(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoteBody>,
) -> Result<StatusCode, AppError> {
    if body.value != 1 && body.value != -1 {
        return Err(AppError::BadRequest("value must be 1 or -1".into()));
    }
    qa::vote_answer(&s.pool, &id, &user.id, body.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unvote_answer(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    qa::unvote_answer(&s.pool, &id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn flag_answer(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<FlagBody>,
) -> Result<StatusCode, AppError> {
    qa::flag_answer(&s.pool, &id, &user.id, body.reason).await?;
    Ok(StatusCode::NO_CONTENT)
}
