use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CourseReview {
    pub id: Uuid,
    pub user_id: Uuid,
    pub offering_id: Uuid,
    pub anonymous: bool,
    pub difficulty: i16,
    pub teaching: i16,
    pub grading: i16,
    pub content: i16,
    pub workload: i16,
    pub body: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdvisorReview {
    pub id: Uuid,
    pub user_id: Uuid,
    pub faculty_id: Uuid,
    pub anonymous: bool,
    pub research: i16,
    pub availability: i16,
    pub mentorship: i16,
    pub support: i16,
    pub workload: i16,
    pub body: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
