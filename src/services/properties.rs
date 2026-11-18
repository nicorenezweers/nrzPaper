//! Benutzerdefinierte Eigenschaften eines Dokuments. Die KI-Metadaten aus dem Scan (Absender,
//! Datum, Betrag, IBAN …) landen hier: Definitionen werden je Organisation bei Bedarf angelegt,
//! Werte typgerecht gespeichert (Datum als Unix-Zeit, Betrag als Zahl).

use crate::db::models::{new_id, now, PropertyType};
use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;

/// KI-Feld → (Name der Eigenschaft, Typ). Reihenfolge = Anzeige.
pub const SCAN_FIELDS: &[(&str, &str, PropertyType)] = &[
    ("absender", "Absender", PropertyType::Text),
    ("empfaenger", "Empfänger", PropertyType::Text),
    ("datum", "Datum", PropertyType::Date),
    ("betrag", "Betrag", PropertyType::Number),
    ("waehrung", "Währung", PropertyType::Text),
    ("referenz", "Referenz / Rechnungsnr.", PropertyType::Text),
    ("kundennummer", "Kundennummer", PropertyType::Text),
    ("faellig", "Fällig am", PropertyType::Date),
    ("iban", "IBAN", PropertyType::Text),
];
pub const SUMMARY: &str = "Zusammenfassung";

/// Eine Eigenschaft eines Dokuments für die Anzeige.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    pub definition_id: String,
    pub name: String,
    pub kind: PropertyType,
    /// Für Anzeige und Formular: Datum als JJJJ-MM-TT, Zahl ohne unnötige Nullen.
    pub value: String,
}

/// Definition holen oder anlegen (Name ist je Organisation eindeutig, ohne Gross/klein).
pub async fn ensure(
    pool: &SqlitePool,
    org: &str,
    name: &str,
    kind: PropertyType,
    order: i64,
) -> AppResult<(String, PropertyType)> {
    let ts = now();
    sqlx::query(
        "INSERT INTO custom_property_definitions (id, organization_id, name, type, display_order, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT (organization_id, name) DO NOTHING",
    )
    .bind(new_id())
    .bind(org)
    .bind(name)
    .bind(kind)
    .bind(order)
    .bind(ts)
    .bind(ts)
    .execute(pool)
    .await?;
    Ok(sqlx::query_as(
        "SELECT id, type FROM custom_property_definitions WHERE organization_id = ? AND name = ?",
    )
    .bind(org)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

/// "2026-10-15" → Unix-Sekunden (Mitternacht UTC).
fn parse_date(raw: &str) -> Option<i64> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    let date = time::Date::parse(raw.trim().get(..10)?, format).ok()?;
    Some(date.midnight().assume_utc().unix_timestamp())
}

/// "1'250.00", "1 250,50", "CHF 79.90" → Zahl.
fn parse_number(raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-'))
        .collect();
    let cleaned = if cleaned.contains('.') {
        cleaned.replace(',', "")
    } else {
        cleaned.replace(',', ".")
    };
    cleaned.parse().ok().filter(|n: &f64| n.is_finite())
}

fn format_date(ts: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(ts)
        .map(|d| format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day()))
        .unwrap_or_default()
}

