//! Askama-Templates (Ordner `templates/`) – zur Übersetzungszeit geprüft.

use crate::db::models::{
    ApiKey, Document, DocumentSource, OcrStatus, Organization, Permission, PropertyType, Role,
    ScanSettings, ShareLink, Tag, User, Webhook, WebhookDelivery, WebhookEvent,
};
use crate::error::AppResult;
use askama::Template;
use rocket::response::content::RawHtml;

pub fn render<T: Template>(page: &T) -> AppResult<RawHtml<String>> {
    Ok(RawHtml(page.render()?))
}

/// Version für `?v=` an den eigenen Dateien (Browser laden nach Updates neu).
pub fn asset_version() -> &'static str {
    crate::routes::assets::VERSION.as_str()
}

/// Grösse lesbar: 812 B, 14,2 KB, 3,1 MB.
pub fn human_size(bytes: &i64) -> String {
    let b = *bytes as f64;
    let (v, unit) = if b >= 1_048_576.0 {
        (b / 1_048_576.0, "MB")
    } else if b >= 1024.0 {
        (b / 1024.0, "KB")
    } else {
        return format!("{bytes} B");
    };
    format!("{v:.1} {unit}").replace('.', ",")
}

/// Rahmen der App: Benutzer, aktuelle Organisation, Organisationsliste, aktiver Menüpunkt,
/// Suchtext (für das Suchfeld oben) und ob es live sucht (nur auf der Dokumentliste).
pub struct Shell {
    pub user: User,
    pub org: Organization,
    pub role: Role,
    pub orgs: Vec<(Organization, Role)>,
    pub active: &'static str,
    pub query: String,
    pub live_search: bool,
    /// Einträge der Prüfliste, die auf Prüfung warten (Zähler in der Navigation).
    pub review_count: i64,
    /// Ordner der Organisation (Baum, flach) für die Seitenleiste
    pub folders: Vec<crate::services::folders::Entry>,
    /// Geöffneter Ordner (hervorgehoben)
    pub folder_id: String,
}

impl Shell {
    pub fn org_initial(&self) -> String {
        self.org
            .name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
    }

    pub fn role_label(&self) -> &'static str {
        role_label(&self.role)
    }

    pub fn can_manage(&self) -> bool {
        crate::services::organizations::can_manage(self.role)
    }

    /// Menüpunkte: (Pfad unterhalb der Organisation, Bezeichnung, schon verfügbar)
    pub fn nav(&self) -> Vec<(&'static str, &'static str, bool)> {
        vec![
            ("", "Dokumente", true),
            ("scan", "Scannen", true),
            ("review", "Prüfen", true),
            ("tags", "Tags", true),
            ("tagging-rules", "Tagging-Regeln", false),
            ("properties", "Eigenschaften", false),
            ("trash", "Papierkorb", true),
            ("settings", "Einstellungen", true),
        ]
    }
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorPage<'a> {
    pub status: u16,
    pub title: &'a str,
    pub message: &'a str,
}

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginPage {
    pub error: Option<String>,
    pub next: String,
    pub username: String,
    pub can_register: bool,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterPage {
    pub error: Option<String>,
    pub username: String,
    pub first_user: bool,
}

#[derive(Template)]
#[template(path = "orgs/new.html")]
pub struct NewOrgPage {
    pub error: Option<String>,
    pub org_name: String,
    pub username: String,
}

// ------------------------------------------------------------------ Dokumente

/// Art der Vorschau.
pub fn preview_kind(doc: &Document) -> &'static str {
    match doc.mime_type.as_str() {
        "application/pdf" => "pdf",
        m if m.starts_with("image/") => "image",
        "text/plain" => "text",
        _ => "other",
    }
}

pub fn type_label(doc: &Document) -> &'static str {
    match preview_kind(doc) {
        "pdf" => "PDF",
        "image" => "Bild",
        "text" => "Text",
        _ => "Datei",
    }
}

pub fn ocr_label(status: &OcrStatus) -> &'static str {
    match status {
        OcrStatus::Pending => "Texterkennung wartet",
        OcrStatus::Processing => "Texterkennung läuft",
        OcrStatus::Done => "Text erkannt",
        OcrStatus::Failed => "Texterkennung fehlgeschlagen",
        OcrStatus::Skipped => "ohne Texterkennung",
    }
}

pub fn is_busy(status: &OcrStatus) -> bool {
    matches!(status, OcrStatus::Pending | OcrStatus::Processing)
}

/// Zeile der Dokumentliste.
pub struct DocRow {
    pub doc: Document,
    pub tags: Vec<Tag>,
}

/// Link auf ein Dokument; mit Suchbegriff wird er in der Vorschau markiert (`?find=`).
pub fn doc_href(org: &str, id: &str, query: &str) -> String {
    let find = find_text(query);
    if find.is_empty() {
        format!("/organizations/{org}/documents/{id}")
    } else {
        format!(
            "/organizations/{org}/documents/{id}?find={}",
            urlencode(find)
        )
    }
}

