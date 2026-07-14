use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportTarget {
    Course,
    Faculty,
    Lab,
    Offering,
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub target_type: ReportTarget,
    pub target_id: String,
    pub reason: String,
    pub faculty_ids: Option<Vec<String>>,
}

#[derive(Debug, FromRow)]
struct ReportRow {
    id: String,
    target_type: String,
    target_id: String,
    course_code: Option<String>,
    target_label: String,
    reason: String,
    created_at: DateTime<Utc>,
    reporter_id: String,
    reporter_name: String,
    has_faculty_suggestion: bool,
}

#[derive(Debug, FromRow)]
struct ReportFacultyRow {
    report_id: String,
    id: String,
    name: String,
    suggested: bool,
}

#[derive(Debug, Serialize)]
pub struct ReportFaculty {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub course_code: Option<String>,
    pub target_label: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub reporter_id: String,
    pub reporter_name: String,
    pub has_faculty_suggestion: bool,
    pub current_faculty: Vec<ReportFaculty>,
    pub suggested_faculty: Vec<ReportFaculty>,
}

pub async fn create(pool: &PgPool, user_id: &str, body: CreateReport) -> Result<(), AppError> {
    let CreateReport {
        target_type,
        target_id,
        reason,
        faculty_ids,
    } = body;
    let reason = reason.trim();
    let mut tx = pool.begin().await?;

    match target_type {
        ReportTarget::Offering => {
            validate_offering_reason(reason)?;

            let exists = sqlx::query_scalar::<_, String>(
                "SELECT o.id FROM offerings o JOIN courses c ON c.id = o.course_id WHERE o.id = $1 AND o.deleted_at IS NULL AND o.approved = true AND c.deleted_at IS NULL FOR SHARE OF o, c",
            )
            .bind(&target_id)
            .fetch_optional(&mut *tx)
            .await?;
            if exists.is_none() {
                return Err(AppError::NotFound);
            }

            let current_ids = sqlx::query_scalar::<_, String>(
                "SELECT faculty_id FROM offering_faculty WHERE offering_id = $1",
            )
            .bind(&target_id)
            .fetch_all(&mut *tx)
            .await?;
            let faculty_ids = faculty_ids.map(deduplicate_ids);
            let has_faculty_suggestion = faculty_ids.is_some();

            if let Some(suggested_ids) = &faculty_ids {
                let active_ids = sqlx::query_scalar::<_, String>(
                    "SELECT id FROM faculty WHERE id = ANY($1) AND deleted_at IS NULL",
                )
                .bind(suggested_ids)
                .fetch_all(&mut *tx)
                .await?;
                if active_ids.len() != suggested_ids.len() {
                    return Err(AppError::BadRequest(
                        "all suggested faculty must exist and be active".into(),
                    ));
                }
            }

            if reason.is_empty()
                && !faculty_ids
                    .as_ref()
                    .is_some_and(|ids| faculty_sets_differ(&current_ids, ids))
            {
                return Err(AppError::BadRequest(
                    "an empty reason requires a corrected faculty set".into(),
                ));
            }

            let report_id = sqlx::query_scalar::<_, String>(
                "INSERT INTO reports (user_id, offering_id, reason, has_faculty_suggestion) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind(user_id)
            .bind(&target_id)
            .bind(reason)
            .bind(has_faculty_suggestion)
            .fetch_one(&mut *tx)
            .await?;

            if !current_ids.is_empty() {
                sqlx::query(
                    "INSERT INTO report_faculty (report_id, faculty_id, suggested) SELECT $1, unnest($2::text[]), false",
                )
                .bind(&report_id)
                .bind(current_ids)
                .execute(&mut *tx)
                .await?;
            }

            if let Some(faculty_ids) = faculty_ids.filter(|ids| !ids.is_empty()) {
                sqlx::query(
                    "INSERT INTO report_faculty (report_id, faculty_id, suggested) SELECT $1, unnest($2::text[]), true",
                )
                .bind(report_id)
                .bind(faculty_ids)
                .execute(&mut *tx)
                .await?;
            }
        }
        target_type => {
            validate_standard_reason(reason)?;
            if faculty_ids.is_some() {
                return Err(AppError::BadRequest(
                    "faculty_ids is only valid for offering reports".into(),
                ));
            }

            let (exists_sql, insert_sql) = match target_type {
                ReportTarget::Course => (
                    "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1 AND deleted_at IS NULL)",
                    "INSERT INTO reports (user_id, course_id, reason) VALUES ($1, $2, $3)",
                ),
                ReportTarget::Faculty => (
                    "SELECT EXISTS(SELECT 1 FROM faculty WHERE id = $1 AND deleted_at IS NULL)",
                    "INSERT INTO reports (user_id, faculty_id, reason) VALUES ($1, $2, $3)",
                ),
                ReportTarget::Lab => (
                    "SELECT EXISTS(SELECT 1 FROM labs WHERE id = $1 AND deleted_at IS NULL)",
                    "INSERT INTO reports (user_id, lab_id, reason) VALUES ($1, $2, $3)",
                ),
                ReportTarget::Offering => unreachable!(),
            };
            let exists = sqlx::query_scalar::<_, bool>(exists_sql)
                .bind(&target_id)
                .fetch_one(&mut *tx)
                .await?;
            if !exists {
                return Err(AppError::NotFound);
            }
            sqlx::query(insert_sql)
                .bind(user_id)
                .bind(&target_id)
                .bind(reason)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

fn validate_standard_reason(reason: &str) -> Result<(), AppError> {
    if !(3..=1000).contains(&reason.chars().count()) {
        return Err(AppError::BadRequest(
            "reason must be between 3 and 1000 characters".into(),
        ));
    }
    Ok(())
}

fn validate_offering_reason(reason: &str) -> Result<(), AppError> {
    let length = reason.chars().count();
    if length > 1000 || (1..3).contains(&length) {
        return Err(AppError::BadRequest(
            "reason must be empty or between 3 and 1000 characters".into(),
        ));
    }
    Ok(())
}

fn deduplicate_ids(ids: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    ids.into_iter()
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

fn faculty_sets_differ(current: &[String], suggested: &[String]) -> bool {
    current.iter().collect::<HashSet<_>>() != suggested.iter().collect::<HashSet<_>>()
}

pub async fn list(pool: &PgPool) -> Result<Vec<ReportResponse>, AppError> {
    let reports = sqlx::query_as::<_, ReportRow>(
        r#"SELECT r.id,
                  CASE
                    WHEN r.course_id IS NOT NULL THEN 'course'
                    WHEN r.faculty_id IS NOT NULL THEN 'faculty'
                    WHEN r.lab_id IS NOT NULL THEN 'lab'
                    ELSE 'offering'
                   END AS target_type,
                   COALESCE(r.course_id, r.faculty_id, r.lab_id, r.offering_id) AS target_id,
                   COALESCE(c.code, oc.code) AS course_code,
                   CASE
                    WHEN r.course_id IS NOT NULL THEN c.code || ' · ' || c.name
                    WHEN r.faculty_id IS NOT NULL THEN f.name
                    WHEN r.lab_id IS NOT NULL THEN l.shortname || ' · ' || l.name
                    ELSE oc.code || ' · ' || oc.name || ' · ' || CASE o.season WHEN 'M' THEN 'Monsoon ' ELSE 'Spring ' END || o.year
                  END AS target_label,
                  r.reason, r.created_at, r.has_faculty_suggestion,
                  u.id AS reporter_id, u.display_name AS reporter_name
           FROM reports r
           JOIN users u ON u.id = r.user_id
           LEFT JOIN courses c ON c.id = r.course_id
           LEFT JOIN faculty f ON f.id = r.faculty_id
           LEFT JOIN labs l ON l.id = r.lab_id
           LEFT JOIN offerings o ON o.id = r.offering_id
           LEFT JOIN courses oc ON oc.id = o.course_id
           ORDER BY r.created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;

    let report_ids: Vec<_> = reports.iter().map(|report| report.id.clone()).collect();
    let faculty = sqlx::query_as::<_, ReportFacultyRow>(
        r#"SELECT r.id AS report_id, f.id, f.name, false AS suggested
           FROM reports r
           JOIN report_faculty rf ON rf.report_id = r.id
           JOIN faculty f ON f.id = rf.faculty_id
           WHERE r.id = ANY($1) AND rf.suggested = false
           UNION ALL
           SELECT r.id AS report_id, f.id, f.name, true AS suggested
           FROM reports r
           JOIN report_faculty rf ON rf.report_id = r.id
           JOIN faculty f ON f.id = rf.faculty_id
           WHERE r.id = ANY($1) AND rf.suggested = true
           ORDER BY report_id, suggested, name, id"#,
    )
    .bind(&report_ids)
    .fetch_all(pool)
    .await?;

    Ok(attach_faculty(reports, faculty))
}

pub async fn dismiss(pool: &PgPool, id: &str, admin_id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let report = sqlx::query_as::<_, ReportRow>(
        r#"SELECT r.id,
                  CASE
                    WHEN r.course_id IS NOT NULL THEN 'course'
                    WHEN r.faculty_id IS NOT NULL THEN 'faculty'
                    WHEN r.lab_id IS NOT NULL THEN 'lab'
                    ELSE 'offering'
                   END AS target_type,
                   COALESCE(r.course_id, r.faculty_id, r.lab_id, r.offering_id) AS target_id,
                   COALESCE(c.code, oc.code) AS course_code,
                   CASE
                    WHEN r.course_id IS NOT NULL THEN c.code || ' · ' || c.name
                    WHEN r.faculty_id IS NOT NULL THEN f.name
                    WHEN r.lab_id IS NOT NULL THEN l.shortname || ' · ' || l.name
                    ELSE oc.code || ' · ' || oc.name || ' · ' || CASE o.season WHEN 'M' THEN 'Monsoon ' ELSE 'Spring ' END || o.year
                  END AS target_label,
                  r.reason, r.created_at, r.has_faculty_suggestion,
                  u.id AS reporter_id, u.display_name AS reporter_name
           FROM reports r
           JOIN users u ON u.id = r.user_id
           LEFT JOIN courses c ON c.id = r.course_id
           LEFT JOIN faculty f ON f.id = r.faculty_id
           LEFT JOIN labs l ON l.id = r.lab_id
           LEFT JOIN offerings o ON o.id = r.offering_id
           LEFT JOIN courses oc ON oc.id = o.course_id
           WHERE r.id = $1
           FOR UPDATE OF r"#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    let faculty = sqlx::query_as::<_, ReportFacultyRow>(
        r#"SELECT r.id AS report_id, f.id, f.name, false AS suggested
           FROM reports r
           JOIN report_faculty rf ON rf.report_id = r.id
           JOIN faculty f ON f.id = rf.faculty_id
           WHERE r.id = $1 AND rf.suggested = false
           UNION ALL
           SELECT r.id AS report_id, f.id, f.name, true AS suggested
           FROM reports r
           JOIN report_faculty rf ON rf.report_id = r.id
           JOIN faculty f ON f.id = rf.faculty_id
           WHERE r.id = $1 AND rf.suggested = true
           ORDER BY suggested, name, id"#,
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    let mut reports = attach_faculty(vec![report], faculty);
    let report = reports.pop().expect("the locked report exists");

    sqlx::query("DELETE FROM reports WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    let previous = json!({
        "target_type": report.target_type,
        "target_id": report.target_id,
        "target_label": report.target_label,
        "reason": report.reason,
        "reporter_id": report.reporter_id,
        "reporter_name": report.reporter_name,
        "current_faculty": report.current_faculty,
        "suggested_faculty": report.suggested_faculty,
        "has_faculty_suggestion": report.has_faculty_suggestion,
    });
    sqlx::query(
        "INSERT INTO audit_logs (id, admin_id, action, target_type, target_id, previous_state) VALUES ($1, $2, 'DISMISS_REPORT', 'report', $3, $4)",
    )
    .bind(Ulid::new().to_string())
    .bind(admin_id)
    .bind(id)
    .bind(previous)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn approve_faculty_suggestion(
    pool: &PgPool,
    id: &str,
    admin_id: &str,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let report = sqlx::query!(
        r#"SELECT r.offering_id, r.has_faculty_suggestion
           FROM reports r
           JOIN offerings o ON o.id = r.offering_id
           WHERE r.id = $1 AND o.deleted_at IS NULL
           FOR UPDATE OF r, o"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    let offering_id = report.offering_id.ok_or(AppError::NotFound)?;
    if !report.has_faculty_suggestion {
        return Err(AppError::BadRequest(
            "report does not include an instructor change".into(),
        ));
    }

    let suggested_faculty = sqlx::query_scalar!(
        "SELECT faculty_id FROM report_faculty WHERE report_id = $1 AND suggested = true ORDER BY faculty_id",
        id
    )
    .fetch_all(&mut *tx)
    .await?;

    let active_faculty_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM faculty WHERE id = ANY($1) AND deleted_at IS NULL",
        &suggested_faculty
    )
    .fetch_one(&mut *tx)
    .await?;
    if active_faculty_count != Some(suggested_faculty.len() as i64) {
        return Err(AppError::BadRequest(
            "all suggested faculty must be active".into(),
        ));
    }

    let previous_faculty = sqlx::query_scalar!(
        "SELECT f.slug FROM offering_faculty ofac JOIN faculty f ON f.id = ofac.faculty_id WHERE ofac.offering_id = $1 ORDER BY f.slug",
        offering_id
    )
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM offering_faculty WHERE offering_id = $1", offering_id)
        .execute(&mut *tx)
        .await?;
    if !suggested_faculty.is_empty() {
        sqlx::query!(
            "INSERT INTO offering_faculty (offering_id, faculty_id) SELECT $1, unnest($2::text[])",
            offering_id,
            &suggested_faculty
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query!("DELETE FROM reports WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    let previous = json!({
        "report_id": id,
        "faculty": previous_faculty,
        "suggested_faculty_ids": suggested_faculty,
    });
    sqlx::query(
        "INSERT INTO audit_logs (id, admin_id, action, target_type, target_id, previous_state) VALUES ($1, $2, 'APPROVE_REPORT', 'offering', $3, $4)",
    )
    .bind(Ulid::new().to_string())
    .bind(admin_id)
    .bind(offering_id)
    .bind(previous)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

fn attach_faculty(reports: Vec<ReportRow>, faculty: Vec<ReportFacultyRow>) -> Vec<ReportResponse> {
    let mut by_report: HashMap<String, (Vec<ReportFaculty>, Vec<ReportFaculty>)> = HashMap::new();
    for row in faculty {
        let lists = by_report.entry(row.report_id).or_default();
        let faculty = ReportFaculty {
            id: row.id,
            name: row.name,
        };
        if row.suggested {
            lists.1.push(faculty);
        } else {
            lists.0.push(faculty);
        }
    }

    reports
        .into_iter()
        .map(|report| {
            let (current_faculty, suggested_faculty) =
                by_report.remove(&report.id).unwrap_or_default();
            ReportResponse {
                id: report.id,
                target_type: report.target_type,
                target_id: report.target_id,
                course_code: report.course_code,
                target_label: report.target_label,
                reason: report.reason,
                created_at: report.created_at,
                reporter_id: report.reporter_id,
                reporter_name: report.reporter_name,
                has_faculty_suggestion: report.has_faculty_suggestion,
                current_faculty,
                suggested_faculty,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_faculty_ids_without_reordering() {
        assert_eq!(
            deduplicate_ids(vec!["b".into(), "a".into(), "b".into()]),
            vec!["b", "a"]
        );
    }

    #[test]
    fn compares_faculty_as_sets() {
        assert!(!faculty_sets_differ(
            &["a".into(), "b".into()],
            &["b".into(), "a".into(), "a".into()]
        ));
        assert!(faculty_sets_differ(&["a".into()], &[]));
        assert!(faculty_sets_differ(&[], &["a".into()]));
    }

    #[test]
    fn validates_target_specific_reason_lengths() {
        assert!(validate_standard_reason("abc").is_ok());
        assert!(validate_standard_reason("").is_err());
        assert!(validate_offering_reason("").is_ok());
        assert!(validate_offering_reason("ab").is_err());
        assert!(validate_offering_reason("abc").is_ok());
        assert!(validate_offering_reason(&"a".repeat(1001)).is_err());
    }
}