fn format_number(n: f64) -> String {
    let s = format!("{n:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Wert setzen (leer = entfernen). Passt der Text nicht zum Typ, gibt es einen Fehler.
pub async fn set(
    pool: &SqlitePool,
    document_id: &str,
    definition_id: &str,
    kind: PropertyType,
    raw: &str,
) -> AppResult<()> {
    let raw = raw.trim();
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM document_property_values WHERE document_id = ? AND definition_id = ?")
        .bind(document_id)
        .bind(definition_id)
        .execute(&mut *tx)
        .await?;
    if !raw.is_empty() {
        let q = sqlx::query(
            "INSERT INTO document_property_values (id, document_id, definition_id, text_value, number_value, date_value, boolean_value)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id())
        .bind(document_id)
        .bind(definition_id);
        let q = match kind {
            PropertyType::Number => {
                let n = parse_number(raw)
                    .ok_or_else(|| AppError::bad(format!("„{raw}“ ist keine Zahl.")))?;
                q.bind(None::<String>)
                    .bind(Some(n))
                    .bind(None::<i64>)
                    .bind(None::<bool>)
            }
            PropertyType::Date => {
                let d = parse_date(raw).ok_or_else(|| {
                    AppError::bad(format!("„{raw}“ ist kein Datum (JJJJ-MM-TT)."))
                })?;
                q.bind(None::<String>)
                    .bind(None::<f64>)
                    .bind(Some(d))
                    .bind(None::<bool>)
            }
            PropertyType::Boolean => {
                let b = matches!(raw.to_lowercase().as_str(), "ja" | "true" | "1" | "yes");
                q.bind(None::<String>)
                    .bind(None::<f64>)
                    .bind(None::<i64>)
                    .bind(Some(b))
            }
            _ => q
                .bind(Some(raw.chars().take(2000).collect::<String>()))
                .bind(None::<f64>)
                .bind(None::<i64>)
                .bind(None::<bool>),
        };
        q.execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Metadaten und Zusammenfassung aus der Prüfung übernehmen. Unpassende Werte (z. B. ein Datum,
/// das keins ist) werden als Text-Hinweis zurückgegeben statt die Ablage scheitern zu lassen.
pub async fn apply_scan_metadata(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
    metadata: &std::collections::BTreeMap<String, String>,
    summary: &str,
) -> AppResult<Vec<String>> {
    let mut warnings = Vec::new();
    for (order, (key, name, kind)) in SCAN_FIELDS.iter().enumerate() {
        let Some(value) = metadata.get(*key).filter(|v| !v.trim().is_empty()) else {
            continue;
        };
        let (def, actual) = ensure(pool, org, name, *kind, order as i64).await?;
        if let Err(e) = set(pool, document_id, &def, actual, value).await {
            warnings.push(format!("{name}: {}", e.public_message()));
        }
    }
    if !summary.trim().is_empty() {
        let (def, actual) = ensure(
            pool,
            org,
            SUMMARY,
            PropertyType::Text,
            SCAN_FIELDS.len() as i64,
        )
        .await?;
        set(pool, document_id, &def, actual, summary).await?;
    }
    Ok(warnings)
}

/// Alle gesetzten Eigenschaften eines Dokuments, in Anzeige-Reihenfolge.
pub async fn for_document(pool: &SqlitePool, document_id: &str) -> AppResult<Vec<Shown>> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, String, PropertyType, Option<String>, Option<f64>, Option<i64>, Option<bool>)> = sqlx::query_as(
        "SELECT d.id, d.name, d.type, v.text_value, v.number_value, v.date_value, v.boolean_value
         FROM document_property_values v JOIN custom_property_definitions d ON d.id = v.definition_id
         WHERE v.document_id = ? ORDER BY d.display_order, d.name COLLATE NOCASE",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(definition_id, name, kind, text, number, date, boolean)| {
            let value = match (text, number, date, boolean) {
                (Some(t), ..) => t,
                (_, Some(n), ..) => format_number(n),
                (_, _, Some(d), _) => format_date(d),
                (_, _, _, Some(b)) => if b { "Ja" } else { "Nein" }.to_string(),
                _ => String::new(),
            };
            Shown {
                definition_id,
                name,
                kind,
                value,
            }
        })
        .collect())
}

/// Definition einer Organisation (für das Bearbeiten auf der Dokumentseite).
pub async fn definition(pool: &SqlitePool, org: &str, id: &str) -> AppResult<PropertyType> {
    sqlx::query_scalar(
        "SELECT type FROM custom_property_definitions WHERE id = ? AND organization_id = ?",
    )
    .bind(id)
    .bind(org)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_values() {
        assert_eq!(parse_number("1'250.00"), Some(1250.0));
        assert_eq!(parse_number("CHF 79.90"), Some(79.9));
        assert_eq!(parse_number("1 250,50"), Some(1250.5));
        assert_eq!(parse_number("abc"), None);
        let d = parse_date("2026-10-15").unwrap();
        assert_eq!(format_date(d), "2026-10-15");
        assert_eq!(parse_date("15.10.2026"), None);
        assert_eq!(format_number(79.9), "79.9");
        assert_eq!(format_number(1250.0), "1250");
    }
}

// ------------------------------------------------------------------ Verwaltung (alle Typen)

/// Definition samt Auswahlmöglichkeiten.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    #[serde(flatten)]
    pub definition: crate::db::models::PropertyDefinition,
    pub options: Vec<crate::db::models::PropertyOption>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub kind: PropertyType,
    /// Nur für Auswahl/Mehrfachauswahl: die Möglichkeiten in Reihenfolge
    #[serde(default)]
    pub options: Vec<String>,
}

fn validate_definition(i: &DefinitionInput) -> AppResult<Vec<String>> {
    let name = i.name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(AppError::bad("Name der Eigenschaft: 1–64 Zeichen."));
    }
    let mut options: Vec<String> = Vec::new();
    for o in i.options.iter().map(|o| o.trim()).filter(|o| !o.is_empty()) {
        if o.chars().count() > 100 {
            return Err(AppError::bad("Auswahlmöglichkeiten höchstens 100 Zeichen."));
        }
        if !options.iter().any(|x| x.eq_ignore_ascii_case(o)) {
            options.push(o.to_string());
        }
    }
    if matches!(i.kind, PropertyType::Select | PropertyType::MultiSelect) && options.is_empty() {
        return Err(AppError::bad(
            "Auswahl braucht mindestens eine Möglichkeit.",
        ));
    }
    Ok(options)
}

