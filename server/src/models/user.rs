use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub cas_id: String,
    pub display_name: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}
