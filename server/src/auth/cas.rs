use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tower_sessions::Session;
use ulid::Ulid;
use crate::{error::AppError, state::AppState};
use super::password;
use super::session::{AuthUser, AUTH_METHOD_KEY, IS_ADMIN_KEY, USER_ID_KEY, VERIFIED_KEY};
use super::validate::{is_allowed_email_domain, normalize_username, validate_password};

const CAS_PENDING_ID: &str = "cas_pending_id";
const CAS_PENDING_NAME: &str = "cas_pending_name";

/// HMAC-SHA256 of the lowercased email, keyed by `verify_email_secret`, hex-encoded.
/// Load-bearing for anonymity: a DB reader sees only this hash, never the address.
fn email_hmac(email: &str, secret: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(email.trim().to_lowercase().as_bytes());
    let bytes = mac.finalize().into_bytes();
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

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

    // Only allowed-domain emails may log in via CAS.
    if !is_allowed_email_domain(&cas_id, &s.cfg.allowed_email_domains) {
        return Ok(Redirect::to(&format!("{}/login?error=domain", s.cfg.frontend_url)));
    }

    let hash = email_hmac(&cas_id, &s.cfg.verify_email_secret);
    let already_used = sqlx::query_scalar!(
        "SELECT 1 AS one FROM verified_emails WHERE email_hash = $1",
        hash
    )
    .fetch_optional(&s.pool)
    .await?
    .is_some();
    if already_used {
        return Ok(Redirect::to(&format!("{}/login?error=email_has_local", s.cfg.frontend_url)));
    }

    let display_name = parse_cas_display_name(&xml).unwrap_or_else(|| name_from_cas_id(&cas_id));

    // New user: show confirmation page before creating the account.
    let is_new = sqlx::query_scalar!("SELECT 1 AS one FROM users WHERE cas_id = $1", cas_id)
        .fetch_optional(&s.pool)
        .await?
        .is_none();
    if is_new {
        session.insert(CAS_PENDING_ID, &cas_id).await.map_err(|_| AppError::Internal)?;
        session.insert(CAS_PENDING_NAME, &display_name).await.map_err(|_| AppError::Internal)?;
        return Ok(Redirect::to(&format!("{}/login?cas=pending", s.cfg.frontend_url)));
    }

    let is_moderator = s.cfg.moderator_emails.contains(&cas_id);
    let id = Ulid::new().to_string();
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, cas_id, display_name, verified)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (cas_id) DO UPDATE
            SET cas_id = EXCLUDED.cas_id,
                display_name = EXCLUDED.display_name,
                verified = true
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
    session.insert(VERIFIED_KEY, true).await.map_err(|_| AppError::Internal)?;
    session.insert(AUTH_METHOD_KEY, "cas").await.map_err(|_| AppError::Internal)?;

    Ok(Redirect::to(&s.cfg.frontend_url))
}

/// Confirm a new CAS account after the user has reviewed the privacy notice.
/// Reads the pending CAS identity stored by `callback`, creates the user row,
/// and establishes a full session. If no pending data exists, redirects to login.
pub async fn casconfirm(
    State(s): State<AppState>,
    session: Session,
) -> Result<Redirect, AppError> {
    let Some(cas_id): Option<String> = session.get(CAS_PENDING_ID).await.map_err(|_| AppError::Internal)? else {
        return Ok(Redirect::to(&format!("{}/login", s.cfg.frontend_url)));
    };
    let display_name: String = session
        .get(CAS_PENDING_NAME)
        .await
        .map_err(|_| AppError::Internal)?
        .unwrap_or_else(|| name_from_cas_id(&cas_id));

    session.remove::<String>(CAS_PENDING_ID).await.map_err(|_| AppError::Internal)?;
    session.remove::<String>(CAS_PENDING_NAME).await.map_err(|_| AppError::Internal)?;

    let is_moderator = s.cfg.moderator_emails.contains(&cas_id);
    let id = Ulid::new().to_string();
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, cas_id, display_name, verified)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (cas_id) DO UPDATE
            SET display_name = EXCLUDED.display_name,
                verified = true
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
    session.insert(VERIFIED_KEY, true).await.map_err(|_| AppError::Internal)?;
    session.insert(AUTH_METHOD_KEY, "cas").await.map_err(|_| AppError::Internal)?;

    Ok(Redirect::to(&s.cfg.frontend_url))
}

