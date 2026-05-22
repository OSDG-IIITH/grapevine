use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "course_type", rename_all = "lowercase")]
pub enum CourseType {
    Core,
    Open,
    Breadth,
    Stream,
    Bouquet,
    Hs,
    Sci,
    Math,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Course {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub kind: CourseType,
}
