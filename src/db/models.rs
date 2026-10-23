//! Datenmodelle – eine Struktur je Tabelle aus `migrations/0001_init.sql`.
//! Abfragen laufen zur Laufzeit (`query_as`), da die Makros zur Übersetzungszeit eine
//! unverschlüsselte Datenbank bräuchten.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Neue ID: UUIDv7 (zeitlich sortierbar) als Text.
pub fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// Jetzt als Unix-Sekunden (UTC) – so stehen Zeitpunkte in der Datenbank.
pub fn now() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

/// Aufzählungen, die als TEXT gespeichert werden (siehe CHECK-Bedingungen im Schema).
macro_rules! text_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
        #[sqlx(type_name = "TEXT")]
        pub enum $name {
            $(
                #[sqlx(rename = $text)]
                #[serde(rename = $text)]
                $variant,
            )+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self { $($name::$variant => $text),+ }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($text => Ok($name::$variant),)+
                    other => Err(format!("unbekannter Wert für {}: {other}", stringify!($name))),
                }
            }
        }
    };
}

text_enum!(
    /// Rolle in einer Organisation.
    Role { Owner = "owner", Admin = "admin", Member = "member" }
);

impl Role {
    /// Darf Mitglieder, Regeln, Webhooks und Eigenschaften verwalten.
    pub fn can_manage(self) -> bool {
        matches!(self, Role::Owner | Role::Admin)
    }
}

text_enum!(
    /// Stand der Texterkennung.
    OcrStatus { Pending = "pending", Processing = "processing", Done = "done", Failed = "failed", Skipped = "skipped" }
);

text_enum!(
    /// Woher ein Dokument kam.
    DocumentSource { Web = "web", Api = "api", Ingestion = "ingestion" }
);

text_enum!(
    MatchMode { All = "all", Any = "any" }
);

text_enum!(
    ConditionField { Name = "name", Content = "content" }
);

text_enum!(
    ConditionOperator {
        Contains = "contains",
        NotContains = "not_contains",
        Equals = "equals",
        StartsWith = "starts_with",
        EndsWith = "ends_with",
        Regex = "regex",
    }
);

text_enum!(
    PropertyType {
        Text = "text",
        Number = "number",
        Date = "date",
        Boolean = "boolean",
        Select = "select",
        MultiSelect = "multi_select",
        UserRelation = "user_relation",
        DocumentRelation = "document_relation",
    }
);

impl PropertyType {
    /// Mehrere Werte je Dokument erlaubt.
    pub fn is_multi(self) -> bool {
        matches!(
            self,
            PropertyType::MultiSelect | PropertyType::UserRelation | PropertyType::DocumentRelation
        )
    }
}

text_enum!(
    /// Ereignisse für Webhooks (wie Papra).
    WebhookEvent {
        DocumentCreated = "document:created",
        DocumentUpdated = "document:updated",
        DocumentDeleted = "document:deleted",
        DocumentRestored = "document:restored",
        TagAdded = "document:tag:added",
        TagRemoved = "document:tag:removed",
    }
);

text_enum!(
    /// Berechtigungen eines API-Keys (als JSON-Liste in `api_keys.permissions`).
    Permission {
        OrganizationsRead = "organizations:read",
        OrganizationsCreate = "organizations:create",
        OrganizationsUpdate = "organizations:update",
        OrganizationsDelete = "organizations:delete",
        DocumentsRead = "documents:read",
        DocumentsCreate = "documents:create",
        DocumentsUpdate = "documents:update",
        DocumentsDelete = "documents:delete",
        TagsRead = "tags:read",
        TagsCreate = "tags:create",
        TagsUpdate = "tags:update",
        TagsDelete = "tags:delete",
        PropertiesRead = "custom-properties:read",
        PropertiesCreate = "custom-properties:create",
        PropertiesUpdate = "custom-properties:update",
        PropertiesDelete = "custom-properties:delete",
    }
);

// ============================================================ Benutzer & Anmeldung

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    #[serde(skip)]
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub token_hash: String,
    pub user_id: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_seen_at: i64,
    pub user_agent: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ApiKey {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub prefix: String,
    #[serde(skip)]
    pub key_hash: String,
    /// JSON-Liste von `Permission`
    pub permissions: sqlx::types::Json<Vec<Permission>>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub expires_at: Option<i64>,
}

// ============================================================ Organisationen

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct OrganizationMember {
    pub organization_id: String,
    pub user_id: String,
    pub role: Role,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Invitation {
    pub id: String,
    pub organization_id: String,
    pub invitee: String,
    pub role: Role,
    pub invited_by: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub accepted_at: Option<i64>,
}

// ============================================================ Dokumente

/// Ein Dokument ohne Datei-Inhalt (der liegt in `document_chunks`).
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Document {
    #[serde(skip)]
    pub num: i64,
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub sha256: String,
    pub document_date: Option<i64>,
    pub content: String,
    pub page_count: Option<i64>,
    pub ocr_status: OcrStatus,
    pub ocr_error: Option<String>,
    pub source: DocumentSource,
    pub created_by: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub deleted_by: Option<String>,
}

impl Document {
    pub fn is_pdf(&self) -> bool {
        self.mime_type == "application/pdf"
    }

    pub fn in_trash(&self) -> bool {
        self.deleted_at.is_some()
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DocumentActivity {
    pub id: String,
    pub document_id: String,
    pub event: String,
    pub user_id: Option<String>,
    pub data: sqlx::types::Json<serde_json::Value>,
    pub created_at: i64,
}

// ============================================================ Tags & Regeln

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Tag {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub color: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TaggingRule {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub match_mode: MatchMode,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TaggingRuleCondition {
    pub id: String,
    pub rule_id: String,
    pub field: ConditionField,
    pub operator: ConditionOperator,
    pub value: String,
    pub case_sensitive: bool,
}

// ============================================================ Eigenschaften

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PropertyDefinition {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub kind: PropertyType,
    pub display_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PropertyOption {
    pub id: String,
    pub definition_id: String,
    pub name: String,
    pub display_order: i64,
}

/// Ein gespeicherter Wert; genau eine der Wert-Spalten ist gefüllt.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PropertyValue {
    pub id: String,
    pub document_id: String,
    pub definition_id: String,
    pub text_value: Option<String>,
    pub number_value: Option<f64>,
    pub date_value: Option<i64>,
    pub boolean_value: Option<bool>,
    pub option_id: Option<String>,
    pub user_id: Option<String>,
    pub related_document_id: Option<String>,
}

// ============================================================ Freigaben & Webhooks

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ShareLink {
    pub id: String,
    pub document_id: String,
    #[serde(skip)]
    pub token_hash: String,
    pub created_by: Option<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub revoked_at: Option<i64>,
    pub view_count: i64,
}

impl ShareLink {
    pub fn is_active(&self, at: i64) -> bool {
        self.revoked_at.is_none() && self.expires_at.is_none_or(|e| e > at)
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Webhook {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub url: String,
    #[serde(skip)]
    pub secret: Option<String>,
    pub enabled: bool,
    pub created_by: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event: String,
    pub payload: String,
    pub response_status: Option<i64>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub created_at: i64,
}
