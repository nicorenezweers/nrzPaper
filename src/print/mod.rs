//! Drucken über das Netz (IPP): Drucker im Netz finden, Angaben abfragen, Dokumente schicken.
//! PDFs gehen direkt an Drucker, die PDF verstehen (die meisten aktuellen); sonst werden die
//! Seiten mit Poppler zu JPEG-Bildern (verstehen AirPrint-Drucker immer). Bilder gehen als JPEG.

pub mod ipp;

use crate::db::models::{new_id, now};
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::LazyLock;
use std::time::Duration;

/// Drucker haben oft selbst signierte Zertifikate (ipps://) – nur für Drucker zugelassen.
static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("HTTP-Client")
});

/// Was ein Drucker über sich sagt.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub url: String,
    pub name: String,
    pub make: String,
    pub formats: Vec<String>,
    /// idle, processing oder stopped
    pub state: String,
    pub color: bool,
    pub duplex: bool,
    pub message: String,
}

async fn call(url: &str, req: &ipp::Request, data: &[u8], timeout: Duration) -> Result<ipp::Response, String> {
    let mut body = req.encode();
    body.extend_from_slice(data);
    let resp = HTTP
        .post(url)
        .header("Content-Type", "application/ipp")
        .timeout(timeout)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Drucker nicht erreichbar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Drucker antwortet mit HTTP {}.", resp.status().as_u16()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    ipp::Response::parse(&bytes)
}

/// Angaben eines Druckers (`printer` als ipp://, ipps://, http:// oder nur Host).
pub async fn query(printer: &str) -> Result<Info, String> {
    query_timeout(printer, Duration::from_secs(8)).await
}

async fn query_timeout(printer: &str, timeout: Duration) -> Result<Info, String> {
    let url = ipp::http_url(printer)?;
    let mut req = ipp::Request::new(ipp::GET_PRINTER_ATTRIBUTES, &ipp::printer_uri(&url));
    req.op_keywords(
        "requested-attributes",
        &[
            "printer-name",
            "printer-info",
            "printer-make-and-model",
            "document-format-supported",
            "printer-state",
            "printer-state-message",
            "color-supported",
            "sides-supported",
        ],
    );
    let r = call(&url, &req, &[], timeout).await?;
    if !r.ok() {
        return Err(format!("Kein IPP-Drucker unter {url} (Status {:#06x}).", r.status));
    }
    let state = match r.first("printer-state") {
        Some("3") => "idle",
        Some("4") => "processing",
        Some("5") => "stopped",
        _ => "unknown",
    };
    let name = r
        .first("printer-info")
        .filter(|s| !s.is_empty())
        .or(r.first("printer-name"))
        .unwrap_or("Drucker")
        .to_string();
    Ok(Info {
        url: url.clone(),
        name,
        make: r.first("printer-make-and-model").unwrap_or("").to_string(),
        formats: r.all("document-format-supported"),
        state: state.into(),
        color: r.first("color-supported") == Some("true"),
        duplex: r.all("sides-supported").iter().any(|s| s.starts_with("two-sided")),
        message: r.first("printer-state-message").unwrap_or("").to_string(),
    })
}

/// Sucht in jedem /24-Netz nach IPP-Druckern (Port 631) – parallel, kurze Zeitgrenze.
pub async fn discover(subnets: &[String]) -> (Vec<Info>, Vec<String>) {
    let mut prefixes: Vec<String> = subnets.iter().filter_map(|s| crate::scan::subnet_prefix(s)).collect();
    prefixes.sort();
    prefixes.dedup();
    let limit = std::sync::Arc::new(tokio::sync::Semaphore::new(96));
    let mut jobs = tokio::task::JoinSet::new();
    for p in &prefixes {
        for host in 1..=254u8 {
            let (limit, ip) = (limit.clone(), format!("{p}.{host}"));
            jobs.spawn(async move {
                let _permit = limit.acquire().await.ok()?;
                // erst prüfen, ob der Port offen ist – spart viele HTTP-Versuche
                let addr = format!("{ip}:631");
                tokio::time::timeout(Duration::from_millis(700), tokio::net::TcpStream::connect(&addr))
                    .await
                    .ok()?
                    .ok()?;
                for path in ["/ipp/print", "/ipp", "/ipp/printer", "/"] {
                    if let Ok(i) = query_timeout(&format!("ipp://{ip}:631{path}"), Duration::from_secs(4)).await {
                        return Some(i);
                    }
                }
                None
            });
        }
    }
    let mut found = Vec::new();
    while let Some(r) = jobs.join_next().await {
        if let Ok(Some(i)) = r {
            found.push(i);
        }
    }
    found.sort_by(|a, b| a.url.cmp(&b.url));
    (found, prefixes.into_iter().map(|p| format!("{p}.0/24")).collect())
}

/// Druckoptionen aus der Oberfläche.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(default = "one")]
    pub copies: i32,
    #[serde(default)]
    pub duplex: bool,
    #[serde(default)]
    pub grayscale: bool,
}

fn one() -> i32 {
    1
}

