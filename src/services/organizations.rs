//! Organisationen und Mitgliedschaften.

use crate::db::models::{new_id, now, Organization, Role};
use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;

pub fn validate_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::bad("Name der Organisation: 1–100 Zeichen."));
    }
    Ok(name.to_string())
}

/// Neue Organisation; wer sie anlegt, wird Besitzer.
pub async fn create(pool: &SqlitePool, owner_id: &str, name: &str) -> AppResult<Organization> {
    let name = validate_name(name)?;
    let id = new_id();
    let ts = now();
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO organizations (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&name)
        .bind(ts)
        .bind(ts)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role, created_at) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(owner_id)
        .bind(Role::Owner)
        .bind(ts)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Organization { id, name, created_at: ts, updated_at: ts, deleted_at: None })
}

/// Organisationen eines Benutzers (ohne gelöschte), nach Name.
pub async fn list_for_user(pool: &SqlitePool, user_id: &str) -> AppResult<Vec<(Organization, Role)>> {
    let rows: Vec<(String, String, i64, i64, Option<i64>, Role)> = sqlx::query_as(
        "SELECT o.id, o.name, o.created_at, o.updated_at, o.deleted_at, m.role
         FROM organizations o JOIN organization_members m ON m.organization_id = o.id
         WHERE m.user_id = ? AND o.deleted_at IS NULL
         ORDER BY o.name COLLATE NOCASE",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, created_at, updated_at, deleted_at, role)| (Organization { id, name, created_at, updated_at, deleted_at }, role))
        .collect())
}

/// Organisation samt Rolle des Benutzers – oder 404, wenn er kein Mitglied ist
/// (fremde Organisationen sollen nicht einmal ihre Existenz verraten).
pub async fn membership(pool: &SqlitePool, user_id: &str, org_id: &str) -> AppResult<(Organization, Role)> {
    let row: Option<(String, String, i64, i64, Option<i64>, Role)> = sqlx::query_as(
        "SELECT o.id, o.name, o.created_at, o.updated_at, o.deleted_at, m.role
         FROM organizations o JOIN organization_members m ON m.organization_id = o.id
         WHERE o.id = ? AND m.user_id = ? AND o.deleted_at IS NULL",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    let (id, name, created_at, updated_at, deleted_at, role) = row.ok_or(AppError::NotFound)?;
    Ok((Organization { id, name, created_at, updated_at, deleted_at }, role))
}
