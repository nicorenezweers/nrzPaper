//! Webhooks: bei Ereignissen (Dokument angelegt, geändert, gelöscht, Tag gesetzt …) ein
//! signiertes JSON an eine URL schicken – nach dem Schema von „Standard Webhooks“, wie Papra:
//!
//! ```text
//! webhook-id:        msg_<uuid>
//! webhook-timestamp: <Unix-Sekunden>
//! webhook-signature: v1,<Base64(HMAC-SHA256(secret, "<id>.<timestamp>.<body>"))>
//! Body:              {"type": "document:created", "timestamp": "…", "data": {…}}
//! ```
//!
//! Zustellung im Hintergrund, bis zu drei Versuche (sofort, nach 5 s, nach 30 s); jeder Versuch
//! landet im Protokoll (`webhook_deliveries`).

use crate::db::models::{new_id, now, Webhook, WebhookDelivery, WebhookEvent};
use crate::error::{AppError, AppResult};
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::SqlitePool;
use std::sync::LazyLock;
use std::time::Duration;

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("nrzPaper-Webhook/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP-Client")
});

const RETRY_DELAYS: [u64; 2] = [5, 30];

pub fn validate_url(url: &str) -> AppResult<String> {
    let url = url.trim();
    let parsed = reqwest::Url::parse(url).map_err(|_| AppError::bad("Ungültige Webhook-URL."))?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err(AppError::bad(
            "Die Webhook-URL muss mit http:// oder https:// beginnen.",
        ));
    }
    Ok(url.to_string())
}

pub fn new_secret() -> String {
    format!(
        "whsec_{}",
        base64::engine::general_purpose::STANDARD.encode(rand::random::<[u8; 24]>())
    )
}

/// Signatur nach Standard Webhooks.
pub fn sign(secret: &str, id: &str, timestamp: i64, body: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC nimmt jede Schlüssellänge");
    mac.update(format!("{id}.{timestamp}.{body}").as_bytes());
    format!(
        "v1,{}",
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    )
}

pub async fn list(pool: &SqlitePool, org: &str) -> AppResult<Vec<(Webhook, Vec<WebhookEvent>)>> {
    let hooks: Vec<Webhook> =
        sqlx::query_as("SELECT * FROM webhooks WHERE organization_id = ? ORDER BY created_at")
            .bind(org)
            .fetch_all(pool)
            .await?;
    let mut out = Vec::new();
    for h in hooks {
        let events = events_of(pool, &h.id).await?;
        out.push((h, events));
    }
    Ok(out)
}

