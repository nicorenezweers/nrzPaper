use tracing_subscriber::EnvFilter;

#[rocket::main]
async fn main() {
    // `nrzpaper healthcheck` – für den Docker-HEALTHCHECK (das Image hat kein curl).
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        std::process::exit(if healthcheck() { 0 } else { 1 });
    }

    tracing_subscriber::fmt()
        // Farben nur im Terminal, nicht in Docker-Logs
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .with_env_filter(
            // Rockets Routenliste beim Start und Zeilen je Anfrage (Matched/Outcome/…) nur als Warnung – jede Anfrage
            // protokolliert schon RequestLog in einer Zeile.
            EnvFilter::try_from_env("NRZPAPER_LOG").unwrap_or_else(|_| {
                EnvFilter::new(
                    "info,rocket::server=warn,rocket::launch::_=warn,rocket::shield=warn",
                )
            }),
        )
        .init();

    let rocket = match nrzpaper::build().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = rocket.launch().await {
        tracing::error!("Server beendet: {e}");
        std::process::exit(1);
    }
}

/// Fragt `/api/v1/health` auf dem eigenen Port ab; true bei Status 200.
fn healthcheck() -> bool {
    use std::io::{Read, Write};
    let port: u16 = nrzpaper::config::figment()
        .extract_inner("port")
        .unwrap_or(8320);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(3))
    else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(3)));
    let request = "GET /api/v1/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    let mut response = String::new();
    stream.write_all(request.as_bytes()).is_ok()
        && stream.read_to_string(&mut response).is_ok()
        && response.starts_with("HTTP/1.1 200")
}
