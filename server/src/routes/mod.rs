pub mod courses;
pub mod offerings;
pub mod faculty;
pub mod labs;
pub mod reviews;
pub mod admin;

use axum::{routing::{get, post}, Router};
use axum::http::{HeaderValue, HeaderName, Method};
use tower_http::cors::CorsLayer;
use crate::{auth::cas, state::AppState};

pub fn app(state: AppState) -> Router {
    let origin: HeaderValue = state.cfg.frontend_url.parse().expect("invalid FRONTEND_URL");
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([HeaderName::from_static("content-type"), HeaderName::from_static("authorization")])
        .allow_credentials(true);

    Router::new()
        .route("/auth/login", get(cas::login))
        .route("/auth/callback", get(cas::callback))
        .route("/auth/logout", post(cas::logout))
        .merge(courses::router())
        .merge(offerings::router())
        .merge(faculty::router())
        .merge(labs::router())
        .merge(reviews::router())
        .merge(admin::router())
        .layer(cors)
        .with_state(state)
}
