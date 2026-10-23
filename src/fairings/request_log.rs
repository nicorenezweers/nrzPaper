//! Protokolliert jede Anfrage mit Status und Dauer über `tracing`.

use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};
use std::time::Instant;

pub struct RequestLog;

#[derive(Clone, Copy)]
struct Started(Option<Instant>);

#[rocket::async_trait]
impl Fairing for RequestLog {
    fn info(&self) -> Info {
        Info { name: "Anfrage-Protokoll", kind: Kind::Request | Kind::Response }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        req.local_cache(|| Started(Some(Instant::now())));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let ms = req.local_cache(|| Started(None)).0.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        let status = res.status().code;
        // Statische Dateien und der Health-Check nur auf debug-Stufe.
        if req.uri().path().starts_with("/static/") || req.uri().path() == "/api/v1/health" {
            tracing::debug!(method = %req.method(), path = %req.uri().path(), status, ms, "Anfrage");
        } else {
            tracing::info!(method = %req.method(), path = %req.uri().path(), status, ms, "Anfrage");
        }
    }
}