async fn print_job(url: &str, user: &str, job: &str, mime: &str, data: &[u8], o: &Options, info: &Info) -> Result<i64, String> {
    let mut req = ipp::Request::new(ipp::PRINT_JOB, &ipp::printer_uri(url));
    req.user(user).job_name(job).format(mime);
    if o.copies > 1 {
        req.copies(o.copies.min(99));
    }
    if o.duplex && info.duplex {
        req.keyword("sides", "two-sided-long-edge");
    }
    if o.grayscale && info.color {
        req.keyword("print-color-mode", "monochrome");
    }
    let r = call(url, &req, data, Duration::from_secs(120)).await?;
    if !r.ok() {
        let msg = r.first("status-message").unwrap_or("");
        return Err(format!("Der Drucker hat den Auftrag abgelehnt (Status {:#06x}) {msg}", r.status).trim().to_string());
    }
    Ok(r.first("job-id").and_then(|s| s.parse().ok()).unwrap_or(0))
}

/// Ein Dokument drucken. Gibt die Auftragsnummern des Druckers zurück.
pub async fn print_document(printer: &str, user: &str, name: &str, mime: &str, data: &[u8], o: &Options) -> Result<Vec<i64>, String> {
    let info = query(printer).await?;
    let can = |m: &str| info.formats.iter().any(|f| f == m);
    let url = info.url.clone();
    if mime == "application/pdf" {
        if can("application/pdf") {
            return Ok(vec![print_job(&url, user, name, "application/pdf", data, o, &info).await?]);
        }
        if !can("image/jpeg") {
            return Err("Der Drucker nimmt weder PDF noch JPEG an.".into());
        }
        let pages = crate::ai::pdf_pages(data, 200, 300).await;
        if pages.is_empty() {
            return Err("Das PDF liess sich nicht in Bilder umwandeln (Poppler fehlt?).".into());
        }
        let mut jobs = Vec::new();
        for (i, page) in pages.iter().enumerate() {
            jobs.push(print_job(&url, user, &format!("{name} ({})", i + 1), "image/jpeg", page, o, &info).await?);
        }
        return Ok(jobs);
    }
    if mime.starts_with("image/") {
        let jpeg = crate::scan::to_jpeg(data)?;
        if can("image/jpeg") {
            return Ok(vec![print_job(&url, user, name, "image/jpeg", &jpeg, o, &info).await?]);
        }
        if can("application/pdf") {
            let pdf = crate::scan::make_pdf(std::slice::from_ref(&jpeg))?;
            return Ok(vec![print_job(&url, user, name, "application/pdf", &pdf, o, &info).await?]);
        }
        return Err("Der Drucker nimmt weder JPEG noch PDF an.".into());
    }
    if mime.starts_with("text/") && can("text/plain") {
        return Ok(vec![print_job(&url, user, name, "text/plain", data, o, &info).await?]);
    }
    Err("Diese Dateiart lässt sich nicht drucken – nur PDFs, Bilder und Text.".into())
}

// ------------------------------------------------------------------ gespeicherte Drucker

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Printer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub formats: sqlx::types::Json<Vec<String>>,
    pub make: String,
    pub created_at: i64,
}

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Printer>> {
    Ok(sqlx::query_as("SELECT * FROM printers ORDER BY name COLLATE NOCASE").fetch_all(pool).await?)
}

pub async fn get(pool: &SqlitePool, id: &str) -> AppResult<Printer> {
    sqlx::query_as("SELECT * FROM printers WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Drucker hinzufügen: wird zuerst abgefragt (muss erreichbar sein). Eigener Name optional.
pub async fn add(pool: &SqlitePool, url: &str, name: &str) -> AppResult<Printer> {
    let info = query(url).await.map_err(AppError::bad)?;
    let name = name.trim();
    let name = if name.is_empty() { info.name.clone() } else { name.chars().take(100).collect() };
    let id = new_id();
    sqlx::query("INSERT INTO printers (id, name, url, formats, make, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&name)
        .bind(&info.url)
        .bind(sqlx::types::Json(&info.formats))
        .bind(&info.make)
        .bind(now())
        .execute(pool)
        .await
        .map_err(|e| AppError::unique(e, "Dieser Drucker ist schon eingerichtet."))?;
    get(pool, &id).await
}

pub async fn rename(pool: &SqlitePool, id: &str, name: &str) -> AppResult<Printer> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::bad("Name des Druckers: 1–100 Zeichen."));
    }
    get(pool, id).await?;
    sqlx::query("UPDATE printers SET name = ? WHERE id = ?").bind(name).bind(id).execute(pool).await?;
    get(pool, id).await
}

pub async fn remove(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let n = sqlx::query("DELETE FROM printers WHERE id = ?").bind(id).execute(pool).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// Gegen einen echten Drucker: `NRZ_TEST_PRINTER=ipp://… cargo test printer_live -- --ignored`
    #[tokio::test]
    #[ignore]
    async fn printer_live() {
        let Ok(p) = std::env::var("NRZ_TEST_PRINTER") else { return };
        let info = super::query(&p).await.expect("Drucker antwortet");
        println!("{info:#?}");
        assert!(!info.formats.is_empty());
    }
}
