//! Scannen und Prüfen (aus nrzDocsScanner): die Seiten selbst sind server-seitig gerendert,
//! die Arbeit darauf (scannen, umsortieren, prüfen, freigeben) erledigt `static/scan.js` über
//! die JSON-Endpunkte hier. Alles gilt für die Organisation in der Adresse; der Benutzer muss
//! Mitglied sein.

use super::pages::shell;
use super::Page;
use crate::ai::{self, AiConfig};
use crate::auth::guards::CurrentUser;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::ocr::OcrQueue;
use crate::routes::upload::UploadForm;
use crate::services::{inbox, organizations, scan as scans, tags};
use crate::templates::{render, ReviewPage, ScanPage};
use rocket::form::Form;
use rocket::http::{ContentType, Header};
use rocket::response::Redirect;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, FromForm, Responder, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![
        scan_page,
        review_page,
        session,
        update_session,
        clear_session,
        scan_now,
        add_photos,
        page_image,
        delete_page,
        finish,
        import_pdfs,
        items,
        item_pdf,
        item_page,
        edit_item,
        approve,
        analyze,
        discard,
        scanner_status,
        discover,
        save_scanner,
        ai_test
    ]
}

async fn member(pool: &SqlitePool, user: &CurrentUser, org: &str) -> AppResult<()> {
    organizations::membership(pool, &user.0.id, org)
        .await
        .map(|_| ())
}

/// Bild oder PDF aus der Datenbank, nur für den angemeldeten Benutzer zwischenspeicherbar.
#[derive(Responder)]
struct Binary {
    body: Vec<u8>,
    content_type: ContentType,
    cache: Header<'static>,
}

impl Binary {
    fn new(body: Vec<u8>, content_type: ContentType) -> Self {
        Binary {
            body,
            content_type,
            cache: Header::new("Cache-Control", "private, max-age=86400"),
        }
    }
}

/// Tags der Organisation für die Auswahl in der Oberfläche.
async fn tag_list(pool: &SqlitePool, org: &str) -> AppResult<Value> {
    let list: Vec<Value> = tags::list(pool, org)
        .await?
        .into_iter()
        .map(|(t, _)| json!({ "id": t.id, "name": t.name, "color": t.color, "description": t.description }))
        .collect();
    Ok(json!(list))
}

// ------------------------------------------------------------------ Seiten

#[get("/organizations/<org>/scan")]
async fn scan_page(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    let settings = scans::settings(pool, config, &user.0.id).await?;
    let shell = shell(pool, user.0, org, "scan", "").await?;
    Ok(Page::Html(render(&ScanPage {
        shell,
        settings,
        ai_available: AiConfig::from(config).available(),
    })?))
}