pub async fn definitions(pool: &SqlitePool, org: &str) -> AppResult<Vec<Definition>> {
    let defs: Vec<crate::db::models::PropertyDefinition> = sqlx::query_as(
        "SELECT * FROM custom_property_definitions WHERE organization_id = ? ORDER BY display_order, name COLLATE NOCASE",
    )
    .bind(org)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(defs.len());
    for d in defs {
        let options = sqlx::query_as("SELECT * FROM custom_property_options WHERE definition_id = ? ORDER BY display_order, name")
            .bind(&d.id)
            .fetch_all(pool)
            .await?;
        out.push(Definition {
            definition: d,
            options,
        });
    }
    Ok(out)
}

pub async fn get_definition(pool: &SqlitePool, org: &str, id: &str) -> AppResult<Definition> {
    definitions(pool, org)
        .await?
        .into_iter()
        .find(|d| d.definition.id == id)
        .ok_or(AppError::NotFound)
}

/// Möglichkeiten abgleichen: bestehende (gleicher Name) behalten ihre ID, neue kommen dazu,
/// entfernte verschwinden samt ihren Werten.
async fn sync_options(
    tx: &mut sqlx::SqliteConnection,
    def: &str,
    options: &[String],
) -> AppResult<()> {
    let existing: Vec<(String, String)> =
        sqlx::query_as("SELECT id, name FROM custom_property_options WHERE definition_id = ?")
            .bind(def)
            .fetch_all(&mut *tx)
            .await?;
    for (id, name) in &existing {
        if !options.iter().any(|o| o.eq_ignore_ascii_case(name)) {
            sqlx::query("DELETE FROM custom_property_options WHERE id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }
    for (i, o) in options.iter().enumerate() {
        match existing.iter().find(|(_, n)| n.eq_ignore_ascii_case(o)) {
            Some((id, _)) => {
                sqlx::query(
                    "UPDATE custom_property_options SET name = ?, display_order = ? WHERE id = ?",
                )
                .bind(o)
                .bind(i as i64)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }
            None => {
                sqlx::query("INSERT INTO custom_property_options (id, definition_id, name, display_order) VALUES (?, ?, ?, ?)")
                    .bind(new_id())
                    .bind(def)
                    .bind(o)
                    .bind(i as i64)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }
    Ok(())
}

pub async fn create_definition(
    pool: &SqlitePool,
    org: &str,
    i: &DefinitionInput,
) -> AppResult<Definition> {
    let options = validate_definition(i)?;
    let id = new_id();
    let ts = now();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO custom_property_definitions (id, organization_id, name, description, type, display_order, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, (SELECT coalesce(max(display_order), -1) + 1 FROM custom_property_definitions WHERE organization_id = ?), ?, ?)",
    )
    .bind(&id)
    .bind(org)
    .bind(i.name.trim())
    .bind(i.description.trim())
    .bind(i.kind)
    .bind(org)
    .bind(ts)
    .bind(ts)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::unique(e, &format!("Die Eigenschaft „{}“ gibt es schon.", i.name.trim())))?;
    if matches!(i.kind, PropertyType::Select | PropertyType::MultiSelect) {
        sync_options(&mut tx, &id, &options).await?;
    }
    tx.commit().await?;
    get_definition(pool, org, &id).await
}

/// Name, Beschreibung und Möglichkeiten ändern. Der Typ bleibt (sonst passen gespeicherte Werte nicht).
pub async fn update_definition(
    pool: &SqlitePool,
    org: &str,
    id: &str,
    i: &DefinitionInput,
) -> AppResult<Definition> {
    let current = get_definition(pool, org, id).await?;
    if current.definition.kind != i.kind {
        return Err(AppError::bad(
            "Der Typ einer Eigenschaft lässt sich nicht ändern – neue Eigenschaft anlegen.",
        ));
    }
    let options = validate_definition(i)?;
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE custom_property_definitions SET name = ?, description = ?, updated_at = ? WHERE id = ?")
        .bind(i.name.trim())
        .bind(i.description.trim())
        .bind(now())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::unique(e, &format!("Die Eigenschaft „{}“ gibt es schon.", i.name.trim())))?;
    if matches!(i.kind, PropertyType::Select | PropertyType::MultiSelect) {
        sync_options(&mut tx, id, &options).await?;
    }
    tx.commit().await?;
    get_definition(pool, org, id).await
}

pub async fn delete_definition(pool: &SqlitePool, org: &str, id: &str) -> AppResult<()> {
    get_definition(pool, org, id).await?;
    sqlx::query("DELETE FROM custom_property_definitions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Reihenfolge der Eigenschaften festlegen (IDs in neuer Reihenfolge).
pub async fn reorder(pool: &SqlitePool, org: &str, ids: &[String]) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    for (i, id) in ids.iter().enumerate() {
        sqlx::query("UPDATE custom_property_definitions SET display_order = ? WHERE id = ? AND organization_id = ?")
            .bind(i as i64)
            .bind(id)
            .bind(org)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Wert(e) einer Eigenschaft am Dokument – roh für Formulare und lesbar für die Anzeige.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub definition_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: PropertyType,
    /// Text, Zahl, JJJJ-MM-TT, "true"/"false" bzw. IDs (Möglichkeit, Benutzer, Dokument)
    pub values: Vec<String>,
    /// Lesbar, z. B. Namen der gewählten Möglichkeiten
    pub display: Vec<String>,
}

/// Alle Eigenschaften der Organisation mit den Werten dieses Dokuments (auch leere).
pub async fn values(pool: &SqlitePool, org: &str, document_id: &str) -> AppResult<Vec<Value>> {
    let defs = definitions(pool, org).await?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, Option<String>, Option<f64>, Option<i64>, Option<bool>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT definition_id, text_value, number_value, date_value, boolean_value, option_id, user_id, related_document_id
         FROM document_property_values WHERE document_id = ? ORDER BY rowid",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::with_capacity(defs.len());
    for d in defs {
        let mut values = Vec::new();
        let mut display = Vec::new();
        for (_, text, number, date, boolean, option, user, related) in
            rows.iter().filter(|r| r.0 == d.definition.id)
        {
            if let Some(t) = text {
                values.push(t.clone());
                display.push(t.clone());
            } else if let Some(n) = number {
                values.push(format_number(*n));
                display.push(format_number(*n));
            } else if let Some(ts) = date {
                values.push(format_date(*ts));
                display.push(format_date(*ts));
            } else if let Some(b) = boolean {
                values.push(b.to_string());
                display.push(if *b { "Ja" } else { "Nein" }.into());
            } else if let Some(o) = option {
                values.push(o.clone());
                display.push(
                    d.options
                        .iter()
                        .find(|x| &x.id == o)
                        .map(|x| x.name.clone())
                        .unwrap_or_default(),
                );
            } else if let Some(u) = user {
                values.push(u.clone());
                let name: Option<String> =
                    sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
                        .bind(u)
                        .fetch_optional(pool)
                        .await?;
                display.push(name.unwrap_or_default());
            } else if let Some(r) = related {
                values.push(r.clone());
                let name: Option<String> =
                    sqlx::query_scalar("SELECT name FROM documents WHERE id = ?")
                        .bind(r)
                        .fetch_optional(pool)
                        .await?;
                display.push(name.unwrap_or_default());
            }
        }
        out.push(Value {
            definition_id: d.definition.id.clone(),
            name: d.definition.name.clone(),
            kind: d.definition.kind,
            values,
            display,
        });
    }
    Ok(out)
}

/// Werte einer Eigenschaft setzen (leer = entfernen). Mehrere Werte nur bei Mehrfach-Typen.
/// Auswahl nimmt die ID oder den Namen der Möglichkeit; Benutzer müssen Mitglied sein,
/// Dokumente zur selben Organisation gehören.
pub async fn set_values(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
    definition_id: &str,
    raw: &[String],
) -> AppResult<()> {
    let def = get_definition(pool, org, definition_id).await?;
    let kind = def.definition.kind;
    let raw: Vec<&str> = raw
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .collect();
    if raw.len() > 1 && !kind.is_multi() {
        return Err(AppError::bad(format!(
            "„{}“ hat nur einen Wert.",
            def.definition.name
        )));
    }
    if !matches!(
        kind,
        PropertyType::Select
            | PropertyType::MultiSelect
            | PropertyType::UserRelation
            | PropertyType::DocumentRelation
    ) {
        return set(
            pool,
            document_id,
            definition_id,
            kind,
            raw.first().copied().unwrap_or(""),
        )
        .await;
    }
    // Erst alles prüfen, dann schreiben.
    let mut resolved: Vec<String> = Vec::new();
    for v in &raw {
        let id = match kind {
            PropertyType::Select | PropertyType::MultiSelect => def
                .options
                .iter()
                .find(|o| o.id == *v || o.name.eq_ignore_ascii_case(v))
                .map(|o| o.id.clone())
                .ok_or_else(|| {
                    AppError::bad(format!(
                        "„{v}“ ist keine Möglichkeit von „{}“.",
                        def.definition.name
                    ))
                })?,
            PropertyType::UserRelation => {
                let found: Option<String> = sqlx::query_scalar(
                    "SELECT u.id FROM users u JOIN organization_members m ON m.user_id = u.id WHERE m.organization_id = ? AND (u.id = ? OR u.username = ?)",
                )
                .bind(org)
                .bind(v)
                .bind(v)
                .fetch_optional(pool)
                .await?;
                found.ok_or_else(|| {
                    AppError::bad(format!("„{v}“ ist kein Mitglied der Organisation."))
                })?
            }
            _ => {
                if *v == document_id {
                    return Err(AppError::bad(
                        "Ein Dokument kann nicht auf sich selbst verweisen.",
                    ));
                }
                crate::services::documents::get(pool, org, v).await?.id
            }
        };
        if !resolved.contains(&id) {
            resolved.push(id);
        }
    }
    let column = match kind {
        PropertyType::Select | PropertyType::MultiSelect => "option_id",
        PropertyType::UserRelation => "user_id",
        _ => "related_document_id",
    };
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM document_property_values WHERE document_id = ? AND definition_id = ?")
        .bind(document_id)
        .bind(definition_id)
        .execute(&mut *tx)
        .await?;
    for id in resolved {
        // Spaltenname ist eine feste Auswahl oben, kein Eingabewert.
        let sql = format!("INSERT INTO document_property_values (id, document_id, definition_id, {column}) VALUES (?, ?, ?, ?)");
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(new_id())
            .bind(document_id)
            .bind(definition_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
