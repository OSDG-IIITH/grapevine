use axum::{
    extract::{Query, State},
    response::Redirect,
};
use serde::Deserialize;
use tower_sessions::Session;
use ulid::Ulid;
use crate::{error::AppError, state::AppState};
use super::session::{IS_ADMIN_KEY, USER_ID_KEY};

#[derive(Deserialize)]
pub struct TicketQuery {
    ticket: String,
}

pub async fn login(State(s): State<AppState>) -> Redirect {
    let service = format!("{}/auth/callback", s.cfg.app_url);
    Redirect::to(&format!("{}/login?service={}", s.cfg.cas_base_url, urlenc(&service)))
}

pub async fn callback(
    Query(q): Query<TicketQuery>,
    State(s): State<AppState>,
    session: Session,
) -> Result<Redirect, AppError> {
    let service = format!("{}/auth/callback", s.cfg.app_url);
    let validate_url = format!(
        "{}/serviceValidate?ticket={}&service={}",
        s.cfg.cas_base_url, q.ticket, urlenc(&service)
    );

    let xml = reqwest::get(&validate_url)
        .await
        .map_err(|_| AppError::Internal)?
        .text()
        .await
        .map_err(|_| AppError::Internal)?;

    let cas_id = parse_cas_user(&xml).ok_or(AppError::Unauthorized)?;
    let is_moderator = s.cfg.moderator_emails.contains(&cas_id);

    let id = Ulid::new().to_string();
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, cas_id, display_name)
        VALUES ($1, $2, $2)
        ON CONFLICT (cas_id) DO UPDATE SET cas_id = EXCLUDED.cas_id
        RETURNING id, is_admin
        "#,
        id, cas_id
    )
    .fetch_one(&s.pool)
    .await?;

    let is_admin = if is_moderator {
        sqlx::query_scalar!(
            "UPDATE users SET is_admin = true WHERE id = $1 RETURNING is_admin",
            row.id
        )
        .fetch_one(&s.pool)
        .await?
    } else {
        row.is_admin
    };

    session.insert(USER_ID_KEY, row.id).await.map_err(|_| AppError::Internal)?;
    session.insert(IS_ADMIN_KEY, is_admin).await.map_err(|_| AppError::Internal)?;

    Ok(Redirect::to(&s.cfg.frontend_url))
}

pub async fn logout(State(s): State<AppState>, session: Session) -> Result<Redirect, AppError> {
    session.flush().await.map_err(|_| AppError::Internal)?;
    Ok(Redirect::to(&format!("{}/logout", s.cfg.cas_base_url)))
}

fn parse_cas_user(xml: &str) -> Option<String> {
    let start = xml.find("<cas:user>")?;
    let end = xml.find("</cas:user>")?;
    Some(xml[start + 10..end].trim().to_string())
}

fn urlenc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ':' => out.push_str("%3A"),
            '/' => out.push_str("%2F"),
            '?' => out.push_str("%3F"),
            '=' => out.push_str("%3D"),
            '&' => out.push_str("%26"),
            _ => out.push(c),
        }
    }
    out
}
