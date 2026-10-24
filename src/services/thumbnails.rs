//! Vorschaubilder für die Kachelansicht: ein kleines JPEG der ersten Seite. Es entsteht beim
//! ersten Abruf (PDF über Poppler, Bilder über die `image`-Bibliothek) und liegt danach in der
//! Datenbank. Lässt sich keins erzeugen (z. B. Text, Poppler fehlt), wird ein leerer Eintrag
//! gespeichert – die Oberfläche zeigt dann einen Platzhalter und es wird nicht ständig neu versucht.

use crate::db::models::{now, Document};
use crate::error::AppResult;
use sqlx::SqlitePool;
use std::sync::LazyLock;

/// Breite der Vorschau in Pixeln (Kacheln sind etwa 200 px breit, doppelt für scharfe Bildschirme).
const WIDTH: u32 = 420;

/// Höchstens zwei Vorschauen gleichzeitig erzeugen.
static LIMIT: LazyLock<tokio::sync::Semaphore> = LazyLock::new(|| tokio::sync::Semaphore::new(2));

/// Vorschau holen oder erzeugen. `None`: es gibt keine (Platzhalter zeigen).
pub async fn get(pool: &SqlitePool, doc: &Document) -> AppResult<Option<Vec<u8>>> {
    if let Some(data) = stored(pool, &doc.id).await? {
        return Ok((!data.is_empty()).then_some(data));
    }
    let _permit = LIMIT.acquire().await.expect("Semaphore");
    // Vielleicht hat sie inzwischen jemand anderes erzeugt.
    if let Some(data) = stored(pool, &doc.id).await? {
        return Ok((!data.is_empty()).then_some(data));
    }
    let data = crate::storage::read_all(pool, &doc.id).await?;
    let thumb = match render(&doc.mime_type, data).await {
        Ok(t) => t,
        Err(e) => {
            tracing::debug!(document = %doc.id, "Keine Vorschau: {e}");
            Vec::new()
        }
    };
    sqlx::query("INSERT OR REPLACE INTO document_thumbnails (document_id, data, created_at) VALUES (?, ?, ?)")
        .bind(&doc.id)
        .bind(&thumb)
        .bind(now())
        .execute(pool)
        .await?;
    Ok((!thumb.is_empty()).then_some(thumb))
}

async fn stored(pool: &SqlitePool, id: &str) -> AppResult<Option<Vec<u8>>> {
    Ok(
        sqlx::query_scalar("SELECT data FROM document_thumbnails WHERE document_id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

async fn render(mime: &str, data: Vec<u8>) -> Result<Vec<u8>, String> {
    if mime == "application/pdf" {
        let width = WIDTH.to_string();
        let args = [
            "-jpeg",
            "-jpegopt",
            "quality=80",
            "-singlefile",
            "-scale-to-x",
            &width,
            "-scale-to-y",
            "-1",
            "-f",
            "1",
            "-l",
            "1",
            "fd://0",
            "-",
        ];
        let out = crate::ocr::engine::run("pdftocairo", &args, &data).await?;
        if out.is_empty() {
            return Err("pdftocairo lieferte nichts".into());
        }
        Ok(out)
    } else if mime.starts_with("image/") {
        tokio::task::spawn_blocking(move || {
            let img = image::load_from_memory(&data).map_err(|e| e.to_string())?;
            let small = img.thumbnail(WIDTH, WIDTH * 2).to_rgb8();
            let mut out = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 80)
                .encode_image(&small)
                .map_err(|e| e.to_string())?;
            Ok(out)
        })
        .await
        .map_err(|e| e.to_string())?
    } else {
        Err(format!("kein Vorschaubild für {mime}"))
    }
}

/// Platzhalter (SVG) mit dem Dateityp, wenn es keine Vorschau gibt.
pub fn placeholder(label: &str) -> String {
    let label: String = label
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(5)
        .collect();
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 210 280"><rect width="210" height="280" fill="#18181b"/><rect x="70" y="95" width="70" height="90" rx="8" fill="#27272a"/><text x="105" y="148" text-anchor="middle" font-family="system-ui,sans-serif" font-size="16" font-weight="600" fill="#71717a">{label}</text></svg>"##
    )
}
