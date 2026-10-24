//! Daten der API (camelCase-JSON). Felder, die fehlen dürfen, haben `serde(default)`.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OrgRef {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Me {
    pub user: User,
    pub organizations: Vec<OrgRef>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub needs_setup: bool,
    pub registration_open: bool,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub documents_count: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Folder {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub depth: usize,
    pub tag_ids: Vec<String>,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Overview {
    pub role: String,
    pub review_count: i64,
    pub folders: Vec<Folder>,
    pub tags: Vec<Tag>,
    pub ai_available: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Document {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub sha256: String,
    pub content: String,
    pub ocr_status: String,
    pub ocr_error: Option<String>,
    pub page_count: Option<i64>,
    pub source: String,
    pub created_by: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub tags: Vec<Tag>,
}

impl Document {
    /// "pdf", "image", "text" oder "" (keine Vorschau)
    pub fn kind(&self) -> &'static str {
        if self.mime_type == "application/pdf" {
            "pdf"
        } else if self.mime_type.starts_with("image/") && self.mime_type != "image/tiff" {
            "image"
        } else if self.mime_type.starts_with("text/") {
            "text"
        } else {
            ""
        }
    }

    pub fn type_label(&self) -> &'static str {
        match self.kind() {
            "pdf" => "PDF",
            "image" => "Bild",
            "text" => "Text",
            _ => "Datei",
        }
    }

    pub fn busy(&self) -> bool {
        matches!(self.ocr_status.as_str(), "pending" | "processing")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct DocumentList {
    pub documents: Vec<Document>,
    pub documents_count: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Activity {
    pub id: String,
    pub event: String,
    pub username: Option<String>,
    pub data: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ShareLink {
    pub id: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub revoked_at: Option<i64>,
    pub view_count: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PropertyOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PropertyDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub display_order: i64,
    pub options: Vec<PropertyOption>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PropertyValue {
    pub definition_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub values: Vec<String>,
    pub display: Vec<String>,
}

pub const PROPERTY_TYPES: &[(&str, &str)] = &[
    ("text", "Text"),
    ("number", "Zahl"),
    ("date", "Datum"),
    ("boolean", "Ja/Nein"),
    ("select", "Auswahl"),
    ("multi_select", "Mehrfachauswahl"),
    ("user_relation", "Benutzer"),
    ("document_relation", "Dokument"),
];

pub fn property_type_label(kind: &str) -> &'static str {
    PROPERTY_TYPES.iter().find(|(k, _)| *k == kind).map(|(_, l)| *l).unwrap_or("?")
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Condition {
    pub field: String,
    pub operator: String,
    pub value: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TaggingRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub match_mode: String,
    pub conditions: Vec<Condition>,
    pub tag_ids: Vec<String>,
}

pub const OPERATORS: &[(&str, &str)] = &[
    ("contains", "enthält"),
    ("not_contains", "enthält nicht"),
    ("equals", "ist gleich"),
    ("starts_with", "beginnt mit"),
    ("ends_with", "endet mit"),
    ("regex", "passt auf Regex"),
];

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct InboxItem {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub status: String,
    pub step: String,
    pub error: String,
    pub warning: String,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub summary: String,
    pub model: String,
    pub page_count: i64,
    pub document_id: Option<String>,
    pub created_at: i64,
    pub source: String,
    pub source_info: String,
}

impl InboxItem {
    pub fn editable(&self) -> bool {
        matches!(self.status.as_str(), "pruefen" | "fehler")
    }
    pub fn busy(&self) -> bool {
        matches!(self.status.as_str(), "analyse" | "import")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MetaField {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct InboxList {
    pub items: Vec<InboxItem>,
    pub meta_fields: Vec<MetaField>,
    pub tags: Vec<Tag>,
    pub organizations: Vec<OrgRef>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScanSession {
    pub name: String,
    pub hint_tags: Vec<String>,
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScanState {
    pub session: ScanSession,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScannerSettings {
    pub scanner_url: String,
    pub color: bool,
    pub dpi: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MailAccount {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub security: String,
    pub username: String,
    pub folder: String,
    pub mark_seen: bool,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub interval_minutes: i64,
    pub last_check_at: Option<i64>,
    pub last_error: String,
    pub imported_count: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub permissions: Vec<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Delivery {
    pub id: String,
    pub event: String,
    pub response_status: Option<i64>,
    pub error: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Member {
    pub id: String,
    pub username: String,
    pub role: String,
    pub since: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AiSettings {
    pub disabled: bool,
    pub model: String,
    pub fallbacks: Vec<String>,
    pub effective_model: String,
    pub effective_fallbacks: Vec<String>,
    pub default_model: String,
    pub default_fallbacks: Vec<String>,
    pub key_source: String,
    pub key_hint: String,
    pub available: bool,
}

pub const WEBHOOK_EVENTS: &[(&str, &str)] = &[
    ("document:created", "Dokument angelegt"),
    ("document:updated", "Dokument geändert"),
    ("document:deleted", "Dokument gelöscht"),
    ("document:restored", "Dokument wiederhergestellt"),
    ("document:tag:added", "Tag hinzugefügt"),
    ("document:tag:removed", "Tag entfernt"),
];

pub const PERMISSIONS: &[(&str, &str)] = &[
    ("organizations:read", "Organisationen lesen"),
    ("organizations:create", "Organisationen anlegen"),
    ("organizations:update", "Organisationen verwalten"),
    ("organizations:delete", "Organisationen löschen"),
    ("documents:read", "Dokumente lesen"),
    ("documents:create", "Dokumente hochladen"),
    ("documents:update", "Dokumente ändern"),
    ("documents:delete", "Dokumente löschen"),
    ("tags:read", "Tags lesen"),
    ("tags:create", "Tags anlegen"),
    ("tags:update", "Tags ändern"),
    ("tags:delete", "Tags löschen"),
    ("custom-properties:read", "Eigenschaften lesen"),
    ("custom-properties:create", "Eigenschaften anlegen"),
    ("custom-properties:update", "Eigenschaften ändern"),
    ("custom-properties:delete", "Eigenschaften löschen"),
];

pub fn role_label(role: &str) -> &'static str {
    match role {
        "owner" => "Besitzer",
        "admin" => "Admin",
        _ => "Mitglied",
    }
}

pub fn ocr_label(status: &str) -> &'static str {
    match status {
        "pending" => "Wartet auf Texterkennung",
        "processing" => "Texterkennung läuft",
        "done" => "Text erkannt",
        "failed" => "Texterkennung fehlgeschlagen",
        _ => "Ohne Texterkennung",
    }
}
