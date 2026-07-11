use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::{error::AppError, state::AppState};

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SearchResult {
    Course { name: String, code: String },
    Faculty { name: String, slug: String },
    Lab { name: String, shortname: String },
}

#[derive(Debug)]
struct Scored<T> {
    score: f32,
    item: T,
}

pub async fn search(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let q = match q.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(q) => q.to_owned(),
        None => return Ok(Json(vec![])),
    };

    let (courses, faculty, labs) = tokio::join!(
        search_courses(&s.pool, &q),
        search_faculty(&s.pool, &q),
        search_labs(&s.pool, &q),
    );

    let mut results: Vec<Scored<SearchResult>> = vec![];

    for (score, name, code) in courses? {
        results.push(Scored { score, item: SearchResult::Course { name, code } });
    }
    for (score, name, slug) in faculty? {
        results.push(Scored { score, item: SearchResult::Faculty { name, slug } });
    }
    for (score, name, shortname) in labs? {
        results.push(Scored { score, item: SearchResult::Lab { name, shortname } });
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(10);

    Ok(Json(results.into_iter().map(|s| s.item).collect()))
}

async fn search_courses(pool: &PgPool, q: &str) -> Result<Vec<(f32, String, String)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT GREATEST(similarity(name, $1), similarity(code, $1),
                COALESCE((SELECT MAX(similarity(s, $1)) FROM unnest(shortnames) s), 0))::real AS score,
                name, code
         FROM courses
         WHERE (similarity(name, $1) > 0.1 OR similarity(code, $1) > 0.1
             OR EXISTS (SELECT 1 FROM unnest(shortnames) s WHERE similarity(s, $1) > 0.1))
           AND deleted_at IS NULL
         ORDER BY score DESC LIMIT 10"
    )
    .bind(q)
    .fetch_all(pool)
    .await?)
}

async fn search_faculty(pool: &PgPool, q: &str) -> Result<Vec<(f32, String, String)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT similarity(name, $1)::real AS score, name, slug
         FROM faculty
         WHERE similarity(name, $1) > 0.1 AND deleted_at IS NULL
         ORDER BY score DESC LIMIT 10"
    )
    .bind(q)
    .fetch_all(pool)
    .await?)
}

async fn search_labs(pool: &PgPool, q: &str) -> Result<Vec<(f32, String, String)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT GREATEST(similarity(name, $1), similarity(shortname, $1))::real AS score, name, shortname
         FROM labs
         WHERE (similarity(name, $1) > 0.1 OR similarity(shortname, $1) > 0.1) AND deleted_at IS NULL
         ORDER BY score DESC LIMIT 10"
    )
    .bind(q)
    .fetch_all(pool)
    .await?)
}
