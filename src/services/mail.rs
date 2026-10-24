//! E-Mail-Eingang (nur Empfang, IMAP): Jeder Benutzer kann Postfächer eintragen. Neue Nachrichten
//! werden regelmässig abgeholt; PDFs und Bilder aus den Anhängen landen – mit den vorgegebenen
//! Tags und den Vorschlägen der KI – in der Inbox (Prüfliste) und warten dort auf die Freigabe.
//!
//! Abgeholt wird alles seit dem Einrichten des Postfachs (IMAP `SINCE`); was schon verarbeitet
//! ist, merkt sich `mail_seen` (UIDVALIDITY:UID) – so kommt nichts doppelt, auch wenn die Mail
//! inzwischen am Handy gelesen wurde. Nachrichten werden nur gelesen (`BODY.PEEK`), nie gelöscht.

use crate::ai::AiConfig;
use crate::db::models::{new_id, now, MailAccount};
use crate::error::{AppError, AppResult};
use crate::services::inbox;
use futures::StreamExt;
use mail_parser::{MessageParser, MimeHeaders};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

/// Höchstens so viele Nachrichten je Abruf (der Rest kommt beim nächsten Mal).
const BATCH: usize = 40;
/// Angehängte Bilder kleiner als das sind meist Logos; eingebettete (inline) zählen erst ab
/// `MIN_INLINE_BYTES` – das sind fast immer Signaturen oder Werbung.
const MIN_IMAGE_BYTES: usize = 15 * 1024;
const MIN_INLINE_BYTES: usize = 100 * 1024;
const TIMEOUT: Duration = Duration::from_secs(60);

// ------------------------------------------------------------------ Verwaltung

/// Eingaben aus dem Formular. Leeres Passwort beim Bearbeiten = altes behalten.
#[derive(Debug, Default, Deserialize)]
pub struct AccountInput {
    pub name: String,
    pub organization_id: String,
    pub host: String,
    pub port: i64,
    pub security: String,
    pub username: String,
    pub password: String,
    pub folder: String,
    pub mark_seen: bool,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub interval_minutes: i64,
}

fn validate(i: &AccountInput) -> AppResult<()> {
    if i.host.trim().is_empty() || i.host.contains(char::is_whitespace) || i.host.contains('/') {
        return Err(AppError::bad(
            "Server (IMAP) angeben, z. B. imap.gmail.com.",
        ));
    }
    if !(1..=65535).contains(&i.port) {
        return Err(AppError::bad("Port: 1–65535 (meist 993)."));
    }
    if !matches!(i.security.as_str(), "tls" | "starttls" | "none") {
        return Err(AppError::bad("Unbekannte Verschlüsselung."));
    }
    if i.username.trim().is_empty() {
        return Err(AppError::bad(
            "Benutzername angeben (meist die E-Mail-Adresse).",
        ));
    }
    if !(1..=1440).contains(&i.interval_minutes) {
        return Err(AppError::bad("Abruf alle 1–1440 Minuten."));
    }
    Ok(())
}

fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in tags
        .iter()
        .flat_map(|t| t.split(','))
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        if !out.iter().any(|x| x.eq_ignore_ascii_case(t)) {
            out.push(t.chars().take(64).collect());
        }
    }
    out
}

