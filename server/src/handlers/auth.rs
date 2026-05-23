use axum::{extract::State, Json};
use serde::Serialize;
use crate::{auth::session::AuthUser, error::AppError, state::AppState};

#[derive(Serialize)]
pub struct Me {
    pub id: String,
    pub display_name: String,
    pub is_admin: bool,
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
