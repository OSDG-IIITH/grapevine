use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub cas_base_url: String,
    pub app_url: String,
    pub frontend_url: String,
    #[allow(dead_code)]
    pub session_secret: String,
}

impl Config {
    pub fn load() -> Arc<Self> {
        dotenvy::dotenv().ok();
        Arc::new(Self {
            database_url: var("DATABASE_URL"),
            cas_base_url: var("CAS_BASE_URL"),
            app_url: var("APP_URL"),
            frontend_url: var("FRONTEND_URL"),
            session_secret: var("SESSION_SECRET"),
        })
    }
}

fn var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}
