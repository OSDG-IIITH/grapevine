pub mod courses;
pub mod offerings;
pub mod faculty;
pub mod labs;
pub mod reviews;
pub mod admin;

use axum::{extract::{FromRequestParts, Request}, middleware::{self, Next}, response::Response, routing::{get, post}, Router};
use axum::http::{HeaderValue, HeaderName, Method};
use tower_http::cors::CorsLayer;
use crate::{auth::{cas, session::VerifiedUser}, error::AppError, handlers, state::AppState};

/// Gate the protected sub-router: 401 if no session, 403 if logged in but unverified.
/// Reuses the `VerifiedUser` extractor logic.
async fn require_verified(req: Request, next: Next) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();
    // VerifiedUser only reads the session (no state needed for its extraction).
    VerifiedUser::from_request_parts(&mut parts, &()).await?;
    Ok(next.run(Request::from_parts(parts, body)).await)
}

pub fn app(state: AppState) -> Router {
    let origin: HeaderValue = state.cfg.frontend_url.parse().expect("invalid FRONTEND_URL");
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([HeaderName::from_static("content-type"), HeaderName::from_static("authorization")])
        .allow_credentials(true);

    // Open routes: no verification gate. The first four need no session at all;
    // /me, /auth/verify(/callback) and /auth/logout require a logged-in AuthUser.
    let open = Router::new()
        .route("/auth/login", get(cas::login))
        .route("/auth/callback", get(cas::callback))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login/local", post(handlers::auth::login_local))
        .route("/auth/recovery/:username", get(handlers::auth::recovery_info))
        .route("/auth/reset-password", post(handlers::auth::reset_password))
        .route("/auth/verify", get(cas::verify_login))
        .route("/auth/verify/callback", get(cas::verify_callback))
        .route("/auth/logout", post(cas::logout))
        .route("/me", get(handlers::auth::me).patch(handlers::auth::update_me));

    // Protected routes: all app content, gated behind require_verified.
    let protected = Router::new()
        .route("/me/reviews", get(handlers::auth::my_reviews))
        .route("/search", get(handlers::search::search))
        .merge(courses::router())
        .merge(offerings::router())
        .merge(faculty::router())
        .merge(labs::router())
        .merge(reviews::router())
        .merge(admin::router())
        .layer(middleware::from_fn(require_verified));

    open.merge(protected)
        .layer(cors)
        .with_state(state)
}
