use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lab {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}
