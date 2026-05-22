use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "offering_season", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Season {
    M,
    S,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Offering {
    pub id: Uuid,
    pub course_id: Uuid,
    pub season: Season,
    pub year: i16,
    pub created_at: DateTime<Utc>,
}