/// Start the anonymous-account verification flow: send the (logged-in, possibly
/// unverified) user to CAS with a service URL distinct from the login callback.
pub async fn verify_login(State(s): State<AppState>, _user: AuthUser) -> Redirect {
    let service = format!("{}/auth/verify/callback", s.cfg.app_url);
    Redirect::to(&format!("{}/login?service={}", s.cfg.cas_base_url, urlenc(&service)))
}

/// Finish verification: validate the CAS ticket, confirm an allowed-domain email
/// that doesn't already back a CAS account, record only its HMAC, flip the
/// session's own user row to verified, and never write the email onto that row.
pub async fn verify_callback(
    Query(q): Query<TicketQuery>,
    State(s): State<AppState>,
    session: Session,
    user: AuthUser,
) -> Result<Redirect, AppError> {
    let service = format!("{}/auth/verify/callback", s.cfg.app_url);
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

    if !is_allowed_email_domain(&cas_id, &s.cfg.allowed_email_domains) {
        return Ok(Redirect::to(&format!("{}/verify?error=domain", s.cfg.frontend_url)));
    }

    let hash = email_hmac(&cas_id, &s.cfg.verify_email_secret);

    // One transaction: re-check the CAS account, claim the email (PK dedup), and
    // flip the user row together, so concurrent verifies can't interleave and a
    // failed update never leaves an orphaned hash behind.
    let mut tx = s.pool.begin().await?;

    // Reject if a CAS account already exists for this email (use the plaintext we hold).
    let cas_account_exists = sqlx::query_scalar!(
        "SELECT id FROM users WHERE cas_id = $1",
        cas_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_some();
    if cas_account_exists {
        return Ok(Redirect::to(&format!("{}/verify?error=email_has_cas", s.cfg.frontend_url)));
    }

    // This flow is the only writer to verified_emails. The email_hash PK is the
    // real guard: a duplicate insert (23505) means the email already verified
    // another account — surface it as the friendly redirect, not a 500.
    match sqlx::query!("INSERT INTO verified_emails (email_hash) VALUES ($1)", hash)
        .execute(&mut *tx)
        .await
    {
        Ok(_) => {}
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => {
            return Ok(Redirect::to(&format!("{}/verify?error=email_used", s.cfg.frontend_url)));
        }
        Err(e) => return Err(e.into()),
    }

    // Flip only the session's own user row; never write cas_id/email onto it.
    sqlx::query!("UPDATE users SET verified = true WHERE id = $1", user.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    session.insert(VERIFIED_KEY, true).await.map_err(|_| AppError::Internal)?;
    // Session-fixation mitigation: rotate the session id (available in tower-sessions 0.13).
    session.cycle_id().await.map_err(|_| AppError::Internal)?;

    Ok(Redirect::to(&s.cfg.frontend_url))
}

/// Start the link flow: redirect a logged-in local user to CAS with a distinct service URL.
pub async fn link_login(State(s): State<AppState>, user: AuthUser) -> Result<Redirect, AppError> {
    let is_cas = sqlx::query_scalar!("SELECT cas_id FROM users WHERE id = $1", user.id)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::Unauthorized)?
        .is_some();
    if is_cas {
        return Err(AppError::BadRequest("already a CAS account".into()));
    }
    let service = format!("{}/auth/link/callback", s.cfg.app_url);
    Ok(Redirect::to(&format!("{}/login?service={}", s.cfg.cas_base_url, urlenc(&service))))
}

/// Finish the link flow: validate CAS ticket, check email availability, then transition
/// the local account to CAS-Only (sets cas_id, clears username/password_hash).
pub async fn link_callback(
    Query(q): Query<TicketQuery>,
    State(s): State<AppState>,
    session: Session,
    user: AuthUser,
) -> Result<Redirect, AppError> {
    let service = format!("{}/auth/link/callback", s.cfg.app_url);
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

    if !is_allowed_email_domain(&cas_id, &s.cfg.allowed_email_domains) {
        return Ok(Redirect::to(&format!("{}/login?error=domain", s.cfg.frontend_url)));
    }

    let hash = email_hmac(&cas_id, &s.cfg.verify_email_secret);
    let mut tx = s.pool.begin().await?;

    let email_taken = sqlx::query_scalar!(
        "SELECT 1 AS one FROM users WHERE cas_id = $1 AND id != $2",
        cas_id, user.id
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_some();
    if email_taken {
        return Ok(Redirect::to(&format!("{}/login?error=email_taken", s.cfg.frontend_url)));
    }

    let email_used = sqlx::query_scalar!(
        "SELECT 1 AS one FROM verified_emails WHERE email_hash = $1",
        hash
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_some();
    if email_used && !user.verified {
        return Ok(Redirect::to(&format!("{}/login?error=email_has_local", s.cfg.frontend_url)));
    }

    sqlx::query!(
        "DELETE FROM verified_emails WHERE email_hash = $1",
        hash
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE users SET cas_id = $1, username = NULL, password_hash = NULL, verified = true WHERE id = $2",
        cas_id, user.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    session.insert(AUTH_METHOD_KEY, "cas").await.map_err(|_| AppError::Internal)?;
    session.insert(VERIFIED_KEY, true).await.map_err(|_| AppError::Internal)?;

    Ok(Redirect::to(&s.cfg.frontend_url))
}

#[derive(Deserialize)]
pub struct UnlinkBody {
    pub username: String,
    pub password: String,
}

/// Transition a CAS-Only account to Local-Only. The user must provide a new username
/// and password. The CAS email hash is permanently stored in verified_emails.
pub async fn unlink(
    State(s): State<AppState>,
    session: Session,
    user: AuthUser,
    Json(body): Json<UnlinkBody>,
) -> Result<StatusCode, AppError> {
    validate_password(&body.password).map_err(AppError::BadRequest)?;
    let username = normalize_username(&body.username).map_err(AppError::BadRequest)?;

    let cas_id = sqlx::query_scalar!("SELECT cas_id FROM users WHERE id = $1", user.id)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::Unauthorized)?
        .ok_or_else(|| AppError::BadRequest("not a CAS account".into()))?;

    let new_hash = password::hash(&body.password).map_err(|_| AppError::Internal)?;
    let email_hash = email_hmac(&cas_id, &s.cfg.verify_email_secret);

    let mut tx = s.pool.begin().await?;

    sqlx::query!(
        "INSERT INTO verified_emails (email_hash) VALUES ($1) ON CONFLICT DO NOTHING",
        email_hash
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE users SET cas_id = NULL, username = $1, password_hash = $2 WHERE id = $3",
        username, new_hash, user.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    session.insert(AUTH_METHOD_KEY, "local").await.map_err(|_| AppError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn logout(State(s): State<AppState>, session: Session) -> Result<Json<LogoutResponse>, AppError> {
    let auth_method: Option<String> = session.get(AUTH_METHOD_KEY).await.unwrap_or(None);
    session.flush().await.map_err(|_| AppError::Internal)?;
    let redirect_url = if auth_method.as_deref() == Some("local") {
        // Local sessions have no CAS ticket to clear; bounce to the frontend login page.
        format!("{}/login", s.cfg.frontend_url)
    } else {
        format!("{}/logout?service={}", s.cfg.cas_base_url, urlenc(&s.cfg.frontend_url))
    };
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