pub async fn list(pool: &SqlitePool, user: &str) -> AppResult<Vec<MailAccount>> {
    Ok(
        sqlx::query_as("SELECT * FROM mail_accounts WHERE user_id = ? ORDER BY created_at")
            .bind(user)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get(pool: &SqlitePool, user: &str, id: &str) -> AppResult<MailAccount> {
    sqlx::query_as("SELECT * FROM mail_accounts WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Neues Postfach (der Aufrufer prüft die Mitgliedschaft in der Organisation).
pub async fn create(pool: &SqlitePool, user: &str, i: &AccountInput) -> AppResult<MailAccount> {
    validate(i)?;
    if i.password.is_empty() {
        return Err(AppError::bad(
            "Passwort angeben (bei Gmail/iCloud/Outlook ein App-Passwort).",
        ));
    }
    let id = new_id();
    let ts = now();
    sqlx::query(
        "INSERT INTO mail_accounts (id, user_id, organization_id, name, host, port, security, username, password, folder,
         mark_seen, tags, enabled, interval_minutes, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user)
    .bind(&i.organization_id)
    .bind(if i.name.trim().is_empty() { i.username.trim() } else { i.name.trim() })
    .bind(i.host.trim())
    .bind(i.port)
    .bind(&i.security)
    .bind(i.username.trim())
    .bind(&i.password)
    .bind(if i.folder.trim().is_empty() { "INBOX" } else { i.folder.trim() })
    .bind(i.mark_seen)
    .bind(sqlx::types::Json(clean_tags(&i.tags)))
    .bind(i.enabled)
    .bind(i.interval_minutes)
    .bind(ts)
    .bind(ts)
    .execute(pool)
    .await?;
    get(pool, user, &id).await
}

pub async fn update(
    pool: &SqlitePool,
    user: &str,
    id: &str,
    i: &AccountInput,
) -> AppResult<MailAccount> {
    let old = get(pool, user, id).await?;
    validate(i)?;
    let password = if i.password.is_empty() {
        old.password
    } else {
        i.password.clone()
    };
    sqlx::query(
        "UPDATE mail_accounts SET organization_id = ?, name = ?, host = ?, port = ?, security = ?, username = ?, password = ?,
         folder = ?, mark_seen = ?, tags = ?, enabled = ?, interval_minutes = ?, last_error = '', updated_at = ? WHERE id = ?",
    )
    .bind(&i.organization_id)
    .bind(if i.name.trim().is_empty() { i.username.trim() } else { i.name.trim() })
    .bind(i.host.trim())
    .bind(i.port)
    .bind(&i.security)
    .bind(i.username.trim())
    .bind(password)
    .bind(if i.folder.trim().is_empty() { "INBOX" } else { i.folder.trim() })
    .bind(i.mark_seen)
    .bind(sqlx::types::Json(clean_tags(&i.tags)))
    .bind(i.enabled)
    .bind(i.interval_minutes)
    .bind(now())
    .bind(id)
    .execute(pool)
    .await?;
    get(pool, user, id).await
}

pub async fn delete(pool: &SqlitePool, user: &str, id: &str) -> AppResult<()> {
    get(pool, user, id).await?;
    sqlx::query("DELETE FROM mail_accounts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ------------------------------------------------------------------ Anhänge

/// Ein Dokument aus einer Mail: Name, PDF und – bei Bildern – die Seiten als JPEG.
#[derive(Debug)]
pub struct Found {
    pub name: String,
    pub pdf: Vec<u8>,
    pub pages: Vec<Vec<u8>>,
}

/// Absender und Betreff für die Anzeige in der Inbox.
pub fn describe(raw: &[u8]) -> String {
    let Some(msg) = MessageParser::default().parse(raw) else {
        return String::new();
    };
    let from = msg
        .from()
        .and_then(|a| a.first())
        .map(|a| a.name().or(a.address()).unwrap_or("").to_string())
        .unwrap_or_default();
    let subject = msg.subject().unwrap_or("").trim();
    match (from.is_empty(), subject.is_empty()) {
        (false, false) => format!("E-Mail von {from} · {subject}"),
        (false, true) => format!("E-Mail von {from}"),
        (true, false) => format!("E-Mail · {subject}"),
        _ => "E-Mail".into(),
    }
}

/// Bild → JPEG (JPEGs bleiben, wie sie sind).
fn to_jpeg(data: &[u8]) -> Result<Vec<u8>, String> {
    if crate::scan::is_jpeg(data) {
        return Ok(data.to_vec());
    }
    let img = image::load_from_memory(data)
        .map_err(|e| e.to_string())?
        .to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode_image(&img)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// PDFs und Bilder aus den Anhängen (auch aus weitergeleiteten Mails). Kleine eingebettete
/// Bilder (Logos, Signaturen) werden übersprungen.
pub fn attachments(raw: &[u8]) -> Vec<Found> {
    let Some(msg) = MessageParser::default().parse(raw) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect(&msg, &mut out, 0);
    out
}

fn collect(msg: &mail_parser::Message<'_>, out: &mut Vec<Found>, depth: usize) {
    for part in msg.attachments() {
        if let Some(inner) = part.message() {
            if depth < 3 {
                collect(inner, out, depth + 1);
            }
            continue;
        }
        let data = part.contents();
        let name = part.attachment_name().unwrap_or("Anhang").to_string();
        let lower = name.to_lowercase();
        let (ctype, subtype) = part
            .content_type()
            .map(|c| {
                (
                    c.ctype().to_lowercase(),
                    c.subtype().unwrap_or("").to_lowercase(),
                )
            })
            .unwrap_or_default();
        let is_pdf = data.starts_with(b"%PDF")
            || subtype == "pdf" && data.windows(4).take(1024).any(|w| w == b"%PDF")
            || lower.ends_with(".pdf") && data.starts_with(b"%PDF");
        if is_pdf {
            let start = data.windows(4).position(|w| w == b"%PDF").unwrap_or(0);
            out.push(Found {
                name,
                pdf: data[start..].to_vec(),
                pages: Vec::new(),
            });
            continue;
        }
        let is_image = ctype == "image"
            || [".jpg", ".jpeg", ".png", ".webp", ".tif", ".tiff", ".heic"]
                .iter()
                .any(|e| lower.ends_with(e));
        let inline = part.content_disposition().is_some_and(|d| d.is_inline());
        let min = if inline {
            MIN_INLINE_BYTES
        } else {
            MIN_IMAGE_BYTES
        };
        if is_image && data.len() >= min {
            if let Ok(jpeg) = to_jpeg(data) {
                if let Ok(pdf) = crate::scan::make_pdf(std::slice::from_ref(&jpeg)) {
                    out.push(Found {
                        name,
                        pdf,
                        pages: vec![jpeg],
                    });
                }
            }
        }
    }
}

// ------------------------------------------------------------------ IMAP

static TLS: LazyLock<tokio_rustls::TlsConnector> = LazyLock::new(|| {
    let mut roots = tokio_rustls::rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = tokio_rustls::rustls::ClientConfig::builder_with_provider(Arc::new(
        tokio_rustls::rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("TLS-Versionen")
    .with_root_certificates(roots)
    .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
});

fn imap_err(e: impl std::fmt::Display) -> String {
    let s = e.to_string();
    if s.to_lowercase().contains("authenticat")
        || s.contains("LOGIN")
        || s.to_lowercase().contains("invalid credentials")
    {
        format!("Anmeldung abgelehnt – Benutzername/Passwort prüfen (bei Gmail, iCloud, Outlook ein App-Passwort nötig). ({s})")
    } else {
        s
    }
}

/// Ergebnis eines Abrufs.
#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub messages: usize,
    pub documents: usize,
}

/// Postfach abholen. `dry_run`: nur anmelden und den Ordner öffnen (Verbindungstest).
pub async fn fetch(
    pool: &SqlitePool,
    ai: &AiConfig,
    account: &MailAccount,
    dry_run: bool,
) -> Result<Report, String> {
    let addr = (account.host.as_str(), account.port as u16);
    let tcp = tokio::time::timeout(
        Duration::from_secs(15),
        tokio::net::TcpStream::connect(addr),
    )
    .await
    .map_err(|_| format!("{}:{} antwortet nicht.", account.host, account.port))?
    .map_err(|e| format!("Keine Verbindung zu {}:{}: {e}", account.host, account.port))?;
    let server = tokio_rustls::rustls::pki_types::ServerName::try_from(account.host.clone())
        .map_err(|e| e.to_string())?;
    let work = async {
        match account.security.as_str() {
            "tls" => {
                let tls = TLS
                    .connect(server, tcp)
                    .await
                    .map_err(|e| format!("TLS: {e}"))?;
                let mut client = async_imap::Client::new(tls);
                greeting(&mut client).await?;
                session(pool, ai, account, client, dry_run).await
            }
            "starttls" => {
                let mut client = async_imap::Client::new(tcp);
                greeting(&mut client).await?;
                client
                    .run_command_and_check_ok("STARTTLS", None)
                    .await
                    .map_err(|e| format!("STARTTLS: {e}"))?;
                let tcp = client.into_inner();
                let tls = TLS
                    .connect(server, tcp)
                    .await
                    .map_err(|e| format!("TLS: {e}"))?;
                session(pool, ai, account, async_imap::Client::new(tls), dry_run).await
            }
            _ => {
                let mut client = async_imap::Client::new(tcp);
                greeting(&mut client).await?;
                session(pool, ai, account, client, dry_run).await
            }
        }
    };
    tokio::time::timeout(TIMEOUT * 3, work)
        .await
        .map_err(|_| "Zeitlimit beim Abruf überschritten.".to_string())?
}

async fn greeting<T: AsyncRead + AsyncWrite + Unpin + std::fmt::Debug + Send>(
    client: &mut async_imap::Client<T>,
) -> Result<(), String> {
    match tokio::time::timeout(Duration::from_secs(15), client.read_response()).await {
        Ok(Some(Ok(_))) => Ok(()),
        Ok(Some(Err(e))) => Err(format!("Server: {e}")),
        Ok(None) => Err("Server hat die Verbindung geschlossen.".into()),
        Err(_) => Err("Server meldet sich nicht (falscher Port oder Verschlüsselung?).".into()),
    }
}

async fn session<T: AsyncRead + AsyncWrite + Unpin + std::fmt::Debug + Send>(
    pool: &SqlitePool,
    ai: &AiConfig,
    account: &MailAccount,
    client: async_imap::Client<T>,
    dry_run: bool,
) -> Result<Report, String> {
    let mut s = client
        .login(&account.username, &account.password)
        .await
        .map_err(|(e, _)| imap_err(e))?;
    let mailbox = s
        .select(&account.folder)
        .await
        .map_err(|e| format!("Ordner „{}“: {e}", account.folder))?;
    let validity = mailbox.uid_validity.unwrap_or(0);
    let mut report = Report::default();
    if dry_run {
        let _ = s.logout().await;
        return Ok(report);
    }
    // Alles seit dem Tag vor dem Einrichten (IMAP-Datum ohne Uhrzeit).
    let since = time::OffsetDateTime::from_unix_timestamp(account.created_at - 86_400)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let query = format!(
        "SINCE {}-{}-{}",
        since.day(),
        months[since.month() as usize - 1],
        since.year()
    );
    let mut uids: Vec<u32> = s
        .uid_search(&query)
        .await
        .map_err(|e| format!("Suche: {e}"))?
        .into_iter()
        .collect();
    uids.sort_unstable();

    let mut new_uids = Vec::new();
    for uid in uids {
        let key = format!("{validity}:{uid}");
        let seen: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM mail_seen WHERE account_id = ? AND message_key = ?")
                .bind(&account.id)
                .bind(&key)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;
        if seen.is_none() {
            new_uids.push(uid);
        }
        if new_uids.len() >= BATCH {
            break;
        }
    }

    for uid in new_uids {
        let raw: Option<Vec<u8>> = {
            let mut stream = s
                .uid_fetch(uid.to_string(), "BODY.PEEK[]")
                .await
                .map_err(|e| format!("Abruf: {e}"))?;
            let mut body = None;
            while let Some(item) = stream.next().await {
                let fetch = item.map_err(|e| format!("Abruf: {e}"))?;
                if let Some(b) = fetch.body() {
                    body = Some(b.to_vec());
                }
            }
            body
        };
        let Some(raw) = raw else { continue };
        let info = describe(&raw);
        let found = attachments(&raw);
        report.messages += 1;
        for f in found {
            inbox::create(
                pool,
                ai,
                &account.organization_id,
                Some(&account.user_id),
                &f.name,
                true,
                "mail",
                &info,
                account.tags.0.clone(),
                f.pdf,
                f.pages,
            )
            .await
            .map_err(|e| e.public_message())?;
            report.documents += 1;
        }
        sqlx::query("INSERT OR IGNORE INTO mail_seen (account_id, message_key, created_at) VALUES (?, ?, ?)")
            .bind(&account.id)
            .bind(format!("{validity}:{uid}"))
            .bind(now())
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        if account.mark_seen {
            if let Ok(mut st) = s.uid_store(uid.to_string(), "+FLAGS (\\Seen)").await {
                while st.next().await.is_some() {}
            }
        }
    }
    let _ = s.logout().await;
    Ok(report)
}

/// Abrufen und Ergebnis (Zeitpunkt, Fehler, Anzahl) am Postfach festhalten.
pub async fn run(
    pool: &SqlitePool,
    ai: &AiConfig,
    account: &MailAccount,
) -> Result<Report, String> {
    let result = fetch(pool, ai, account, false).await;
    let (error, added) = match &result {
        Ok(r) => (String::new(), r.documents as i64),
        Err(e) => (e.clone(), 0),
    };
    if let Err(e) = &result {
        tracing::warn!(account = %account.id, "E-Mail-Abruf: {e}");
    } else if added > 0 {
        tracing::info!(account = %account.id, documents = added, "E-Mail-Abruf");
    }
    let _ = sqlx::query("UPDATE mail_accounts SET last_check_at = ?, last_error = ?, imported_count = imported_count + ? WHERE id = ?")
        .bind(now())
        .bind(error)
        .bind(added)
        .bind(&account.id)
        .execute(pool)
        .await;
    result
}

/// Hintergrund: jede Minute nachsehen, welche Postfächer fällig sind.
pub fn spawn(pool: SqlitePool, ai: AiConfig, mut shutdown: rocket::Shutdown) {
    tokio::spawn(async move {
        loop {
            let due: Vec<MailAccount> = sqlx::query_as(
                "SELECT * FROM mail_accounts WHERE enabled = 1 AND (last_check_at IS NULL OR last_check_at + interval_minutes * 60 <= ?)",
            )
            .bind(now())
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
            for account in due {
                let _ = run(&pool, &ai, &account).await;
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(60)) => {}
                _ = &mut shutdown => break,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &[u8] = include_bytes!("../../assets/test-page.jpg");

    fn mail(parts: &[(&str, &str, &[u8])]) -> Vec<u8> {
        use base64::Engine;
        let mut m = String::from("From: Swisscom <rechnung@swisscom.ch>\r\nTo: nico@example.ch\r\nSubject: Ihre Rechnung Oktober\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"G\"\r\n\r\n--G\r\nContent-Type: text/plain\r\n\r\nHallo\r\n");
        for (name, ctype, data) in parts {
            m.push_str(&format!(
                "--G\r\nContent-Type: {ctype}; name=\"{name}\"\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n",
                base64::engine::general_purpose::STANDARD.encode(data)
            ));
        }
        m.push_str("--G--\r\n");
        m.into_bytes()
    }

    #[test]
    fn finds_pdfs_and_images() {
        let pdf = include_bytes!("../../tests/fixtures/text.pdf");
        let logo = vec![0u8; 500];
        let raw = mail(&[
            ("Rechnung.pdf", "application/pdf", pdf),
            ("Foto.jpg", "image/jpeg", PAGE),
            ("logo.png", "image/png", &logo),
            ("notiz.txt", "text/plain", b"x"),
        ]);
        let found = attachments(&raw);
        assert_eq!(found.len(), 2, "PDF und Foto, kein Logo, kein Text");
        assert_eq!(found[0].name, "Rechnung.pdf");
        assert_eq!(found[0].pdf, pdf);
        assert!(found[0].pages.is_empty());
        assert_eq!(found[1].name, "Foto.jpg");
        assert!(found[1].pdf.starts_with(b"%PDF"));
        assert_eq!(found[1].pages, vec![PAGE.to_vec()]);
        assert_eq!(
            describe(&raw),
            "E-Mail von Swisscom · Ihre Rechnung Oktober"
        );
    }

    #[test]
    fn png_becomes_jpeg_page() {
        let png = include_bytes!("../../tests/fixtures/scan.png");
        let raw = mail(&[("Scan.png", "image/png", png)]);
        let found = attachments(&raw);
        assert_eq!(found.len(), 1);
        assert!(crate::scan::is_jpeg(&found[0].pages[0]));
    }

    #[test]
    fn tags_cleaned() {
        assert_eq!(
            clean_tags(&["Rechnung, Mail".into(), "rechnung".into(), " ".into()]),
            ["Rechnung", "Mail"]
        );
    }
}
