//! Ordner: eine eigene Struktur über den Tags. Ein Ordner fasst beliebig viele Tags zusammen und
//! zeigt jedes Dokument, das mindestens einen davon trägt – Unterordner eingeschlossen. Ein
//! Dokument kann so in mehreren Ordnern auftauchen, ohne kopiert zu werden.

use crate::db::models::{new_id, now};
use crate::error::{AppError, AppResult};
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, Clone, sqlx::FromRow)]
struct Row {
    id: String,
    parent_id: Option<String>,
    name: String,
    position: i64,
}

/// Ordner in Baum-Reihenfolge (Eltern vor Kindern) mit Tiefe, Tags und Anzahl Dokumente.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub depth: usize,
    pub tag_ids: Vec<String>,
    /// Dokumente im Ordner samt Unterordnern (ohne Papierkorb)
    pub count: i64,
}

pub fn validate_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::bad("Name des Ordners: 1–100 Zeichen."));
    }
    Ok(name.to_string())
}

/// Alle Ordner einer Organisation als flacher Baum.
pub async fn tree(pool: &SqlitePool, org: &str) -> AppResult<Vec<Entry>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, parent_id, name, position FROM folders WHERE organization_id = ?",
    )
    .bind(org)
    .fetch_all(pool)
    .await?;
    let links: Vec<(String, String)> = sqlx::query_as(
        "SELECT ft.folder_id, ft.tag_id FROM folder_tags ft JOIN folders f ON f.id = ft.folder_id WHERE f.organization_id = ?",
    )
    .bind(org)
    .fetch_all(pool)
    .await?;
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();
    for (f, t) in links {
        tags.entry(f).or_default().push(t);
    }
    let mut children: HashMap<Option<String>, Vec<Row>> = HashMap::new();
    for r in rows {
        children.entry(r.parent_id.clone()).or_default().push(r);
    }
    for list in children.values_mut() {
        list.sort_by(|a, b| {
            a.position
                .cmp(&b.position)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
    }
    let mut out = Vec::new();
    fn walk(
        parent: Option<String>,
        depth: usize,
        children: &HashMap<Option<String>, Vec<Row>>,
        tags: &HashMap<String, Vec<String>>,
        out: &mut Vec<Entry>,
    ) {
        if depth > 16 {
            return;
        }
        for r in children.get(&parent).into_iter().flatten() {
            out.push(Entry {
                id: r.id.clone(),
                parent_id: r.parent_id.clone(),
                name: r.name.clone(),
                depth,
                tag_ids: tags.get(&r.id).cloned().unwrap_or_default(),
                count: 0,
            });
            walk(Some(r.id.clone()), depth + 1, children, tags, out);
        }
    }
    walk(None, 0, &children, &tags, &mut out);
    // Anzahl je Ordner (mit Unterordnern)
    for i in 0..out.len() {
        let ids = subtree_tags(&out, &out[i].id);
        out[i].count = if ids.is_empty() {
            0
        } else {
            count_with_any(pool, org, &ids).await?
        };
    }
    Ok(out)
}

/// Tags eines Ordners und aller seiner Unterordner.
pub fn subtree_tags(tree: &[Entry], id: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let mut stack = vec![id.to_string()];
    while let Some(cur) = stack.pop() {
        for e in tree {
            if e.id == cur {
                for t in &e.tag_ids {
                    if !ids.contains(t) {
                        ids.push(t.clone());
                    }
                }
            }
            if e.parent_id.as_deref() == Some(cur.as_str()) {
                stack.push(e.id.clone());
            }
        }
    }
    ids
}

async fn count_with_any(pool: &SqlitePool, org: &str, tag_ids: &[String]) -> AppResult<i64> {
    let mut b: sqlx::QueryBuilder<sqlx::Sqlite> =
        sqlx::QueryBuilder::new("SELECT count(*) FROM documents d WHERE d.organization_id = ");
    b.push_bind(org.to_string())
        .push(" AND d.deleted_at IS NULL AND EXISTS (SELECT 1 FROM document_tags dt WHERE dt.document_id = d.id AND dt.tag_id IN (");
    let mut sep = b.separated(", ");
    for id in tag_ids {
        sep.push_bind(id.clone());
    }
    b.push("))");
    Ok(b.build_query_scalar().fetch_one(pool).await?)
}

/// Prüft, dass alle Tags zur Organisation gehören.
async fn check_tags(pool: &SqlitePool, org: &str, tag_ids: &[String]) -> AppResult<()> {
    for id in tag_ids {
        crate::services::tags::get(pool, org, id).await?;
    }
    Ok(())
}

async fn set_tags(pool: &SqlitePool, id: &str, tag_ids: &[String]) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM folder_tags WHERE folder_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for t in tag_ids {
        sqlx::query("INSERT OR IGNORE INTO folder_tags (folder_id, tag_id) VALUES (?, ?)")
            .bind(id)
            .bind(t)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Elternordner prüfen: muss zur Organisation gehören und darf nicht im eigenen Teilbaum liegen.
async fn check_parent(
    pool: &SqlitePool,
    org: &str,
    id: Option<&str>,
    parent: Option<&str>,
) -> AppResult<()> {
    let Some(parent) = parent else { return Ok(()) };
    let t = tree(pool, org).await?;
    if !t.iter().any(|e| e.id == parent) {
        return Err(AppError::bad("Übergeordneter Ordner nicht gefunden."));
    }
    if let Some(id) = id {
        // parent darf weder der Ordner selbst noch einer seiner Nachfahren sein
        let mut cur = Some(parent.to_string());
        while let Some(c) = cur {
            if c == id {
                return Err(AppError::bad(
                    "Ein Ordner kann nicht in sich selbst liegen.",
                ));
            }
            cur = t
                .iter()
                .find(|e| e.id == c)
                .and_then(|e| e.parent_id.clone());
        }
    }
    Ok(())
}

pub async fn get(pool: &SqlitePool, org: &str, id: &str) -> AppResult<Entry> {
    tree(pool, org)
        .await?
        .into_iter()
        .find(|e| e.id == id)
        .ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &SqlitePool,
    org: &str,
    name: &str,
    parent: Option<&str>,
    tag_ids: &[String],
) -> AppResult<String> {
    let name = validate_name(name)?;
    check_parent(pool, org, None, parent).await?;
    check_tags(pool, org, tag_ids).await?;
    let id = new_id();
    let ts = now();
    sqlx::query(
        "INSERT INTO folders (id, organization_id, parent_id, name, position, created_at, updated_at)
         VALUES (?, ?, ?, ?, (SELECT coalesce(max(position), -1) + 1 FROM folders WHERE organization_id = ?), ?, ?)",
    )
    .bind(&id)
    .bind(org)
    .bind(parent)
    .bind(&name)
    .bind(org)
    .bind(ts)
    .bind(ts)
    .execute(pool)
    .await?;
    set_tags(pool, &id, tag_ids).await?;
    Ok(id)
}

pub async fn update(
    pool: &SqlitePool,
    org: &str,
    id: &str,
    name: &str,
    parent: Option<&str>,
    tag_ids: &[String],
) -> AppResult<()> {
    get(pool, org, id).await?;
    let name = validate_name(name)?;
    check_parent(pool, org, Some(id), parent).await?;
    check_tags(pool, org, tag_ids).await?;
    sqlx::query("UPDATE folders SET name = ?, parent_id = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(parent)
        .bind(now())
        .bind(id)
        .execute(pool)
        .await?;
    set_tags(pool, id, tag_ids).await
}

/// Ordner samt Unterordnern löschen – Dokumente und Tags bleiben.
pub async fn delete(pool: &SqlitePool, org: &str, id: &str) -> AppResult<()> {
    get(pool, org, id).await?;
    sqlx::query("DELETE FROM folders WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(id: &str, parent: Option<&str>, tags: &[&str]) -> Entry {
        Entry {
            id: id.into(),
            parent_id: parent.map(String::from),
            name: id.into(),
            depth: 0,
            tag_ids: tags.iter().map(|t| t.to_string()).collect(),
            count: 0,
        }
    }

    #[test]
    fn subtree_collects_children() {
        let t = vec![
            e("a", None, &["t1"]),
            e("b", Some("a"), &["t2", "t1"]),
            e("c", Some("b"), &["t3"]),
            e("d", None, &["t4"]),
        ];
        let mut got = subtree_tags(&t, "a");
        got.sort();
        assert_eq!(got, ["t1", "t2", "t3"]);
        assert_eq!(subtree_tags(&t, "c"), ["t3"]);
        assert_eq!(subtree_tags(&t, "d"), ["t4"]);
    }
}