/// Seitenzahlen für die Blätterleiste (0-basiert, `None` = Auslassung).
pub fn page_window(page: &u32, pages: &u32) -> Vec<Option<u32>> {
    let (page, pages) = (*page, *pages);
    let mut out = Vec::new();
    for p in 0..pages {
        if p == 0 || p + 1 == pages || p.abs_diff(page) <= 1 {
            out.push(Some(p));
        } else if out.last() != Some(&None) {
            out.push(None);
        }
    }
    out
}

/// URL-kodiert für Links (Suche, Tags).
pub fn urlencode(s: impl AsRef<str>) -> String {
    s.as_ref()
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Nur die freien Wörter einer Suche (ohne `tag:` usw.) – zum Markieren in der Vorschau.
pub fn find_text(query: &str) -> String {
    let q = crate::search::parse(query);
    q.terms
        .iter()
        .chain(&q.name_terms)
        .chain(&q.content_terms)
        .map(|t| t.trim_matches('"'))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Ergebnisliste – auch einzeln abrufbar für die Live-Suche.
#[derive(Template)]
#[template(path = "documents/_results.html")]
pub struct ResultsPartial {
    pub org_id: String,
    /// "grid" (Kacheln) oder "list" (Tabelle)
    pub view: String,
    /// Adresse dieser Liste für die Blätterleiste (Dokumente oder ein Ordner)
    pub base_url: String,
    pub rows: Vec<DocRow>,
    pub total: i64,
    pub page: u32,
    pub pages: u32,
    pub query: String,
    pub any_documents: bool,
}

#[derive(Template)]
#[template(path = "documents/list.html")]
pub struct DocumentsPage {
    pub shell: Shell,
    /// Fertig gerenderte Ergebnisliste (dasselbe Template wie für die Live-Suche)
    pub results_html: String,
    pub tags: Vec<(Tag, i64)>,
    pub poll: bool,
    pub view: String,
    /// Geöffneter Ordner: (Name, Tags des Ordners)
    pub folder: Option<(crate::services::folders::Entry, Vec<Tag>)>,
}

impl DocumentsPage {
    /// Adresse dieser Liste (Dokumente oder Ordner), für Ansicht-Umschalter und Tag-Filter.
    pub fn base_url(&self) -> String {
        match &self.folder {
            Some((f, _)) => format!("/organizations/{}/folders/{}", self.shell.org.id, f.id),
            None => format!("/organizations/{}", self.shell.org.id),
        }
    }

    /// Ist dieser Tag schon als Filter in der Suche?
    pub fn tag_active(&self, name: &str) -> bool {
        crate::search::parse(&self.shell.query)
            .tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case(name))
    }

    /// Suche mit diesem Tag ergänzt bzw. ohne ihn.
    pub fn tag_toggle_query(&self, name: &str) -> String {
        let filter = if name.contains(' ') {
            format!("tag:\"{name}\"")
        } else {
            format!("tag:{name}")
        };
        let q = self.shell.query.trim();
        if self.tag_active(name) {
            q.replace(&filter, "")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        } else if q.is_empty() {
            filter
        } else {
            format!("{q} {filter}")
        }
    }
}

pub struct ActivityRow {
    pub label: String,
    pub user: Option<String>,
    pub at: i64,
}

#[derive(Template)]
#[template(path = "documents/detail.html")]
pub struct DetailPage {
    pub shell: Shell,
    pub doc: Document,
    pub shares: Vec<ShareLink>,
    /// Gerade angelegter Freigabelink (Pfad mit Token, nur einmal sichtbar).
    pub new_share: Option<String>,
    pub tags: Vec<Tag>,
    pub all_tags: Vec<Tag>,
    pub activity: Vec<ActivityRow>,
    pub created_by: Option<String>,
    /// Suchtext aus der Liste – wird im PDF markiert.
    pub find: String,
    /// Benutzerdefinierte Eigenschaften (z. B. Metadaten aus dem Scan)
    pub properties: Vec<crate::services::properties::Shown>,
}

impl DetailPage {
    pub fn unassigned_tags(&self) -> Vec<&Tag> {
        self.all_tags
            .iter()
            .filter(|t| !self.tags.iter().any(|x| x.id == t.id))
            .collect()
    }

    pub fn file_url(&self) -> String {
        format!(
            "/organizations/{}/documents/{}/file",
            self.doc.organization_id, self.doc.id
        )
    }

    pub fn base_url(&self) -> String {
        format!(
            "/organizations/{}/documents/{}",
            self.doc.organization_id, self.doc.id
        )
    }
}

// ------------------------------------------------------------------ Tags & Papierkorb

#[derive(Template)]
#[template(path = "tags/list.html")]
pub struct TagsPage {
    pub shell: Shell,
    pub tags: Vec<(Tag, i64)>,
    pub palette: &'static [&'static str],
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "trash.html")]
pub struct TrashPage {
    pub shell: Shell,
    pub docs: Vec<Document>,
    pub retention_days: u32,
}