async fn events_of(pool: &SqlitePool, id: &str) -> AppResult<Vec<WebhookEvent>> {
    Ok(
        sqlx::query_scalar("SELECT event FROM webhook_events WHERE webhook_id = ? ORDER BY event")
            .bind(id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get(pool: &SqlitePool, org: &str, id: &str) -> AppResult<Webhook> {
    sqlx::query_as("SELECT * FROM webhooks WHERE id = ? AND organization_id = ?")
        .bind(id)
        .bind(org)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Neuer Webhook; ohne Geheimnis wird eines erzeugt (und einmal angezeigt).
pub async fn create(
    pool: &SqlitePool,
    org: &str,
    user: &str,
    name: &str,
    url: &str,
    secret: Option<&str>,
    events: &[WebhookEvent],
) -> AppResult<(Webhook, String)> {
    let url = validate_url(url)?;
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::bad("Name des Webhooks: 1–100 Zeichen."));
    }
    if events.is_empty() {
        return Err(AppError::bad("Mindestens ein Ereignis wählen."));
    }
    let secret = secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(new_secret);
    let id = new_id();
    let ts = now();
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO webhooks (id, organization_id, name, url, secret, enabled, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, ?, ?, ?)")
        .bind(&id)
        .bind(org)
        .bind(name)
        .bind(&url)
        .bind(&secret)
        .bind(user)
        .bind(ts)
        .bind(ts)
        .execute(&mut *tx)
        .await?;
    for e in events {
        sqlx::query("INSERT OR IGNORE INTO webhook_events (webhook_id, event) VALUES (?, ?)")
            .bind(&id)
            .bind(e)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok((get(pool, org, &id).await?, secret))
}

pub async fn set_enabled(pool: &SqlitePool, org: &str, id: &str, enabled: bool) -> AppResult<()> {
    get(pool, org, id).await?;
    sqlx::query("UPDATE webhooks SET enabled = ?, updated_at = ? WHERE id = ?")
        .bind(enabled)
        .bind(now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, org: &str, id: &str) -> AppResult<()> {
    get(pool, org, id).await?;
    sqlx::query("DELETE FROM webhooks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn deliveries(
    pool: &SqlitePool,
    webhook_id: &str,
    limit: i64,
) -> AppResult<Vec<WebhookDelivery>> {
    Ok(sqlx::query_as("SELECT * FROM webhook_deliveries WHERE webhook_id = ? ORDER BY created_at DESC, id DESC LIMIT ?")
        .bind(webhook_id)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

/// Ereignis auslösen: passende, aktive Webhooks der Organisation bekommen es im Hintergrund.
pub async fn emit(pool: &SqlitePool, org: &str, event: WebhookEvent, data: serde_json::Value) {
    let hooks: Vec<Webhook> = match sqlx::query_as(
        "SELECT w.* FROM webhooks w JOIN webhook_events e ON e.webhook_id = w.id WHERE w.organization_id = ? AND w.enabled = 1 AND e.event = ?",
    )
    .bind(org)
    .bind(event)
    .fetch_all(pool)
    .await
    {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Webhooks für {event} nicht lesbar: {e}");
            return;
        }
    };
    if hooks.is_empty() {
        return;
    }
    let body = serde_json::json!({
        "type": event,
        "timestamp": time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
        "data": data,
    })
    .to_string();
    for hook in hooks {
        let (pool, body) = (pool.clone(), body.clone());
        tokio::spawn(async move { deliver(&pool, &hook, event, &body).await });
    }
}

/// Zustellen mit Wiederholungen; gibt zurück, ob es geklappt hat.
pub async fn deliver(pool: &SqlitePool, hook: &Webhook, event: WebhookEvent, body: &str) -> bool {
    let msg_id = format!("msg_{}", new_id());
    for attempt in 0..=RETRY_DELAYS.len() {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(RETRY_DELAYS[attempt - 1])).await;
        }
        let ts = now();
        let mut req = HTTP
            .post(&hook.url)
            .header("content-type", "application/json")
            .header("webhook-id", &msg_id)
            .header("webhook-timestamp", ts.to_string())
            .body(body.to_string());
        if let Some(secret) = hook.secret.as_deref().filter(|s| !s.is_empty()) {
            req = req.header("webhook-signature", sign(secret, &msg_id, ts, body));
        }
        let (status, response, error) = match req.send().await {
            Ok(res) => {
                let status = res.status().as_u16();
                let text = res.text().await.unwrap_or_default();
                (
                    Some(status as i64),
                    Some(text.chars().take(2000).collect::<String>()),
                    None,
                )
            }
            Err(e) => (None, None, Some(e.to_string())),
        };
        let ok = status.is_some_and(|s| (200..300).contains(&s));
        let _ = sqlx::query(
            "INSERT INTO webhook_deliveries (id, webhook_id, event, payload, response_status, response_body, error, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id())
        .bind(&hook.id)
        .bind(event)
        .bind(body)
        .bind(status)
        .bind(&response)
        .bind(&error)
        .bind(now())
        .execute(pool)
        .await;
        if ok {
            return true;
        }
        tracing::warn!(webhook = %hook.id, attempt = attempt + 1, ?status, ?error, "Webhook-Zustellung fehlgeschlagen");
    }
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn signature_matches_standard_webhooks() {
        // Gleiche Rechnung wie Papras signature.ts: HMAC-SHA256 über "<id>.<ts>.<body>", Base64, Präfix v1
        let sig = super::sign("geheim", "msg_1", 1700000000, r#"{"a":1}"#);
        // Gegenrechnung mit Python: hmac.new(b"geheim", b"msg_1.1700000000.{\"a\":1}", sha256)
        assert_eq!(sig, "v1,OW2tlUxUeQYpYOYTBNrZJxZP3icKFXRmFoP/qC2e50Q=");
    }

    #[test]
    fn urls() {
        assert!(super::validate_url("https://example.com/hook").is_ok());
        assert!(super::validate_url("http://192.168.0.245:8787/papra").is_ok());
        assert!(super::validate_url("ftp://x").is_err());
        assert!(super::validate_url("kein url").is_err());
    }
}
