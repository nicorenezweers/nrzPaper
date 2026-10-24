//! Sicherheits-Header für jede Antwort. Die Content-Security-Policy erlaubt nur die
//! eigenen Inhalte plus Tailwind vom CDN; PDF.js ist eingebettet (siehe routes::assets).

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct SecurityHeaders;

const CSP: &str = "default-src 'self'; \
    script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; \
    style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
    font-src 'self' https://fonts.gstatic.com; \
    img-src 'self' data: blob:; \
    worker-src 'self' blob:; \
    connect-src 'self'; \
    object-src 'none'; \
    frame-ancestors 'none'; \
    base-uri 'self'; \
    form-action 'self'";

#[rocket::async_trait]
impl Fairing for SecurityHeaders {
    fn info(&self) -> Info {
        Info {
            name: "Sicherheits-Header",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        for (name, value) in [
            ("Content-Security-Policy", CSP),
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("Referrer-Policy", "same-origin"),
            (
                "Permissions-Policy",
                "camera=(), microphone=(), geolocation=()",
            ),
            ("Cross-Origin-Opener-Policy", "same-origin"),
        ] {
            if !res.headers().contains(name) {
                res.set_header(Header::new(name, value));
            }
        }
    }
}
