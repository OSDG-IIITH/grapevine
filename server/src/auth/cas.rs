use axum::{
    extract::{Query, State},
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use ulid::Ulid;
use crate::{error::AppError, state::AppState};
use super::session::{IS_ADMIN_KEY, USER_ID_KEY};

#[derive(Deserialize)]
pub struct TicketQuery {
    ticket: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub redirect_url: String,
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
    let display_name = parse_cas_display_name(&xml).unwrap_or_else(|| name_from_cas_id(&cas_id));
    let is_moderator = s.cfg.moderator_emails.contains(&cas_id);

    let id = Ulid::new().to_string();
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, cas_id, display_name)
        VALUES ($1, $2, $3)
        ON CONFLICT (cas_id) DO UPDATE
            SET cas_id = EXCLUDED.cas_id,
                display_name = EXCLUDED.display_name
        RETURNING id, is_admin
        "#,
        id, cas_id, display_name
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

pub async fn logout(State(s): State<AppState>, session: Session) -> Result<Json<LogoutResponse>, AppError> {
    session.flush().await.map_err(|_| AppError::Internal)?;
    let redirect_url = format!("{}/logout?service={}", s.cfg.cas_base_url, urlenc(&s.cfg.frontend_url));
    Ok(Json(LogoutResponse { redirect_url }))
}

fn parse_cas_user(xml: &str) -> Option<String> {
    let start = xml.find("<cas:user>")?;
    let end = xml.find("</cas:user>")?;
    Some(xml[start + 10..end].trim().to_string())
}

fn parse_cas_display_name(xml: &str) -> Option<String> {
    parse_cas_tag(xml, "displayName")
        .or_else(|| parse_cas_tag(xml, "displayname"))
        .or_else(|| parse_cas_tag(xml, "cn"))
        .or_else(|| parse_cas_tag(xml, "fullName"))
        .or_else(|| {
            let given = parse_cas_tag(xml, "givenName").or_else(|| parse_cas_tag(xml, "firstname"));
            let family = parse_cas_tag(xml, "sn").or_else(|| parse_cas_tag(xml, "lastname"));
            match (given, family) {
                (Some(g), Some(f)) => Some(format!("{} {}", g, f)),
                (Some(g), None) => Some(g),
                (None, Some(f)) => Some(f),
                _ => None,
            }
        })
}

fn parse_cas_tag(xml: &str, tag: &str) -> Option<String> {
    parse_tag(xml, &format!("cas:{}", tag)).or_else(|| parse_tag(xml, tag))
}

fn parse_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let start = xml.find(&open)?;
    let after_open = xml[start..].find('>')? + start;
    let close = format!("</{}>", tag);
    let end = xml[after_open + 1..].find(&close)? + after_open + 1;
    Some(xml[after_open + 1..end].trim().to_string())
}

fn name_from_cas_id(cas_id: &str) -> String {
    let local = cas_id.split('@').next().unwrap_or(cas_id);
    let mut parts = Vec::new();
    for part in local.split(|c| c == '.' || c == '_' || c == '-') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        parts.push(titlecase_word(part));
    }
    if parts.is_empty() {
        cas_id.to_string()
    } else {
        parts.join(" ")
    }
}

fn titlecase_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else { return String::new(); };
    let mut out = String::new();
    out.extend(first.to_uppercase());
    out.push_str(&chars.as_str().to_lowercase());
    out
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
