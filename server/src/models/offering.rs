use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "offering_season", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Season {
    M,
    S,
}
