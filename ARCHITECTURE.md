# Architecture

nrzPaper is one Rust program. It serves the API, the web interface (a Dioxus/WebAssembly app embedded at build time), the PDF viewer and all background work: OCR, e-mail, watched folders, webhooks and clean-up. All data, including the files, lives in a single SQLCipher-encrypted SQLite database.

```
┌────────────── Browser ──────────────┐
│  Dioxus app (WASM)   PDF.js viewer  │
└──────────────┬──────────────────────┘
               │ /api/v1 (session cookie or API key)
┌──────────────▼──────────────────────────────────────────────────────────┐
│ Rocket 0.5                                                              │
│  routes/api/*   routes/spa (embedded UI)   routes/files (ranges, shares)│
│  services/*  ── documents, inbox, tags, folders, rules, properties,     │
│                 members, reminders, mail (IMAP), watch folders,        │
│                 webhooks, assistant (Dieter), share links              │
│  ai.rs (any provider)  ocr/ (Tesseract, Poppler)  scan/ (eSCL)         │
│  print/ (IPP)  search/ (FTS5 query language)                            │
└──────────────┬──────────────────────────────────────────────────────────┘
               │ sqlx
┌──────────────▼───────────────┐
│ nrzpaper.db (SQLCipher, FTS5) │  documents, chunks (file data), users, …
└──────────────────────────────┘
```

## Decisions

| Area | Choice | Why |
|---|---|---|
| Server | Rocket 0.5.1 | Typed guards for sessions, API keys and permissions; graceful shutdown |
| Storage | One SQLite file with **SQLCipher** (bundled, vendored OpenSSL), files stored as chunks in the database | Backup = one file plus key; nothing unencrypted on disk |
| Search | SQLite **FTS5** with prefix matching, umlaut folding and a small query language (`tag:`, `-tag:`, `name:`, `content:`, `type:`, `has:tags`, `created:`, `status:`) | No extra service |
| OCR | Tesseract + Poppler as external programs, a queue with workers | Text layer first (`pdftotext`), page images only for scans |
| Interface | **Dioxus 0.7** (Rust → WebAssembly) with components adapted from **rust-ui**; built with `dx bundle` and embedded by `build.rs` (gzip) | One binary, no CDN, works offline; UI talks only to `/api/v1` |
| PDF viewer | PDF.js 4 embedded under `/static/vendor`, loaded by `static/viewer.js` | Thumbnails, zoom, rotate, full-text search inside the PDF, range requests |
| AI | `ai.rs`: provider catalog, OpenAI chat protocol + Anthropic Messages protocol | Works with cloud and local models; PDF → images for providers without PDF input, OCR text for text-only models |
| Printing | Minimal IPP client (`print/ipp.rs`) | Every network printer speaks IPP; JPEG fallback for printers without PDF |
| i18n | German source strings as keys (`tr("…")`, `trf("… {0}", …)`), catalogs in `locales/<lang>.json` loaded on demand | Readable code; server messages translate on the client too |

## Data model (migrations)

- `0001_init`: users, sessions, API keys, organizations and members, invitations, documents (+ chunks, FTS), tags, tagging rules, custom properties, webhooks and deliveries, share links, activity log, settings
- `0002_scan`: scan sessions, scanner settings, inbox items and their files
- `0003_mail_folders_thumbnails`: thumbnails, folders made of tags, mail accounts
- `0004_reminders_folders_printers`: reminders, inbox source reference, watched folders, printers

## Security

- Passwords with Argon2id; sessions as random tokens (hash in the DB), `HttpOnly`, `SameSite=Lax`, optional `Secure`
- Changing requests with a foreign `Origin` are rejected (CSRF); API keys with fine-grained permissions
- CSP: scripts only from the server (plus `wasm-unsafe-eval` and `unsafe-eval` for the WASM app), no inline scripts, `frame-ancestors 'none'`
- AI and mail credentials are stored in the encrypted database and never returned to the browser (only a hint such as `sk-or-v1-0…cdef`)
- A built-in OpenRouter key can be added at build time as a BuildKit secret; it is never part of the source

## Build

`Dockerfile` stages:

1. `ui`: installs `dioxus-cli` and builds the WebAssembly interface (`dx bundle --release`)
2. `build`: Rust release build (cross-compiled for amd64/arm64), embeds the interface and `locales/`
3. `ocr`: Debian with Tesseract (`deu`, `eng`) and Poppler; also exports a standalone OCR package
4. `umbrel`: program plus OCR package only (for platforms that bring their own base image)
5. `runtime`: the final image (non-root, `/data` volume, health check)

## Tests

`cargo test` runs unit tests and integration tests with Rocket's local client: auth, permissions, the full API, scanning against a fake eSCL scanner, mail against a fake IMAP server, AI providers against fake servers, printing against a fake IPP printer, the assistant, translations (complete catalogs, same placeholders).
