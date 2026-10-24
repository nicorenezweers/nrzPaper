//! Tags einer Organisation und ihre Zuordnung zu Dokumenten.

use crate::db::models::{new_id, now, Tag, WebhookEvent};
use crate::error::{AppError, AppResult};
use crate::services::documents;
use crate::services::webhooks;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashMap;

/// Graustufen und dezente Farben für neue Tags (die Oberfläche bleibt grau, Tags dürfen Farbe tragen).
pub const PALETTE: &[&str] = &[
    "#71717a", "#a1a1aa", "#e5484d", "#f76b15", "#ffc53d", "#46a758", "#12a594", "#0090ff",
    "#6e56cf", "#d6409f",
];

pub fn validate(name: &str, color: &str) -> AppResult<(String, String)> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(AppError::bad("Tag-Name: 1–64 Zeichen."));
    }
    let color = color.trim().to_lowercase();
    let ok = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    Ok((
        name.to_string(),
        if ok { color } else { PALETTE[0].to_string() },
    ))
}

/// Tags mit Anzahl der (nicht gelöschten) Dokumente, nach Name.
pub async fn list(pool: &SqlitePool, org: &str) -> AppResult<Vec<(Tag, i64)>> {
    let tags: Vec<Tag> =
        sqlx::query_as("SELECT * FROM tags WHERE organization_id = ? ORDER BY name COLLATE NOCASE")
            .bind(org)
            .fetch_all(pool)
            .await?;
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT dt.tag_id, count(*) FROM document_tags dt JOIN documents d ON d.id = dt.document_id
         WHERE d.organization_id = ? AND d.deleted_at IS NULL GROUP BY dt.tag_id",
    )
    .bind(org)
    .fetch_all(pool)
    .await?;
    let counts: HashMap<String, i64> = counts.into_iter().collect();
    Ok(tags
        .into_iter()
        .map(|t| {
            let n = counts.get(&t.id).copied().unwrap_or(0);
            (t, n)
        })
        .collect())
}

pub async fn get(pool: &SqlitePool, org: &str, id: &str) -> AppResult<Tag> {
    sqlx::query_as("SELECT * FROM tags WHERE id = ? AND organization_id = ?")
        .bind(id)
        .bind(org)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn by_name(pool: &SqlitePool, org: &str, name: &str) -> AppResult<Option<Tag>> {
    Ok(
        sqlx::query_as("SELECT * FROM tags WHERE organization_id = ? AND name = ?")
            .bind(org)
            .bind(name.trim())
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn create(
    pool: &SqlitePool,
    org: &str,
    name: &str,
    color: &str,
    description: &str,
) -> AppResult<Tag> {
    let (name, color) = validate(name, color)?;
    let id = new_id();
    let ts = now();
    sqlx::query("INSERT INTO tags (id, organization_id, name, color, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(org)
        .bind(&name)
        .bind(&color)
        .bind(description.trim())
        .bind(ts)
        .bind(ts)
        .execute(pool)
        .await
        .map_err(|e| AppError::unique(e, &format!("Den Tag „{name}“ gibt es schon.")))?;
    get(pool, org, &id).await
}

pub async fn update(
    pool: &SqlitePool,
    org: &str,
    id: &str,
    name: &str,
    color: &str,
    description: &str,
) -> AppResult<Tag> {
    get(pool, org, id).await?;
    let (name, color) = validate(name, color)?;
    sqlx::query(
        "UPDATE tags SET name = ?, color = ?, description = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&color)
    .bind(description.trim())
    .bind(now())
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::unique(e, &format!("Den Tag „{name}“ gibt es schon.")))?;
    get(pool, org, id).await
}

pub async fn delete(pool: &SqlitePool, org: &str, id: &str) -> AppResult<()> {
    get(pool, org, id).await?;
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Tag an ein Dokument hängen (beide müssen zur Organisation gehören). `false`: hing schon.
pub async fn add_to_document(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
    tag_id: &str,
    user: Option<&str>,
) -> AppResult<bool> {
    let doc = documents::get(pool, org, document_id).await?;
    let tag = get(pool, org, tag_id).await?;
    let added = sqlx::query(
        "INSERT OR IGNORE INTO document_tags (document_id, tag_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(document_id)
    .bind(tag_id)
    .bind(now())
    .execute(pool)
    .await?
    .rows_affected()
        > 0;
    if added {
        documents::log(
            pool,
            document_id,
            "tag_added",
            user,
            serde_json::json!({ "tag": tag.name }),
        )
        .await?;
        let data = serde_json::json!({ "documentId": doc.id, "organizationId": org, "tagId": tag.id, "tagName": tag.name });
        webhooks::emit(pool, org, WebhookEvent::TagAdded, data).await;
    }
    Ok(added)
}

pub async fn remove_from_document(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
    tag_id: &str,
    user: Option<&str>,
) -> AppResult<bool> {
    let doc = documents::get(pool, org, document_id).await?;
    let tag = get(pool, org, tag_id).await?;
    let removed = sqlx::query("DELETE FROM document_tags WHERE document_id = ? AND tag_id = ?")
        .bind(document_id)
        .bind(tag_id)
        .execute(pool)
        .await?
        .rows_affected()
        > 0;
    if removed {
        documents::log(
            pool,
            document_id,
            "tag_removed",
            user,
            serde_json::json!({ "tag": tag.name }),
        )
        .await?;
        let data = serde_json::json!({ "documentId": doc.id, "organizationId": org, "tagId": tag.id, "tagName": tag.name });
        webhooks::emit(pool, org, WebhookEvent::TagRemoved, data).await;
    }
    Ok(removed)
}

/// Tags vieler Dokumente auf einmal (für Listen).
pub async fn for_documents(
    pool: &SqlitePool,
    document_ids: &[String],
) -> AppResult<HashMap<String, Vec<Tag>>> {
    let mut map: HashMap<String, Vec<Tag>> = HashMap::new();
    if document_ids.is_empty() {
        return Ok(map);
    }
    let mut b: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT dt.document_id AS doc, t.id, t.organization_id, t.name, t.color, t.description, t.created_at, t.updated_at
         FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id IN (",
    );
    let mut sep = b.separated(", ");
    for id in document_ids {
        sep.push_bind(id.clone());
    }
    b.push(") ORDER BY t.name COLLATE NOCASE");
    let rows: Vec<(String, String, String, String, String, String, i64, i64)> =
        b.build_query_as().fetch_all(pool).await?;
    for (doc, id, organization_id, name, color, description, created_at, updated_at) in rows {
        map.entry(doc).or_default().push(Tag {
            id,
            organization_id,
            name,
            color,
            description,
            created_at,
            updated_at,
        });
    }
    Ok(map)
}