impl TrashPage {
    /// Tage, bis ein Dokument endgültig gelöscht wird.
    pub fn days_left(&self, doc: &Document) -> i64 {
        let deleted = doc.deleted_at.unwrap_or(0);
        let end = deleted + i64::from(self.retention_days) * 86_400;
        ((end - crate::db::models::now()) as f64 / 86_400.0)
            .ceil()
            .max(0.0) as i64
    }
}

pub fn role_label(role: &Role) -> &'static str {
    match role {
        Role::Owner => "Besitzer",
        Role::Admin => "Admin",
        Role::Member => "Mitglied",
    }
}

pub fn permission_label(p: &Permission) -> &'static str {
    match p {
        Permission::OrganizationsRead => "Organisationen lesen",
        Permission::OrganizationsCreate => "Organisationen anlegen",
        Permission::OrganizationsUpdate => "Organisationen verwalten",
        Permission::OrganizationsDelete => "Organisationen löschen",
        Permission::DocumentsRead => "Dokumente lesen",
        Permission::DocumentsCreate => "Dokumente hochladen",
        Permission::DocumentsUpdate => "Dokumente ändern",
        Permission::DocumentsDelete => "Dokumente löschen",
        Permission::TagsRead => "Tags lesen",
        Permission::TagsCreate => "Tags anlegen",
        Permission::TagsUpdate => "Tags ändern",
        Permission::TagsDelete => "Tags löschen",
        Permission::PropertiesRead => "Eigenschaften lesen",
        Permission::PropertiesCreate => "Eigenschaften anlegen",
        Permission::PropertiesUpdate => "Eigenschaften ändern",
        Permission::PropertiesDelete => "Eigenschaften löschen",
    }
}

/// Lesende Rechte sind beim Anlegen eines Keys vorausgewählt.
pub fn permission_default(p: &Permission) -> bool {
    p.as_str().ends_with(":read")
}

pub fn event_label(e: &WebhookEvent) -> &'static str {
    match e {
        WebhookEvent::DocumentCreated => "Dokument angelegt",
        WebhookEvent::DocumentUpdated => "Dokument geändert",
        WebhookEvent::DocumentDeleted => "Dokument gelöscht",
        WebhookEvent::DocumentRestored => "Dokument wiederhergestellt",
        WebhookEvent::TagAdded => "Tag hinzugefügt",
        WebhookEvent::TagRemoved => "Tag entfernt",
    }
}

// ------------------------------------------------------------------ Konto & Einstellungen

#[derive(Template)]
#[template(path = "account.html")]
pub struct AccountPage {
    pub shell: Shell,
    pub scan: ScanSettings,
    /// KI verfügbar (Schlüssel eingebaut oder gesetzt) und welches Modell
    pub ai_available: bool,
    pub ai_model: String,
    pub keys: Vec<ApiKey>,
    /// Gerade erzeugter Key (nur dieses eine Mal sichtbar).
    pub new_key: Option<String>,
    pub notice: Option<String>,
    pub error: Option<String>,
}

pub struct HookRow {
    pub hook: Webhook,
    pub events: Vec<WebhookEvent>,
    pub deliveries: Vec<WebhookDelivery>,
}

pub struct MemberRow {
    pub username: String,
    pub role: Role,
    pub since: i64,
}

#[derive(Template)]
#[template(path = "orgs/settings.html")]
pub struct SettingsPage {
    pub shell: Shell,
    pub members: Vec<MemberRow>,
    pub hooks: Vec<HookRow>,
    /// (Name des Webhooks, Geheimnis) direkt nach dem Anlegen.
    pub new_secret: Option<(String, String)>,
    pub error: Option<String>,
}

// ------------------------------------------------------------------ Öffentliche Freigabe

#[derive(Template)]
#[template(path = "share.html")]
pub struct SharePage {
    pub doc: Document,
    pub token: String,
    pub link: ShareLink,
    pub find: String,
}

impl SharePage {
    pub fn file_url(&self) -> String {
        format!("/share/{}/file", self.token)
    }
}

// ------------------------------------------------------------------ Scannen & Prüfen

#[derive(Template)]
#[template(path = "scan/scan.html")]
pub struct ScanPage {
    pub shell: Shell,
    pub settings: ScanSettings,
    pub ai_available: bool,
}

#[derive(Template)]
#[template(path = "scan/review.html")]
pub struct ReviewPage {
    pub shell: Shell,
    pub meta_fields: &'static [(&'static str, &'static str)],
}

#[derive(Template)]
#[template(path = "folders/manage.html")]
pub struct FoldersPage {
    pub shell: Shell,
    pub tags: Vec<(Tag, i64)>,
    pub error: Option<String>,
}

impl FoldersPage {
    pub fn tag_name(&self, id: &str) -> Option<&Tag> {
        self.tags.iter().map(|(t, _)| t).find(|t| t.id == id)
    }
}
