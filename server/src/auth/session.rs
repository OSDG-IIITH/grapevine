use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;
use tower_sessions::Session;
use crate::error::AppError;

pub const USER_ID_KEY: &str = "user_id";
pub const IS_ADMIN_KEY: &str = "is_admin";
pub const VERIFIED_KEY: &str = "verified";
#[allow(dead_code)]
pub const AUTH_METHOD_KEY: &str = "auth_method";

pub struct AuthUser {
    pub id: String,
    pub is_admin: bool,
    pub verified: bool,
}

/// extractor for the gated app: requires a logged-in AND verified session.
/// 401 if no session/user, 403 if logged in but not verified.
#[allow(dead_code)]
pub struct VerifiedUser {
    pub id: String,
    pub is_admin: bool,
}

/// extractor for public endpoints that want user_vote when a session exists
pub struct MaybeAuth(pub String);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let id: String = session
            .get(USER_ID_KEY)
            .await
            .map_err(|_| AppError::Unauthorized)?
            .ok_or(AppError::Unauthorized)?;
        let is_admin: bool = session
            .get(IS_ADMIN_KEY)
            .await
            .unwrap_or(None)
            .unwrap_or(false);
        let verified: bool = session
            .get(VERIFIED_KEY)
            .await
            .unwrap_or(None)
            .unwrap_or(false);
        Ok(AuthUser { id, is_admin, verified })
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for VerifiedUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser { id, is_admin, verified } = AuthUser::from_request_parts(parts, state).await?;
        if !verified {
            return Err(AppError::Forbidden);
        }
        Ok(VerifiedUser { id, is_admin })
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for MaybeAuth {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Infallible> {
        let id = async {
            let session = Session::from_request_parts(parts, state).await.ok()?;
            session.get::<String>(USER_ID_KEY).await.ok()?
        }
        .await
        .unwrap_or_default();
        Ok(MaybeAuth(id))
    }
}
