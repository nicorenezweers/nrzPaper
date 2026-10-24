//! Dokumente in der Oberfläche: Upload (Drag & Drop, Antwort als JSON) und die Datei selbst
//! für Vorschau und Download (mit Range-Anfragen für die PDF-Vorschau).

use crate::auth::guards::CurrentUser;
use crate::db::models::DocumentSource;
use crate::error::{AppError, AppResult};
use crate::ocr::OcrQueue;
use crate::routes::upload::UploadForm;
use crate::services::documents::{self, Ingested, NewDocument};
use crate::services::organizations;
use crate::storage::{FileResponse, RangeHeader};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Value};
use rocket::{get, post, routes, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![upload, file, thumb]
}

/// Mehrere Dateien auf einmal; je Datei Erfolg oder Fehler.
#[post("/organizations/<org>/documents", data = "<form>")]
async fn upload(
    user: CurrentUser,
    org: &str,
    form: Form<UploadForm>,
    pool: &State<SqlitePool>,
    queue: &State<OcrQueue>,
) -> AppResult<Custom<Value>> {
    organizations::membership(pool, &user.0.id, org).await?;
    let mut created = Vec::new();
    let mut errors = Vec::new();
    for f in form.into_inner().file {
        let name = f.name.clone();
        let new = NewDocument {
            organization_id: org,
            user_id: Some(&user.0.id),
            source: DocumentSource::Web,
            file_name: &f.name,
            data: f.data,
        };
        match documents::ingest(pool, queue, new).await {
            Ok(i) => created.push(json!({ "id": i.document().id, "name": i.document().name, "restored": matches!(i, Ingested::Restored(_)) })),
            Err(e @ (AppError::Conflict(_) | AppError::BadRequest(_))) => errors.push(json!({ "file": name, "error": e.public_message() })),
            Err(e) => return Err(e),
        }
    }
    let status = if created.is_empty() && !errors.is_empty() {
        Status::Conflict
    } else {
        Status::Created
    };
    Ok(Custom(
        status,
        json!({ "documents": created, "errors": errors }),
    ))
}

#[derive(rocket::Responder)]
pub enum FileReply {
    File(FileResponse),
    Unsatisfiable(Custom<String>),
}

/// Datei ausliefern: `?download` als Anhang, sonst inline; `Range` → 206.
pub async fn serve(
    pool: &SqlitePool,
    org: &str,
    id: &str,
    range: Option<&str>,
    download: bool,
) -> AppResult<FileReply> {
    let doc = documents::get(pool, org, id).await?;
    Ok(
        match FileResponse::load(pool, &doc, range, download).await? {
            Ok(f) => FileReply::File(f),
            Err(size) => FileReply::Unsatisfiable(Custom(
                Status::RangeNotSatisfiable,
                format!("bytes */{size}"),
            )),
        },
    )
}

#[get("/organizations/<org>/documents/<id>/file?<download>")]
async fn file(
    user: CurrentUser,
    org: &str,
    id: &str,
    download: Option<bool>,
    range: RangeHeader,
    pool: &State<SqlitePool>,
) -> AppResult<FileReply> {
    organizations::membership(pool, &user.0.id, org).await?;
    serve(pool, org, id, range.0.as_deref(), download.unwrap_or(false)).await
}

/// Vorschaubild (JPEG) oder – wenn es keins gibt – ein Platzhalter (SVG).
#[derive(rocket::Responder)]
pub enum Thumb {
    Jpeg(
        Vec<u8>,
        rocket::http::ContentType,
        rocket::http::Header<'static>,
    ),
    Svg(
        String,
        rocket::http::ContentType,
        rocket::http::Header<'static>,
    ),
}

#[get("/organizations/<org>/documents/<id>/thumb")]
async fn thumb(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Thumb> {
    organizations::membership(pool, &user.0.id, org).await?;
    let doc = documents::get(pool, org, id).await?;
    let cache = rocket::http::Header::new("Cache-Control", "private, max-age=604800");
    Ok(match crate::services::thumbnails::get(pool, &doc).await? {
        Some(jpeg) => Thumb::Jpeg(jpeg, rocket::http::ContentType::JPEG, cache),
        None => Thumb::Svg(
            crate::services::thumbnails::placeholder(&crate::templates::type_label(&doc)),
            rocket::http::ContentType::SVG,
            rocket::http::Header::new("Cache-Control", "private, max-age=3600"),
        ),
    })
}
