use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub cas_base_url: String,
    pub app_url: String,
    pub frontend_url: String,
    #[allow(dead_code)]
    pub session_secret: String,
    pub moderator_emails: Vec<String>,
    pub allowed_email_domains: Vec<String>,
    pub verify_email_secret: String,
}

const DEFAULT_ALLOWED_EMAIL_DOMAINS: &str = "students.iiit.ac.in,research.iiit.ac.in";

impl Config {
    pub fn load() -> Arc<Self> {
        dotenvy::dotenv().ok();
        let moderator_emails = std::env::var("MODERATOR_EMAILS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let allowed_email_domains = std::env::var("ALLOWED_EMAIL_DOMAINS")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ALLOWED_EMAIL_DOMAINS.to_string())
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        Arc::new(Self {
            database_url: var("DATABASE_URL"),
            cas_base_url: var("CAS_BASE_URL"),
            app_url: var("APP_URL"),
            frontend_url: var("FRONTEND_URL"),
            session_secret: var("SESSION_SECRET"),
            moderator_emails,
            allowed_email_domains,
            verify_email_secret: var("VERIFY_EMAIL_SECRET"),
        })
    }
}

fn var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}
