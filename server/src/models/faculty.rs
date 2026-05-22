use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Faculty {
    pub id: Uuid,
    pub name: String,
    pub bio: Option<String>,
    pub lab_id: Option<Uuid>,
}
