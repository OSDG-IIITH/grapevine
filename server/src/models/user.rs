use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub cas_id: Option<String>,
    pub username: Option<String>,
    /// server-side credential material; never serialized to clients
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub display_name: String,
    pub is_admin: bool,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}
