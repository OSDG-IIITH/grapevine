mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod routes;
mod state;

use tower_sessions::SessionManagerLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::load();
    let pool = db::connect(&cfg.database_url).await;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.expect("session store migration failed");

    // Lax (not the Strict default) so the cookie rides the top-level GET
    // redirect back from CAS to /auth/verify/callback; Strict would drop it
    // there and the verify flow couldn't identify the logged-in local user.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);

    let state = AppState { pool, cfg };
    let app = routes::app(state).layer(session_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