#[get("/organizations/<org>/review")]
async fn review_page(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Page> {
    let shell = shell(pool, user.0, org, "review", "").await?;
    Ok(Page::Html(render(&ReviewPage {
        shell,
        meta_fields: ai::META_FIELDS,
    })?))
}

// ------------------------------------------------------------------ Scan-Sitzung

async fn session_json(pool: &SqlitePool, user: &str, org: &str) -> AppResult<Value> {
    Ok(
        json!({ "session": scans::session(pool, user, org).await?, "tags": tag_list(pool, org).await? }),
    )
}

#[get("/organizations/<org>/scan/session")]
async fn session(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    member(pool, &user, org).await?;
    session_json(pool, &user.0.id, org).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionUpdate {
    name: Option<String>,
    hint_tags: Option<Vec<String>>,
    pages: Option<Vec<String>>,
}

#[put("/organizations/<org>/scan/session", data = "<body>")]
async fn update_session(
    user: CurrentUser,
    org: &str,
    body: Json<SessionUpdate>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let b = body.into_inner();
    scans::update(
        pool,
        &user.0.id,
        org,
        b.name.as_deref(),
        b.hint_tags,
        b.pages,
    )
    .await?;
    session_json(pool, &user.0.id, org).await
}

#[delete("/organizations/<org>/scan/session")]
async fn clear_session(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    member(pool, &user, org).await?;
    scans::clear(pool, &user.0.id, org).await?;
    session_json(pool, &user.0.id, org).await
}

#[derive(Deserialize)]
struct ScanInput {
    color: Option<bool>,
    dpi: Option<i64>,
}

/// Eine Seite vom Glas scannen (wartet, solange der Scanner von jemand anderem belegt ist).
#[post("/organizations/<org>/scan/page", data = "<body>")]
async fn scan_now(
    user: CurrentUser,
    org: &str,
    body: Json<ScanInput>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let settings = scans::settings(pool, config, &user.0.id).await?;
    if settings.scanner_url.is_empty() {
        return Err(AppError::bad(
            "Kein Scanner eingestellt – unter Konto → Scanner eintragen oder suchen.",
        ));
    }
    let color = body.color.unwrap_or(settings.color);
    let dpi = body
        .dpi
        .filter(|d| [150, 300, 600].contains(d))
        .unwrap_or(settings.dpi) as u32;
    let jpeg = {
        let _guard = tokio::time::timeout(
            std::time::Duration::from_secs(240),
            crate::scan::SCAN_LOCK.lock(),
        )
        .await
        .map_err(|_| {
            AppError::Conflict("Der Scanner ist gerade von jemand anderem belegt.".into())
        })?;
        crate::scan::scan_page(&settings.scanner_url, color, dpi)
            .await
            .map_err(AppError::bad)?
    };
    scans::add_page(pool, &user.0.id, org, jpeg).await?;
    session_json(pool, &user.0.id, org).await
}

/// Fotos (JPEG) als Seiten anhängen.
#[post("/organizations/<org>/scan/photos", data = "<form>")]
async fn add_photos(
    user: CurrentUser,
    org: &str,
    form: Form<UploadForm>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let files = form.into_inner().file;
    if files.is_empty() {
        return Err(AppError::bad("Keine Datei erhalten."));
    }
    for f in files {
        scans::add_page(pool, &user.0.id, org, f.data).await?;
    }
    session_json(pool, &user.0.id, org).await
}

#[get("/organizations/<org>/scan/pages/<id>")]
async fn page_image(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Binary> {
    member(pool, &user, org).await?;
    Ok(Binary::new(
        scans::page(pool, &user.0.id, org, id).await?,
        ContentType::JPEG,
    ))
}

#[delete("/organizations/<org>/scan/pages/<id>")]
async fn delete_page(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    scans::delete_page(pool, &user.0.id, org, id).await?;
    session_json(pool, &user.0.id, org).await
}

/// Seiten → PDF → Prüfliste (die KI liest im Hintergrund).
#[post("/organizations/<org>/scan/finish")]
async fn finish(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let item = scans::finish(pool, &AiConfig::from(config), &user.0.id, org).await?;
    Ok(json!({ "item": item }))
}

// ------------------------------------------------------------------ Prüfliste

/// PDFs importieren – sie gehen wie Scans zuerst in die Prüfung.
#[post("/organizations/<org>/review/import", data = "<form>")]
async fn import_pdfs(
    user: CurrentUser,
    org: &str,
    form: Form<UploadForm>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let ai = AiConfig::from(config);
    let mut created = Vec::new();
    for f in form.into_inner().file {
        if !f.data.starts_with(b"%PDF") {
            return Err(AppError::bad(format!("„{}“ ist keine PDF-Datei.", f.name)));
        }
        // Bis die KI einen Namen vorschlägt, gilt der Dateiname.
        created.push(
            inbox::create(
                pool,
                &ai,
                org,
                Some(&user.0.id),
                &f.name,
                true,
                "import",
                "",
                Vec::new(),
                f.data,
                Vec::new(),
            )
            .await?,
        );
    }
    if created.is_empty() {
        return Err(AppError::bad("Keine Datei erhalten."));
    }
    Ok(json!({ "items": created }))
}

#[get("/organizations/<org>/review/items")]
async fn items(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let fields: Vec<Value> = ai::META_FIELDS
        .iter()
        .map(|(k, label)| json!({ "key": k, "label": label }))
        .collect();
    let orgs: Vec<Value> = organizations::list_for_user(pool, &user.0.id)
        .await?
        .into_iter()
        .map(|(o, _)| json!({ "id": o.id, "name": o.name }))
        .collect();
    Ok(json!({
        "items": inbox::list(pool, org).await?,
        "metaFields": fields,
        "tags": tag_list(pool, org).await?,
        "organizations": orgs,
    }))
}

#[get("/organizations/<org>/review/<id>/pdf")]
async fn item_pdf(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Binary> {
    member(pool, &user, org).await?;
    Ok(Binary::new(
        inbox::file(pool, org, id, 0).await?,
        ContentType::PDF,
    ))
}

#[get("/organizations/<org>/review/<id>/pages/<n>")]
async fn item_page(
    user: CurrentUser,
    org: &str,
    id: &str,
    n: i64,
    pool: &State<SqlitePool>,
) -> AppResult<Binary> {
    member(pool, &user, org).await?;
    if n < 1 {
        return Err(AppError::NotFound);
    }
    Ok(Binary::new(
        inbox::file(pool, org, id, n).await?,
        ContentType::JPEG,
    ))
}

/// Verschieben in eine andere Organisation nur, wenn der Benutzer auch dort Mitglied ist.
async fn check_target(pool: &SqlitePool, user: &CurrentUser, edit: &inbox::Edit) -> AppResult<()> {
    if let Some(target) = &edit.organization_id {
        organizations::membership(pool, &user.0.id, target)
            .await
            .map_err(|_| {
                AppError::bad("Diese Organisation gibt es nicht oder du bist kein Mitglied.")
            })?;
    }
    Ok(())
}

#[put("/organizations/<org>/review/<id>", data = "<body>")]
async fn edit_item(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<inbox::Edit>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    check_target(pool, &user, &body).await?;
    Ok(json!({ "item": inbox::edit(pool, org, id, body.into_inner()).await? }))
}

/// Freigeben: als Dokument ablegen (mit Tags und Eigenschaften).
#[post("/organizations/<org>/review/<id>/approve", data = "<body>")]
async fn approve(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<inbox::Edit>,
    pool: &State<SqlitePool>,
    queue: &State<OcrQueue>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    check_target(pool, &user, &body).await?;
    let target = body
        .organization_id
        .clone()
        .unwrap_or_else(|| org.to_string());
    let document_id = inbox::approve(pool, queue, org, id, &user.0.id, body.into_inner()).await?;
    Ok(
        json!({ "documentId": document_id, "url": format!("/organizations/{target}/documents/{document_id}") }),
    )
}

#[post("/organizations/<org>/review/<id>/analyze")]
async fn analyze(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    inbox::reanalyze(pool, &AiConfig::from(config), org, id).await?;
    Ok(json!({ "ok": true }))
}

#[delete("/organizations/<org>/review/<id>")]
async fn discard(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    inbox::delete(pool, org, id).await?;
    Ok(json!({ "ok": true }))
}

// ------------------------------------------------------------------ Scanner & KI

#[get("/scanner/status")]
async fn scanner_status(
    user: CurrentUser,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let settings = scans::settings(pool, config, &user.0.id).await?;
    if settings.scanner_url.is_empty() {
        return Ok(json!({ "ok": false, "state": "Kein Scanner eingestellt", "url": "" }));
    }
    Ok(match crate::scan::status(&settings.scanner_url).await {
        Ok(state) => json!({ "ok": true, "state": state, "url": settings.scanner_url }),
        Err(_) => json!({ "ok": false, "state": "Offline", "url": settings.scanner_url }),
    })
}

#[derive(Deserialize)]
struct DiscoverInput {
    #[serde(default)]
    subnets: String,
}

/// Scanner (eSCL/AirScan) in den angegebenen oder voreingestellten /24-Netzen suchen –
/// das Netz des eingestellten Scanners immer mit.
#[post("/scanner/discover", data = "<body>")]
async fn discover(
    user: CurrentUser,
    body: Json<DiscoverInput>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let source = if body.subnets.trim().is_empty() {
        config.scan_subnets.as_str()
    } else {
        body.subnets.as_str()
    };
    let mut subnets: Vec<String> = source
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let settings = scans::settings(pool, config, &user.0.id).await?;
    if let Some(host) = settings
        .scanner_url
        .split("//")
        .nth(1)
        .and_then(|h| h.split([':', '/']).next())
    {
        subnets.push(host.to_string());
    }
    let (scanners, scanned) = crate::scan::discover(&subnets).await;
    Ok(json!({ "scanners": scanners, "subnets": scanned }))
}

#[derive(FromForm)]
struct ScannerForm<'r> {
    scanner_url: &'r str,
    #[field(default = false)]
    color: bool,
    #[field(default = 300)]
    dpi: i64,
    next: Option<&'r str>,
}

/// Scanner-Einstellungen (Konto-Seite oder Scan-Seite).
#[post("/scanner/settings", data = "<form>")]
async fn save_scanner(
    user: CurrentUser,
    form: Form<ScannerForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    scans::save_settings(pool, &user.0.id, form.scanner_url, form.color, form.dpi).await?;
    Ok(Redirect::to(super::safe_next(form.next)))
}

/// Kurzer Test der KI mit einer eingebauten Beispielseite.
#[post("/ai/test")]
async fn ai_test(_user: CurrentUser, config: &State<Config>) -> AppResult<Value> {
    let ai = AiConfig::from(config);
    let tags = [ai::TagInfo {
        name: "Test".into(),
        description: "immer".into(),
    }];
    let page = include_bytes!("../../../assets/test-page.jpg").to_vec();
    let a = ai::analyze(&ai, "Test", "test.pdf", ai::Input::Pages(&[page]), &tags)
        .await
        .map_err(AppError::bad)?;
    Ok(json!({ "model": a.model, "title": a.title, "tags": a.tags, "metadata": a.metadata }))
}
