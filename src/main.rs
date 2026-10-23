use tracing_subscriber::EnvFilter;

#[rocket::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("NRZPAPER_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
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
